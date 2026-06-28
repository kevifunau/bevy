use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::picking::pointer::{PointerAction, PointerInput, PointerInteraction};
use bevy::prelude::*;
use bevy::ui::ui_transform::UiGlobalTransform;
use bevy::ui::ComputedNode;

use bevy_ai_ui_parser::BuiId;

use crate::app_state::EditorState;

pub fn handle_canvas_click(
    mut editor_state: ResMut<EditorState>,
    mut click_events: MessageReader<PointerInput>,
    pointers: Query<&PointerInteraction>,
    bui_nodes: Query<(Entity, &BuiId, &ComputedNode, &UiGlobalTransform)>,
) {
    for event in click_events.read() {
        if !matches!(
            event.action,
            PointerAction::Press(bevy::picking::pointer::PointerButton::Primary)
        ) {
            continue;
        }

        let mut best_entity: Option<(Entity, f32)> = None;

        for interaction in pointers.iter() {
            for (entity, hit) in interaction.as_slice() {
                let Ok((_, _, computed, _)) = bui_nodes.get(*entity) else {
                    continue;
                };
                let area = computed.size().x * computed.size().y;
                if area <= 0.0 {
                    continue;
                }
                match best_entity {
                    None => best_entity = Some((*entity, area)),
                    Some((_, best_area)) => {
                        if area < best_area {
                            best_entity = Some((*entity, area));
                        }
                    }
                }
            }
        }

        if let Some((entity, _)) = best_entity {
            if let Ok((_, bui_id, _, _)) = bui_nodes.get(entity) {
                editor_state.selected_node_id = Some(bui_id.0.clone());
            }
        }
    }
}
