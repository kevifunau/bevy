use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use bevy_ai_ui_parser::{BuiDocumentResource, BuiNode};

use crate::app_state::EditorState;
use crate::undo::commands::{AddNode, DeleteNode};

#[derive(Resource, Default)]
pub struct Clipboard {
    pub node: Option<BuiNode>,
}

pub fn handle_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<EditorState>,
    mut doc_resource: ResMut<BuiDocumentResource>,
    mut clipboard: ResMut<Clipboard>,
) {
    let ctrl = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        || keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    if ctrl {
        handle_ctrl_hotkeys(&keys, &mut editor_state, &mut doc_resource, &mut clipboard);
        return;
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        let Some(node_id) = editor_state.selected_node_id.clone() else {
            return;
        };
        if node_id == "root" {
            return;
        }

        let doc = &doc_resource.0;
        let parent_id = find_parent_id(&doc.root, &node_id);
        let Some(parent_id) = parent_id else { return };

        let (index, deleted_node) = {
            let Some(parent) = find_node_by_id(&doc.root, &parent_id) else {
                return;
            };
            let Some(idx) = parent.children.iter().position(|c| c.id == node_id) else {
                return;
            };
            (idx, parent.children[idx].clone())
        };

        editor_state.undo_stack.push(Box::new(DeleteNode {
            node_id: node_id.clone(),
            parent_id: parent_id.clone(),
            deleted_node,
            index,
        }));

        let mut doc = doc_resource.0.clone();
        if let Some(parent) = find_node_mut(&mut doc.root, &parent_id) {
            parent.children.retain(|c| c.id != node_id);
        }
        doc_resource.0 = doc;

        editor_state.selected_node_id = None;
        info!("Deleted node: {}", node_id);
        return;
    }

    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowUp) {
        let doc = &doc_resource.0;
        let flat = flatten_tree(&doc.root);
        if flat.is_empty() {
            return;
        }

        let new_sel = match &editor_state.selected_node_id {
            None => flat.first().cloned(),
            Some(id) => {
                let idx = flat.iter().position(|n| n == id);
                match idx {
                    None => flat.first().cloned(),
                    Some(i) => {
                        if keys.just_pressed(KeyCode::ArrowDown) {
                            flat.get(i + 1).cloned()
                        } else if i > 0 {
                            flat.get(i - 1).cloned()
                        } else {
                            None
                        }
                    }
                }
            }
        };

        if let Some(new_sel) = new_sel {
            editor_state.selected_node_id = Some(new_sel);
        }
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        if let Some(id) = &editor_state.selected_node_id {
            let doc = &doc_resource.0;
            if let Some(node) = find_node_by_id(&doc.root, id) {
                if !node.children.is_empty() {
                    editor_state.selected_node_id = Some(node.children[0].id.clone());
                }
            }
        }
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        if let Some(id) = &editor_state.selected_node_id {
            let doc = &doc_resource.0;
            if let Some(parent_id) = find_parent_id(&doc.root, id) {
                if parent_id != doc.root.id || id != &doc.root.id {
                    editor_state.selected_node_id = Some(parent_id);
                }
            }
        }
    }
}

fn handle_ctrl_hotkeys(
    keys: &ButtonInput<KeyCode>,
    editor_state: &mut EditorState,
    doc_resource: &mut BuiDocumentResource,
    clipboard: &mut Clipboard,
) {
    if keys.just_pressed(KeyCode::KeyS) {
        if let Err(e) = editor_state.save(&doc_resource.0) {
            error!("Save failed: {e}");
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        if shift {
            let mut doc = doc_resource.0.clone();
            if editor_state.undo_stack.redo(&mut doc) {
                doc_resource.0 = doc;
                info!("Redo");
            }
        } else {
            let mut doc = doc_resource.0.clone();
            if editor_state.undo_stack.undo(&mut doc) {
                doc_resource.0 = doc;
                info!("Undo");
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyY) {
        let mut doc = doc_resource.0.clone();
        if editor_state.undo_stack.redo(&mut doc) {
            doc_resource.0 = doc;
            info!("Redo");
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyC) {
        if let Some(node_id) = &editor_state.selected_node_id {
            let doc = &doc_resource.0;
            if let Some(node) = find_node_by_id(&doc.root, node_id) {
                clipboard.node = Some(node.clone());
                info!("Copied node: {}", node_id);
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyV) {
        let Some(node_id) = editor_state.selected_node_id.clone() else {
            return;
        };
        let Some(src_node) = clipboard.node.clone() else {
            return;
        };

        let parent_id = {
            let doc = &doc_resource.0;
            find_parent_id(&doc.root, &node_id)
        };
        let Some(parent_id) = parent_id else { return };

        let new_id = format!("{}_copy_{}", src_node.id, timestamp());
        let mut new_node = src_node.clone();
        new_node.id = new_id.clone();

        editor_state.undo_stack.push(Box::new(AddNode {
            parent_id: parent_id.clone(),
            node: new_node.clone(),
        }));

        let mut doc = doc_resource.0.clone();
        if let Some(parent) = find_node_mut(&mut doc.root, &parent_id) {
            parent.children.push(new_node);
        }
        doc_resource.0 = doc;

        editor_state.selected_node_id = Some(new_id);
        info!("Pasted node");
    }
}

fn flatten_tree(node: &BuiNode) -> Vec<String> {
    let mut result = vec![node.id.clone()];
    for child in &node.children {
        result.extend(flatten_tree(child));
    }
    result
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

fn find_node_mut<'a>(root: &'a mut BuiNode, id: &str) -> Option<&'a mut BuiNode> {
    if root.id == id {
        return Some(root);
    }
    for child in &mut root.children {
        if let Some(found) = find_node_mut(child, id) {
            return Some(found);
        }
    }
    None
}

fn find_parent_id(root: &BuiNode, node_id: &str) -> Option<String> {
    for child in &root.children {
        if child.id == node_id {
            return Some(root.id.clone());
        }
        if let Some(found) = find_parent_id(child, node_id) {
            return Some(found);
        }
    }
    None
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_millis() as u64
}
