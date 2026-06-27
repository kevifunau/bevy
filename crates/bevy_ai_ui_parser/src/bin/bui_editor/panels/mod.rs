mod canvas_panel;
mod console_panel;
mod hierarchy_panel;
mod inspector_panel;
mod library_panel;
mod style_editor_panel;

use bevy::prelude::*;
use bevy_ai_ui_parser::BuiNode;
use bevy_egui::egui;

pub use canvas_panel::canvas_panel;
pub use console_panel::console_panel;
pub use hierarchy_panel::hierarchy_panel;
pub use inspector_panel::inspector_panel;
pub use library_panel::library_panel;
pub use style_editor_panel::style_editor_panel;

fn render_bui_node_tree(
    ui: &mut egui::Ui,
    node: &BuiNode,
    depth: usize,
    selected_id: &mut Option<String>,
) {
    let label = if node.children.is_empty() {
        format!("{} [{}]", node.id, node.kind)
    } else {
        format!("{} [{}] ({})", node.id, node.kind, node.children.len())
    };

    let is_selected = selected_id.as_deref() == Some(&node.id);
    let response = ui.selectable_label(is_selected, &label);
    if response.clicked() {
        *selected_id = Some(node.id.clone());
    }

    if !node.children.is_empty() {
        ui.indent(format!("indent_{}_{}", node.id, depth), |ui| {
            for child in &node.children {
                render_bui_node_tree(ui, child, depth + 1, selected_id);
            }
        });
    }
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
