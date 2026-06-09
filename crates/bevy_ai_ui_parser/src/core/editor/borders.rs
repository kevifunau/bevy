use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_math::Rect;
use bevy_picking::Pickable;
use bevy_text::{FontSize, FontSource, TextColor, TextFont};
use bevy_ui::prelude::*;
use bevy_ui::ui_transform::UiGlobalTransform;
use bevy_ui::ComputedNode;

use crate::core::editor::state::{
    BuiEditorBorder, BuiEditorOverlayRoot, BuiEditorState, EditorMode,
};
use crate::core::model::{BuiDocument, BuiNode, BuiNodeType};
use crate::core::runtime::components::{BuiDocumentResource, BuiIdMap, BuiRootEntity};

pub(crate) fn sync_editor_border_system(
    mut commands: Commands,
    editor_state: Res<BuiEditorState>,
    id_map: Res<BuiIdMap>,
    root_entity: Option<Res<BuiRootEntity>>,
    doc_resource: Option<Res<BuiDocumentResource>>,
    target_camera_query: Query<&UiTargetCamera>,
    existing_overlays: Query<Entity, With<BuiEditorOverlayRoot>>,
    existing_borders: Query<Entity, With<BuiEditorBorder>>,
) {
    let is_active = matches!(
        editor_state.mode,
        EditorMode::Active | EditorMode::AwaitingSaveDialog
    );

    if !is_active {
        for entity in &existing_overlays {
            commands.entity(entity).despawn();
        }
        if existing_overlays.is_empty() {
            for entity in &existing_borders {
                commands.entity(entity).despawn();
            }
        }
        return;
    }

    if !existing_borders.is_empty() {
        return;
    }

    let Some(doc_resource) = doc_resource else {
        return;
    };
    let inherited_target_camera =
        resolve_ui_target_camera(root_entity.as_deref(), &target_camera_query);

    let overlay_root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            GlobalZIndex(9998),
            BuiEditorOverlayRoot,
        ))
        .id();
    commands.entity(overlay_root).insert(Pickable::IGNORE);
    if let Some(target_camera) = inherited_target_camera.clone() {
        commands.entity(overlay_root).insert(target_camera);
    }

    let editor_badge = commands
        .spawn((
            Text::new("EDITOR ACTIVE"),
            TextFont {
                font: FontSource::default(),
                font_size: FontSize::Px(14.0),
                ..TextFont::default()
            },
            TextColor(Color::srgb(0.9, 1.0, 0.9)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..Node::default()
            },
            BackgroundColor(Color::srgba(0.06, 0.22, 0.08, 0.86)),
            BorderColor::all(Color::srgba(0.5, 1.0, 0.5, 0.8)),
            Pickable::IGNORE,
        ))
        .id();
    if let Some(target_camera) = inherited_target_camera.clone() {
        commands.entity(editor_badge).insert(target_camera);
    }
    commands.entity(overlay_root).add_child(editor_badge);

    let border_ids = collect_editor_border_ids(&doc_resource.0);
    debug_trace(&format!(
        "spawning editor overlay with {} border candidates",
        border_ids.len()
    ));

    for node_id in border_ids {
        let Some(target_entity) = id_map.0.get(&node_id).copied() else {
            continue;
        };
        let border_entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    border: UiRect::all(Val::Px(2.0)),
                    ..Node::default()
                },
                BorderColor::all(Color::srgba(0.0, 1.0, 0.0, 0.0)),
                BuiEditorBorder {
                    node_id,
                    target_entity,
                },
            ))
            .id();
        if let Some(target_camera) = inherited_target_camera.clone() {
            commands.entity(border_entity).insert(target_camera);
        }

        commands.entity(overlay_root).add_child(border_entity);
    }

    commands.queue(move |world: &mut World| {
        if let Some(mut state) = world.get_resource_mut::<BuiEditorState>() {
            state.overlay_root_entity = Some(overlay_root);
        }
    });
}

pub(crate) fn update_editor_border_visibility_system(
    editor_state: Res<BuiEditorState>,
    mut border_query: Query<(&BuiEditorBorder, &mut BorderColor, &mut Node), With<BuiEditorBorder>>,
) {
    let hovered_id = editor_state.hovered_node_id.as_deref();
    let dragged_id = editor_state.dragged_node_id.as_deref();

    for (border, mut border_color, mut node) in &mut border_query {
        let is_dragged = dragged_id == Some(border.node_id.as_str());
        let is_hovered = hovered_id == Some(border.node_id.as_str());

        if is_dragged {
            *border_color = BorderColor::all(Color::srgba(0.1, 1.0, 0.1, 0.95));
            node.border = UiRect::all(Val::Px(3.0));
        } else if is_hovered {
            *border_color = BorderColor::all(Color::srgba(0.0, 1.0, 0.0, 0.9));
            node.border = UiRect::all(Val::Px(2.0));
        } else {
            *border_color = BorderColor::all(Color::srgba(0.0, 1.0, 0.0, 0.0));
            node.border = UiRect::all(Val::Px(2.0));
        }
    }
}

fn collect_editor_border_ids(document: &BuiDocument) -> Vec<String> {
    let mut ids = Vec::new();
    collect_editor_border_ids_inner(&document.root, false, &mut ids);
    ids
}

fn collect_editor_border_ids_inner(node: &BuiNode, inside_control: bool, ids: &mut Vec<String>) {
    if should_draw_editor_border(node, inside_control) {
        ids.push(node.id.clone());
    }

    let child_inside_control = inside_control || is_control_container(node);
    for child in &node.children {
        collect_editor_border_ids_inner(child, child_inside_control, ids);
    }
}

fn should_draw_editor_border(node: &BuiNode, inside_control: bool) -> bool {
    match node.node_type() {
        BuiNodeType::Button | BuiNodeType::TextInput | BuiNodeType::Toggle => true,
        BuiNodeType::Text | BuiNodeType::Image => !inside_control,
        BuiNodeType::Node => false,
    }
}

fn is_control_container(node: &BuiNode) -> bool {
    matches!(
        node.node_type(),
        BuiNodeType::Button | BuiNodeType::TextInput | BuiNodeType::Toggle
    )
}

pub(crate) fn update_border_positions_system(
    mut border_query: Query<(&BuiEditorBorder, &mut Node), With<BuiEditorBorder>>,
    target_query: Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
) {
    for (border, mut border_node) in &mut border_query {
        let Ok((target_transform, target_computed)) = target_query.get(border.target_entity) else {
            continue;
        };

        let Some(computed_node) = target_computed else {
            continue;
        };

        let size = computed_node.size();
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }

        let inverse_scale = computed_node.inverse_scale_factor();
        let logical_center = target_transform.translation * inverse_scale;
        let logical_size = size * inverse_scale;
        let logical_rect = Rect::from_center_size(logical_center, logical_size);

        border_node.left = Val::Px(logical_rect.min.x);
        border_node.top = Val::Px(logical_rect.min.y);
        border_node.width = Val::Px(logical_size.x);
        border_node.height = Val::Px(logical_size.y);
    }
}

fn resolve_ui_target_camera(
    root_entity: Option<&BuiRootEntity>,
    target_camera_query: &Query<&UiTargetCamera>,
) -> Option<UiTargetCamera> {
    root_entity
        .and_then(|root| target_camera_query.get(root.0).ok())
        .cloned()
}

fn debug_trace(message: &str) {
    let enabled = std::env::var("BUI_EDITOR_DEBUG_TRACE")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);

    if enabled {
        eprintln!("[bui-editor-debug] {message}");
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::{bui_node, text_node, BuiDocument, BuiResources, BuiStateModel};

    use super::collect_editor_border_ids;

    #[test]
    fn collect_editor_border_ids_skips_button_internal_text() {
        let mut root = bui_node("root", "node");

        let mut button = bui_node("action_button", "button");
        button.children.push(text_node(
            "action_button_label",
            "Start",
            18.0,
            "#FFFFFF",
            None,
        ));

        root.children.push(button);
        root.children
            .push(text_node("screen_title", "Title", 24.0, "#FFFFFF", None));

        let document = BuiDocument {
            version: "3.0-ir".to_string(),
            scene_name: "BorderTest".to_string(),
            imports: Vec::new(),
            state_model: BuiStateModel::default(),
            resources: BuiResources::default(),
            root,
        };

        let ids = collect_editor_border_ids(&document);

        assert!(ids.contains(&"action_button".to_string()));
        assert!(ids.contains(&"screen_title".to_string()));
        assert!(!ids.contains(&"action_button_label".to_string()));
    }
}
