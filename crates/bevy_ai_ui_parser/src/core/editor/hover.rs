use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_math::Vec2;
use bevy_picking::hover::HoverMap;
use bevy_text::{FontSize, FontSource, TextColor, TextFont};
use bevy_ui::prelude::*;
use bevy_ui::ComputedNode;

use crate::core::editor::state::{BuiEditorBorder, BuiEditorCloseIcon, BuiEditorState, EditorMode};
use crate::core::runtime::components::BuiRootEntity;

pub(crate) fn editor_hover_system(
    mut editor_state: ResMut<BuiEditorState>,
    hover_map: Res<HoverMap>,
    border_query: Query<(Entity, &BuiEditorBorder), With<BuiEditorBorder>>,
    target_query: Query<Option<&ComputedNode>>,
    root_entity: Option<Res<BuiRootEntity>>,
    target_camera_query: Query<&UiTargetCamera>,
    mut commands: Commands,
) {
    if editor_state.mode != EditorMode::Active {
        editor_state.hovered_node_id = None;
        editor_state.close_icon_entity = None;
        return;
    }

    let mut hovered_border: Option<(Entity, &BuiEditorBorder, f32)> = None;

    for (entity, border) in &border_query {
        for (_, hovered_entities) in hover_map.iter() {
            if hovered_entities.contains_key(&entity) {
                let area = border_target_area(border.target_entity, &target_query);
                let should_replace = hovered_border
                    .map(|(_, current_border, current_area)| {
                        area < current_area
                            || (approx_eq(area, current_area)
                                && border.node_id.len() > current_border.node_id.len())
                    })
                    .unwrap_or(true);
                if should_replace {
                    hovered_border = Some((entity, border, area));
                }
            }
        }
    }

    let new_hovered_id = hovered_border.map(|(_, b, _)| b.node_id.clone());

    if new_hovered_id == editor_state.hovered_node_id {
        return;
    }

    if let Some(existing) = editor_state.close_icon_entity {
        commands.entity(existing).despawn();
        editor_state.close_icon_entity = None;
    }

    editor_state.hovered_node_id = new_hovered_id;

    if let Some((border_entity, border, _)) = hovered_border {
        let inherited_target_camera =
            resolve_ui_target_camera(root_entity.as_deref(), &target_camera_query);
        let close_icon = commands
            .spawn((
                Text::new("×"),
                TextFont {
                    font: FontSource::default(),
                    font_size: FontSize::Px(14.0),
                    ..TextFont::default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    right: Val::Px(-2.0),
                    top: Val::Px(-18.0),
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Node::default()
                },
                BackgroundColor(Color::srgba(0.75, 0.1, 0.1, 0.95)),
                BuiEditorCloseIcon {
                    node_id: border.node_id.clone(),
                },
            ))
            .id();
        if let Some(target_camera) = inherited_target_camera {
            commands.entity(close_icon).insert(target_camera);
        }
        commands.entity(border_entity).add_child(close_icon);

        editor_state.close_icon_entity = Some(close_icon);
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

fn border_target_area(target_entity: Entity, target_query: &Query<Option<&ComputedNode>>) -> f32 {
    target_query
        .get(target_entity)
        .ok()
        .flatten()
        .map(ComputedNode::size)
        .unwrap_or(Vec2::splat(10_000.0))
        .max(Vec2::ZERO)
        .element_product()
}

fn approx_eq(left: f32, right: f32) -> bool {
    (left - right).abs() <= f32::EPSILON
}

#[cfg(test)]
mod tests {
    use bevy_math::Vec2;

    use super::approx_eq;

    #[test]
    fn approx_eq_treats_identical_areas_as_equal() {
        assert!(approx_eq(100.0, 100.0));
        assert!(!approx_eq(100.0, 101.0));
    }

    #[test]
    fn vec2_area_helper_behavior_is_stable() {
        let size = Vec2::new(32.0, 24.0);
        assert_eq!(size.element_product(), 768.0);
    }
}
