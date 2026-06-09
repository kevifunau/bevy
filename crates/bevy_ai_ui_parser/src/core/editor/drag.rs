use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::*;
use bevy_input::prelude::*;
use bevy_math::Rect;
use bevy_picking::hover::HoverMap;
use bevy_ui::prelude::*;
use bevy_ui::ui_transform::UiGlobalTransform;
use bevy_ui::ComputedNode;
use bevy_window::Window;

use crate::core::editor::state::{BuiEdit, BuiEditorBorder, BuiEditorState, EditorMode};
use crate::core::model::BuiDocument;
use crate::core::runtime::components::{BuiDocumentResource, BuiIdMap};
use crate::core::support::tree::{find_bui_node_mut, find_bui_node_ref};

pub(crate) fn editor_drag_system(
    mut editor_state: ResMut<BuiEditorState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    hover_map: Res<HoverMap>,
    windows: Query<&Window>,
    border_query: Query<(Entity, &BuiEditorBorder), With<BuiEditorBorder>>,
    mut target_node_query: Query<&mut Node>,
    target_layout_query: Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
    child_of_query: Query<&ChildOf>,
    id_map: Res<BuiIdMap>,
    mut doc_resource: ResMut<BuiDocumentResource>,
) {
    if editor_state.mode != EditorMode::Active {
        return;
    }

    if editor_state.dragged_node_id.is_some() {
        let dragged_id = editor_state.dragged_node_id.clone().unwrap();

        if !mouse_input.pressed(MouseButton::Left) {
            if editor_state.drag_has_moved {
                let old_left = editor_state
                    .drag_origin_style_left
                    .clone()
                    .unwrap_or_default();
                let old_top = editor_state
                    .drag_origin_style_top
                    .clone()
                    .unwrap_or_default();
                let new_left = format!("{}px", editor_state.drag_current_left.unwrap_or(0.0));
                let new_top = format!("{}px", editor_state.drag_current_top.unwrap_or(0.0));

                let dragged_id = dragged_id.clone();
                if apply_drag_to_document(&mut doc_resource.0, &dragged_id, &new_left, &new_top) {
                    editor_state.pending_edits.push(BuiEdit::PositionChange {
                        node_id: dragged_id,
                        old_left,
                        old_top,
                        new_left,
                        new_top,
                    });
                }
            }
            editor_state.dragged_node_id = None;
            editor_state.drag_origin_cursor = None;
            editor_state.drag_origin_pos = None;
            editor_state.drag_origin_style_left = None;
            editor_state.drag_origin_style_top = None;
            editor_state.drag_current_left = None;
            editor_state.drag_current_top = None;
            editor_state.drag_has_moved = false;
            return;
        }

        let Some(cursor) = get_cursor_pos(&windows) else {
            return;
        };
        let Some(origin_cursor) = editor_state.drag_origin_cursor else {
            return;
        };
        let Some((origin_left, origin_top)) = editor_state.drag_origin_pos else {
            return;
        };

        let new_left = origin_left + (cursor.0 - origin_cursor.0);
        let new_top = origin_top + (cursor.1 - origin_cursor.1);
        let drag_distance = (cursor.0 - origin_cursor.0).abs() + (cursor.1 - origin_cursor.1).abs();

        if drag_distance > 0.5 {
            editor_state.drag_has_moved = true;
        }

        editor_state.drag_current_left = Some(new_left);
        editor_state.drag_current_top = Some(new_top);

        if editor_state.drag_has_moved {
            if let Some(entity) = id_map.0.get(&dragged_id) {
                if let Ok(mut node) = target_node_query.get_mut(*entity) {
                    node.left = Val::Px(new_left);
                    node.top = Val::Px(new_top);
                    node.right = Val::Auto;
                    node.bottom = Val::Auto;
                }
            }
        }
    } else if mouse_input.just_pressed(MouseButton::Left) {
        let Some(cursor) = get_cursor_pos(&windows) else {
            return;
        };

        for (entity, border) in &border_query {
            let is_hovered = hover_map
                .iter()
                .any(|(_, hovered)| hovered.contains_key(&entity));
            if !is_hovered {
                continue;
            }

            let node = find_bui_node_ref(&doc_resource.0.root, &border.node_id);
            let Some(node) = node else { continue };

            let position_type = node.layout.styles.position_type.as_deref();
            if position_type != Some("absolute") && position_type != Some("Absolute") {
                continue;
            }

            let origin_pos = current_absolute_offsets_px(
                border.target_entity,
                &target_layout_query,
                &child_of_query,
            )
            .unwrap_or_else(|| {
                (
                    parse_px_value(node.layout.styles.left.as_deref()),
                    parse_px_value(node.layout.styles.top.as_deref()),
                )
            });

            editor_state.dragged_node_id = Some(border.node_id.clone());
            editor_state.drag_origin_cursor = Some(cursor);
            editor_state.drag_origin_pos = Some(origin_pos);
            editor_state.drag_origin_style_left = node.layout.styles.left.clone();
            editor_state.drag_origin_style_top = node.layout.styles.top.clone();
            editor_state.drag_current_left = Some(origin_pos.0);
            editor_state.drag_current_top = Some(origin_pos.1);
            editor_state.drag_has_moved = false;
            return;
        }
    }
}

fn current_absolute_offsets_px(
    entity: Entity,
    layout_query: &Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
    child_of_query: &Query<&ChildOf>,
) -> Option<(f32, f32)> {
    let child_rect = logical_rect(entity, layout_query)?;
    let parent_min = child_of_query
        .get(entity)
        .ok()
        .and_then(|child_of| logical_rect(child_of.parent(), layout_query))
        .map(|rect| rect.min)
        .unwrap_or_default();

    Some((
        child_rect.min.x - parent_min.x,
        child_rect.min.y - parent_min.y,
    ))
}

fn logical_rect(
    entity: Entity,
    layout_query: &Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
) -> Option<Rect> {
    let (transform, computed) = layout_query.get(entity).ok()?;
    let computed = computed?;
    let size = computed.size();
    if size.x <= 0.0 || size.y <= 0.0 {
        return None;
    }

    let inverse_scale = computed.inverse_scale_factor();
    let logical_center = transform.translation * inverse_scale;
    let logical_size = size * inverse_scale;
    Some(Rect::from_center_size(logical_center, logical_size))
}

fn apply_drag_to_document(
    document: &mut BuiDocument,
    node_id: &str,
    left: &str,
    top: &str,
) -> bool {
    let Some(node) = find_bui_node_mut(&mut document.root, node_id) else {
        return false;
    };

    node.layout.styles.left = Some(left.to_string());
    node.layout.styles.top = Some(top.to_string());
    node.layout.styles.right = None;
    node.layout.styles.bottom = None;
    true
}

fn get_cursor_pos(windows: &Query<&Window>) -> Option<(f32, f32)> {
    for window in windows.iter() {
        if let Some(pos) = window.cursor_position() {
            return Some((pos.x, pos.y));
        }
    }
    None
}

fn parse_px_value(value: Option<&str>) -> f32 {
    value
        .and_then(|v| v.strip_suffix("px"))
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use bevy_ecs::hierarchy::ChildOf;
    use bevy_ecs::prelude::{Query, World};
    use bevy_math::Vec2;
    use bevy_ui::ui_transform::UiGlobalTransform;
    use bevy_ui::ComputedNode;

    use crate::core::model::bui_node;
    use crate::core::support::tree::find_bui_node_ref;

    use super::{apply_drag_to_document, current_absolute_offsets_px};

    #[test]
    fn apply_drag_to_document_updates_left_and_top() {
        let mut root = bui_node("root", "node");
        let mut child = bui_node("absolute_child", "node");
        child.layout.styles.position_type = Some("absolute".to_string());
        root.children.push(child);
        let mut document = crate::core::model::BuiDocument {
            version: "3.0-ir".to_string(),
            scene_name: "DragTest".to_string(),
            imports: Vec::new(),
            state_model: crate::core::model::BuiStateModel::default(),
            resources: crate::core::model::BuiResources::default(),
            root,
        };

        let updated = apply_drag_to_document(&mut document, "absolute_child", "42px", "18px");

        assert!(updated);
        let child =
            find_bui_node_ref(&document.root, "absolute_child").expect("child should exist");
        assert_eq!(child.layout.styles.left.as_deref(), Some("42px"));
        assert_eq!(child.layout.styles.top.as_deref(), Some("18px"));
    }

    #[test]
    fn current_absolute_offsets_px_uses_parent_relative_position() {
        let mut world = World::new();
        let parent = world
            .spawn((
                UiGlobalTransform::from_translation(Vec2::new(100.0, 60.0)),
                ComputedNode {
                    size: Vec2::new(200.0, 120.0),
                    content_size: Vec2::new(200.0, 120.0),
                    unrounded_size: Vec2::new(200.0, 120.0),
                    ..ComputedNode::default()
                },
            ))
            .id();
        let child = world
            .spawn((
                UiGlobalTransform::from_translation(Vec2::new(150.0, 95.0)),
                ComputedNode {
                    size: Vec2::new(40.0, 20.0),
                    content_size: Vec2::new(40.0, 20.0),
                    unrounded_size: Vec2::new(40.0, 20.0),
                    ..ComputedNode::default()
                },
                ChildOf(parent),
            ))
            .id();

        let mut layout_state = bevy_ecs::system::SystemState::<
            Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
        >::new(&mut world);
        let mut child_state = bevy_ecs::system::SystemState::<Query<&ChildOf>>::new(&mut world);
        let layout_query = layout_state
            .get(&world)
            .expect("layout query should resolve");
        let child_of_query = child_state
            .get(&world)
            .expect("child relationship query should resolve");

        let offsets = current_absolute_offsets_px(child, &layout_query, &child_of_query)
            .expect("offsets should resolve");

        assert_eq!(offsets, (130.0, 85.0));
    }
}
