use bevy::prelude::*;
use bevy_ai_ui_parser::{BuiDocumentResource, BuiNode};
use bevy_egui::egui;

use super::find_node_by_id;
use crate::undo::commands::*;

pub fn inspector_panel(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_node_id: &Option<String>,
) {
    ui.heading("Inspector");

    let Some(doc) = world.get_resource::<BuiDocumentResource>() else {
        ui.colored_label(egui::Color32::RED, "No document loaded");
        return;
    };

    let Some(node_id) = selected_node_id.clone() else {
        ui.label("Select a node from the Hierarchy panel or click in the canvas");
        return;
    };

    ui.label(format!("Selected: {}", node_id));
    ui.add_space(4.0);

    let mut doc_clone = doc.0.clone();
    let Some(node) = find_node_by_id_mut(&mut doc_clone.root, &node_id) else {
        ui.colored_label(egui::Color32::RED, format!("Node not found: {node_id}"));
        return;
    };

    let mut commands_to_push: Vec<crate::undo::commands::SetStyleField> = Vec::new();
    let id_seed = ui.next_auto_id();
    let node_id_for_cmds = node_id.clone();

    egui::Grid::new(id_seed.with("node_info"))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("ID");
            ui.label(&node.id);
            ui.end_row();

            ui.label("Kind");
            ui.label(&node.kind);
            ui.end_row();

            ui.label("Classes");
            ui.label(node.classes.join(", "));
            ui.end_row();

            ui.label("Markers");
            ui.label(node.markers.join(", "));
            ui.end_row();

            ui.label("Children");
            ui.label(format!("{}", node.children.len()));
            ui.end_row();
        });

    ui.add_space(8.0);

    let mut text_changed = false;

    render_layout_editor(ui, &mut node.layout.styles, id_seed, &mut commands_to_push, &node_id_for_cmds);
    render_visuals_editor(ui, &mut node.style.visuals, id_seed, &mut commands_to_push, &node_id_for_cmds);

    if let Some(text) = &mut node.content.text {
        ui.add_space(8.0);
        ui.heading("Text Content");
        text_changed |= render_text_editor(ui, text, id_seed);
    }

    if let Some(image) = &mut node.content.image {
        ui.add_space(8.0);
        ui.heading("Image Content");
        egui::Grid::new(id_seed.with("image"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("texture_path");
                ui.label(&image.texture_path);
                ui.end_row();
            });
    }

    let changed = !commands_to_push.is_empty() || text_changed;
    if changed {
        let mut doc_resource = world.resource_mut::<BuiDocumentResource>();
        doc_resource.0 = doc_clone;

        let mut editor_state = world.resource_mut::<crate::app_state::EditorState>();
        for cmd in commands_to_push {
            editor_state.undo_stack.push(Box::new(cmd));
        }
        editor_state.dirty = true;
    }
}

fn render_layout_editor(
    ui: &mut egui::Ui,
    styles: &mut bevy_ai_ui_parser::BuiStyles,
    id_seed: egui::Id,
    commands: &mut Vec<SetStyleField>,
    node_id: &str,
) {
    ui.heading("Layout Styles");

    ui.collapsing("Box Model", |ui| {
        egui::Grid::new(id_seed.with("layout_box"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                track_edit(ui, "display", &mut styles.display, commands, node_id);
                track_edit(ui, "position", &mut styles.position_type, commands, node_id);
                track_edit(ui, "width", &mut styles.width, commands, node_id);
                track_edit(ui, "height", &mut styles.height, commands, node_id);
                track_edit(ui, "min_width", &mut styles.min_width, commands, node_id);
                track_edit(ui, "min_height", &mut styles.min_height, commands, node_id);
                track_edit(ui, "max_width", &mut styles.max_width, commands, node_id);
                track_edit(ui, "max_height", &mut styles.max_height, commands, node_id);
                track_edit(ui, "box_sizing", &mut styles.box_sizing, commands, node_id);
                track_edit(ui, "aspect_ratio", &mut styles.aspect_ratio, commands, node_id);
            });
    });

    ui.collapsing("Positioning", |ui| {
        egui::Grid::new(id_seed.with("layout_pos"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                track_edit(ui, "left", &mut styles.left, commands, node_id);
                track_edit(ui, "top", &mut styles.top, commands, node_id);
                track_edit(ui, "right", &mut styles.right, commands, node_id);
                track_edit(ui, "bottom", &mut styles.bottom, commands, node_id);
                track_edit(ui, "z_index", &mut styles.z_index, commands, node_id);
                track_edit(ui, "global_z_index", &mut styles.global_z_index, commands, node_id);
            });
    });

    ui.collapsing("Flexbox", |ui| {
        egui::Grid::new(id_seed.with("layout_flex"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                track_edit(ui, "flex_direction", &mut styles.flex_direction, commands, node_id);
                track_edit(ui, "flex_wrap", &mut styles.flex_wrap, commands, node_id);
                track_edit(ui, "flex_grow", &mut styles.flex_grow, commands, node_id);
                track_edit(ui, "flex_shrink", &mut styles.flex_shrink, commands, node_id);
                track_edit(ui, "flex_basis", &mut styles.flex_basis, commands, node_id);
                track_edit(ui, "justify_content", &mut styles.justify_content, commands, node_id);
                track_edit(ui, "align_items", &mut styles.align_items, commands, node_id);
                track_edit(ui, "align_self", &mut styles.align_self, commands, node_id);
                track_edit(ui, "row_gap", &mut styles.row_gap, commands, node_id);
                track_edit(ui, "column_gap", &mut styles.column_gap, commands, node_id);
            });
    });

    ui.collapsing("Grid", |ui| {
        egui::Grid::new(id_seed.with("layout_grid"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                track_edit(ui, "grid_template_columns", &mut styles.grid_template_columns, commands, node_id);
                track_edit(ui, "grid_template_rows", &mut styles.grid_template_rows, commands, node_id);
                track_edit(ui, "grid_auto_columns", &mut styles.grid_auto_columns, commands, node_id);
                track_edit(ui, "grid_auto_rows", &mut styles.grid_auto_rows, commands, node_id);
                track_edit(ui, "grid_column", &mut styles.grid_column, commands, node_id);
                track_edit(ui, "grid_row", &mut styles.grid_row, commands, node_id);
            });
    });

    ui.collapsing("Spacing", |ui| {
        egui::Grid::new(id_seed.with("layout_spacing"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                track_edit(ui, "margin", &mut styles.margin, commands, node_id);
                track_edit(ui, "margin_top", &mut styles.margin_top, commands, node_id);
                track_edit(ui, "margin_bottom", &mut styles.margin_bottom, commands, node_id);
                track_edit(ui, "margin_left", &mut styles.margin_left, commands, node_id);
                track_edit(ui, "margin_right", &mut styles.margin_right, commands, node_id);
                track_edit(ui, "padding", &mut styles.padding, commands, node_id);
                track_edit(ui, "padding_top", &mut styles.padding_top, commands, node_id);
                track_edit(ui, "padding_bottom", &mut styles.padding_bottom, commands, node_id);
                track_edit(ui, "padding_left", &mut styles.padding_left, commands, node_id);
                track_edit(ui, "padding_right", &mut styles.padding_right, commands, node_id);
            });
    });

    ui.collapsing("Other", |ui| {
        egui::Grid::new(id_seed.with("layout_other"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                track_edit(ui, "overflow", &mut styles.overflow, commands, node_id);
                track_edit(ui, "visibility", &mut styles.visibility, commands, node_id);
            });
    });
}

fn render_visuals_editor(
    ui: &mut egui::Ui,
    visuals: &mut bevy_ai_ui_parser::BuiVisuals,
    id_seed: egui::Id,
    commands: &mut Vec<SetStyleField>,
    node_id: &str,
) {
    ui.add_space(4.0);
    ui.heading("Visual Styles");

    egui::Grid::new(id_seed.with("visuals"))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            track_edit(ui, "background_color", &mut visuals.background_color, commands, node_id);
            track_edit(ui, "border_color", &mut visuals.border_color, commands, node_id);
            track_edit(ui, "border_width", &mut visuals.border_width, commands, node_id);
            track_edit(ui, "border_radius", &mut visuals.border_radius, commands, node_id);
        });
}

fn render_text_editor(
    ui: &mut egui::Ui,
    text: &mut bevy_ai_ui_parser::BuiTextConfig,
    id_seed: egui::Id,
) -> bool {
    let mut changed = false;

    egui::Grid::new(id_seed.with("text"))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("content");
            let response = ui.text_edit_multiline(&mut text.content);
            if response.changed() {
                changed = true;
            }
            ui.end_row();

            ui.label("font_size");
            let mut font_size = text.font_size;
            let response = ui.add(egui::DragValue::new(&mut font_size).range(1.0..=200.0));
            if response.changed() {
                text.font_size = font_size;
                changed = true;
            }
            ui.end_row();

            ui.label("font_color");
            let mut color = text.font_color.clone();
            let response = ui.text_edit_singleline(&mut color);
            if response.changed() {
                text.font_color = color;
                changed = true;
            }
            ui.end_row();
        });

    changed
}

fn track_edit(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut Option<String>,
    commands: &mut Vec<SetStyleField>,
    node_id: &str,
) {
    ui.label(name);
    let old_value = value.clone();
    let mut text = value.clone().unwrap_or_default();
    let response = ui.text_edit_singleline(&mut text);
    if response.changed() {
        let new_value = if text.is_empty() { None } else { Some(text) };
        *value = new_value.clone();
        commands.push(SetStyleField {
            node_id: node_id.to_string(),
            field_name: name.to_string(),
            old_value,
            new_value,
        });
    }
    ui.end_row();
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
