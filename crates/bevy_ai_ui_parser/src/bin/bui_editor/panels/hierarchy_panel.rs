use std::collections::HashSet;

use bevy::prelude::*;
use bevy_ai_ui_parser::{BuiDocumentResource, BuiNode};
use bevy_egui::egui;

use bevy_ai_ui_parser::BuiNode as DocNode;
use crate::app_state::LibraryItem;
use crate::undo::commands::{AddNode, DeleteNode};

pub fn hierarchy_panel(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_node_id: &mut Option<String>,
    collapsed_nodes: &mut HashSet<String>,
    dragging_library_item: &mut Option<LibraryItem>,
    drag_hover_node_id: &mut Option<String>,
) {
    ui.heading("Hierarchy");

    let root_id = {
        let Some(doc) = world.get_resource::<BuiDocumentResource>() else {
            ui.colored_label(egui::Color32::RED, "No document loaded");
            return;
        };
        doc.0.root.id.clone()
    };

    let is_dragging = dragging_library_item.is_some();

    egui::ScrollArea::vertical()
        .id_salt("hierarchy_scroll")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            render_node_tree(
                ui,
                world,
                &root_id,
                None,
                0,
                selected_node_id,
                collapsed_nodes,
                is_dragging,
                dragging_library_item,
                drag_hover_node_id,
            );
        });

    // Check if drag was released over empty area
    if is_dragging {
        let released = ui.input(|i| !i.pointer.primary_down());
        if released {
            let hover_pos = ui.input(|i| i.pointer.hover_pos());
            if let Some(pos) = hover_pos {
                if ui.clip_rect().contains(pos) && drag_hover_node_id.is_none() {
                    if let Some(item) = dragging_library_item.take() {
                        add_library_node(world, &root_id, &item, selected_node_id, collapsed_nodes);
                    }
                }
            }
            *dragging_library_item = None;
            *drag_hover_node_id = None;
        }
    }

    ui.add_space(8.0);
    ui.label(format!(
        "Selected: {}",
        selected_node_id.as_deref().unwrap_or("(none)")
    ));

    ui.add_space(4.0);
    ui.menu_button("+ Add", |ui| {
        if ui.button("Add child node to root").clicked() {
            add_child_node(world, &root_id, selected_node_id);
            ui.close();
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_node_tree(
    ui: &mut egui::Ui,
    world: &mut World,
    node_id: &str,
    parent_id: Option<&str>,
    depth: usize,
    selected_id: &mut Option<String>,
    collapsed_nodes: &mut HashSet<String>,
    is_dragging: bool,
    dragging_library_item: &mut Option<LibraryItem>,
    drag_hover_node_id: &mut Option<String>,
) {
    let (id, kind, child_ids) = {
        let doc = world.resource::<BuiDocumentResource>();
        let Some(node) = find_node_by_id(&doc.0.root, node_id) else {
            return;
        };
        (
            node.id.clone(),
            node.kind.clone(),
            node.children.iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
        )
    };

    let has_children = !child_ids.is_empty();
    let collapsed = collapsed_nodes.contains(&id);

    let indicator = if has_children {
        if collapsed { "\u{25B8} " } else { "\u{25BE} " }
    } else {
        "  "
    };
    let label = format!("{}{} [{}]", indicator, id, kind);

    let is_selected = selected_id.as_deref() == Some(&id);
    let response = ui.selectable_label(is_selected, &label);

    // Drop target highlight
    let is_drop_target = is_dragging && {
        let hover_pos = response.hover_pos();
        hover_pos.map(|p| response.rect.contains(p)).unwrap_or(false)
    };

    if is_drop_target {
        let bg = egui::Color32::from_rgba_premultiplied(40, 120, 200, 80);
        ui.painter().rect_filled(response.rect, 2.0, bg);
        *drag_hover_node_id = Some(id.clone());
    } else if is_dragging && drag_hover_node_id.as_deref() == Some(&id) {
        // Clear if no longer hovering
        let hover_pos = response.hover_pos();
        if !hover_pos.map(|p| response.rect.contains(p)).unwrap_or(false) {
            *drag_hover_node_id = None;
        }
    }

    if response.clicked() {
        *selected_id = Some(id.clone());
    }

    if has_children && response.double_clicked() {
        if collapsed {
            collapsed_nodes.remove(&id);
        } else {
            collapsed_nodes.insert(id.clone());
        }
    }

    // Check for drop on this node
    if is_dragging && is_drop_target {
        let released = ui.input(|i| !i.pointer.primary_down());
        if released {
            if let Some(item) = dragging_library_item.take() {
                add_library_node(world, &id, &item, selected_id, collapsed_nodes);
            }
            *drag_hover_node_id = None;
        }
    }

    response.context_menu(|ui| {
        if ui.button("Select").clicked() {
            *selected_id = Some(id.clone());
            ui.close();
        }
        if has_children {
            if ui.button(if collapsed { "Expand" } else { "Collapse" }).clicked() {
                if collapsed {
                    collapsed_nodes.remove(&id);
                } else {
                    collapsed_nodes.insert(id.clone());
                }
                ui.close();
            }
            ui.separator();
        }
        if ui.button("Add child node").clicked() {
            add_child_node(world, &id, selected_id);
            ui.close();
        }
        if let Some(pid) = parent_id {
            if id != "root" && ui.button("Delete node").clicked() {
                delete_node(world, &id, pid, selected_id);
                ui.close();
            }
        }
    });

    if has_children && !collapsed {
        ui.indent(format!("indent_{}_{}", id, depth), |ui| {
            for child_id in &child_ids {
                render_node_tree(
                    ui,
                    world,
                    child_id,
                    Some(&id),
                    depth + 1,
                    selected_id,
                    collapsed_nodes,
                    is_dragging,
                    dragging_library_item,
                    drag_hover_node_id,
                );
            }
        });
    }
}

fn add_library_node(
    world: &mut World,
    parent_id: &str,
    item: &LibraryItem,
    selected_id: &mut Option<String>,
    collapsed_nodes: &mut HashSet<String>,
) {
    let new_node = crate::node_factory::create_library_node(item);
    let new_id = new_node.id.clone();

    let mut editor_state = world.resource_mut::<crate::app_state::EditorState>();
    editor_state.undo_stack.push(Box::new(AddNode {
        parent_id: parent_id.to_string(),
        node: new_node.clone(),
    }));

    let mut doc_resource = world.resource_mut::<BuiDocumentResource>();
    let mut doc = doc_resource.0.clone();
    if let Some(parent) = find_node_by_id_mut(&mut doc.root, parent_id) {
        parent.children.push(new_node);
    }
    doc_resource.0 = doc;

    // Auto-expand parent
    collapsed_nodes.remove(parent_id);

    *selected_id = Some(new_id);
    info!("Added {} to {}", item.label(), parent_id);
}

fn add_child_node(world: &mut World, parent_id: &str, selected_id: &mut Option<String>) {
    let new_id = format!("node_{}", timestamp());
    let new_node = BuiNode {
        id: new_id.clone(),
        kind: "node".to_string(),
        markers: vec![],
        classes: vec![],
        actions: vec![],
        bindings: vec![],
        layout: Default::default(),
        style: Default::default(),
        content: Default::default(),
        semantics: Default::default(),
        state_visuals: Default::default(),
        children: vec![],
    };

    let mut editor_state = world.resource_mut::<crate::app_state::EditorState>();
    editor_state.undo_stack.push(Box::new(AddNode {
        parent_id: parent_id.to_string(),
        node: new_node.clone(),
    }));

    let mut doc_resource = world.resource_mut::<BuiDocumentResource>();
    let mut doc = doc_resource.0.clone();
    if let Some(parent) = find_node_by_id_mut(&mut doc.root, parent_id) {
        parent.children.push(new_node);
    }
    doc_resource.0 = doc;

    *selected_id = Some(new_id);
}

fn delete_node(world: &mut World, node_id: &str, parent_id: &str, selected_id: &mut Option<String>) {
    let (index, deleted_node) = {
        let doc = world.resource::<BuiDocumentResource>();
        let Some(parent) = find_node_by_id(&doc.0.root, parent_id) else { return };
        let Some(idx) = parent.children.iter().position(|c| c.id == node_id) else { return };
        (idx, parent.children[idx].clone())
    };

    let mut editor_state = world.resource_mut::<crate::app_state::EditorState>();
    editor_state.undo_stack.push(Box::new(DeleteNode {
        node_id: node_id.to_string(),
        parent_id: parent_id.to_string(),
        deleted_node,
        index,
    }));

    let mut doc_resource = world.resource_mut::<BuiDocumentResource>();
    let mut doc = doc_resource.0.clone();
    if let Some(parent) = find_node_by_id_mut(&mut doc.root, parent_id) {
        parent.children.retain(|c| c.id != node_id);
    }
    doc_resource.0 = doc;

    *selected_id = None;
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn find_node_by_id<'a>(root: &'a DocNode, id: &str) -> Option<&'a DocNode> {
    if root.id == id { return Some(root) }
    for child in &root.children {
        if let Some(found) = find_node_by_id(child, id) { return Some(found) }
    }
    None
}

fn find_node_by_id_mut<'a>(root: &'a mut DocNode, id: &str) -> Option<&'a mut DocNode> {
    if root.id == id { return Some(root) }
    for child in &mut root.children {
        if let Some(found) = find_node_by_id_mut(child, id) { return Some(found) }
    }
    None
}
