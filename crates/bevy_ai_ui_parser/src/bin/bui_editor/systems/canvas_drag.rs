use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::picking::pointer::PointerInteraction;
use bevy::ui::ui_transform::UiGlobalTransform;
use bevy::ui::ComputedNode;
use bevy_ai_ui_parser::{BuiDocumentResource, BuiId, BuiNode};

use crate::app_state::EditorState;
use crate::undo::commands::MoveNode;

#[derive(Resource, Default)]
pub struct DragState {
    pub dragged_node_id: Option<String>,
    pub drag_origin_left: Option<String>,
    pub drag_origin_top: Option<String>,
}

pub fn handle_canvas_drag(
    mut drag_state: ResMut<DragState>,
    mut editor_state: ResMut<EditorState>,
    mut doc_resource: ResMut<BuiDocumentResource>,
    mouse_buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    pointers: Query<&PointerInteraction>,
    bui_nodes: Query<(Entity, &BuiId, &ComputedNode, &UiGlobalTransform)>,
    mut mouse_motion: MessageReader<MouseMotion>,
    windows: Query<&bevy::window::Window, With<bevy::window::PrimaryWindow>>,
) {
    let left_pressed = mouse_buttons.pressed(bevy::input::mouse::MouseButton::Left);
    let just_pressed = mouse_buttons.just_pressed(bevy::input::mouse::MouseButton::Left);
    let just_released = mouse_buttons.just_released(bevy::input::mouse::MouseButton::Left);

    if drag_state.dragged_node_id.is_none() {
        if !just_pressed || !left_pressed {
            return;
        }

        let mut best: Option<(String, f32)> = None;
        for interaction in pointers.iter() {
            for (entity, _) in interaction.as_slice() {
                let Ok((_, bui_id, computed, _)) = bui_nodes.get(*entity) else {
                    continue;
                };
                let area = computed.size().x * computed.size().y;
                if area <= 0.0 {
                    continue;
                }
                match best {
                    None => best = Some((bui_id.0.clone(), area)),
                    Some((_, best_area)) if area < best_area => {
                        best = Some((bui_id.0.clone(), area));
                    }
                    _ => {}
                }
            }
        }

        if let Some((node_id, _)) = best {
            let is_absolute = {
                let doc = &doc_resource.0;
                find_node_by_id(&doc.root, &node_id)
                    .and_then(|n| n.layout.styles.position_type.as_deref())
                    .map(|p| p == "absolute" || p == "Absolute" || p == "fixed")
                    .unwrap_or(false)
            };

            if is_absolute {
                let (old_left, old_top) = {
                    let doc = &doc_resource.0;
                    find_node_by_id(&doc.root, &node_id)
                        .map(|n| (n.layout.styles.left.clone(), n.layout.styles.top.clone()))
                        .unwrap_or((None, None))
                };

                drag_state.dragged_node_id = Some(node_id.clone());
                drag_state.drag_origin_left = old_left;
                drag_state.drag_origin_top = old_top;
                editor_state.selected_node_id = Some(node_id);
            } else {
                editor_state.selected_node_id = Some(node_id);
            }
        }
        return;
    }

    if just_released {
        if let Some(node_id) = drag_state.dragged_node_id.take() {
            let old_left = drag_state.drag_origin_left.take();
            let old_top = drag_state.drag_origin_top.take();

            let new_left;
            let new_top;
            {
                let doc = &doc_resource.0;
                let node = find_node_by_id(&doc.root, &node_id);
                if let Some(node) = node {
                    new_left = node.layout.styles.left.clone();
                    new_top = node.layout.styles.top.clone();
                } else {
                    return;
                }
            }

            if new_left != old_left || new_top != old_top {
                editor_state.undo_stack.push(Box::new(MoveNode {
                    node_id: node_id.clone(),
                    old_left,
                    old_top,
                    new_left,
                    new_top,
                }));
            }
        }
        return;
    }

    if drag_state.dragged_node_id.is_some() {
        let delta: bevy::math::Vec2 = mouse_motion.read().map(|e| e.delta).sum();
        if delta == bevy::math::Vec2::ZERO {
            return;
        }

        let scale = windows
            .single()
            .map(|w| w.scale_factor() as f32)
            .unwrap_or(1.0);

        let dx = delta.x / scale;
        let dy = delta.y / scale;

        let Some(node_id) = drag_state.dragged_node_id.as_ref() else {
            return;
        };
        let node_id = node_id.clone();
        let doc = &mut doc_resource.0;

        if let Some(node) = find_node_by_id_mut(&mut doc.root, &node_id) {
            let current_left = node
                .layout
                .styles
                .left
                .as_deref()
                .and_then(|v| parse_px(v))
                .unwrap_or(0.0);
            let current_top = node
                .layout
                .styles
                .top
                .as_deref()
                .and_then(|v| parse_px(v))
                .unwrap_or(0.0);

            node.layout.styles.left = Some(format!("{}px", current_left + dx));
            node.layout.styles.top = Some(format!("{}px", current_top + dy));
            node.layout.styles.right = None;
            node.layout.styles.bottom = None;
        }
    }
}

fn parse_px(s: &str) -> Option<f32> {
    s.trim()
        .trim_end_matches("px")
        .trim()
        .parse::<f32>()
        .ok()
}

fn find_node_by_id<'a>(root: &'a BuiNode, id: &str) -> Option<&'a BuiNode> {
    if root.id == id {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_node_by_id(child, id) {
            return Some(found);
        }
    }
    None
}

fn find_node_by_id_mut<'a>(root: &'a mut BuiNode, id: &str) -> Option<&'a mut BuiNode> {
    if root.id == id {
        return Some(root);
    }
    for child in &mut root.children {
        if let Some(found) = find_node_by_id_mut(child, id) {
            return Some(found);
        }
    }
    None
}
