use bevy::prelude::*;
use bevy_ai_ui_parser::{BuiDocumentResource, BuiNode};
use bevy_egui::egui;

use super::find_node_by_id;

pub fn style_editor_panel(ui: &mut egui::Ui, _world: &World, _selected_node_id: &Option<String>) {
    ui.heading("Style Editor");
    ui.label("CSS field quick-edit");
    ui.add_space(4.0);

    ui.label(
        "\
        This panel mirrors the Inspector's layout/visuals sections. \
        In Phase 3 it will gain undo/redo, color pickers, and \
        drag-to-adjust numeric fields.",
    );
    ui.add_space(8.0);

    ui.collapsing("CSS Quick Reference", |ui| {
        ui.label("Layout values:");
        ui.label("  display: flex | grid | block | none");
        ui.label("  position: relative | absolute | fixed");
        ui.label("  width/height: 100px | 50% | auto | 100% 100%");
        ui.label("  flex-direction: row | column | row-reverse");
        ui.label("  justify-content: flex-start | center | space-between");
        ui.label("  align-items: flex-start | center | stretch");
        ui.label("");
        ui.label("Color values:");
        ui.label("  #RGB | #RRGGBB | #RRGGBBAA");
        ui.label("  rgb(255,0,0) | rgba(255,0,0,0.5)");
        ui.label("  transparent");
    });
}
