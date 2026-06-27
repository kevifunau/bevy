use bevy_egui::egui;

pub fn console_panel(ui: &mut egui::Ui) {
    ui.heading("Console");
    ui.label("BUI Editor - Standalone IR JSON Editor");
    ui.add_space(8.0);

    ui.heading("Shortcuts");
    egui::Grid::new("shortcuts_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Ctrl/Cmd+S");
            ui.label("Save IR JSON");
            ui.end_row();

            ui.label("Ctrl/Cmd+Z");
            ui.label("Undo");
            ui.end_row();

            ui.label("Ctrl/Cmd+Shift+Z / Ctrl+Y");
            ui.label("Redo");
            ui.end_row();

            ui.label("Ctrl/Cmd+C");
            ui.label("Copy selected node");
            ui.end_row();

            ui.label("Ctrl/Cmd+V");
            ui.label("Paste as sibling");
            ui.end_row();

            ui.label("Delete / Backspace");
            ui.label("Delete selected node");
            ui.end_row();

            ui.label("Arrow Up/Down");
            ui.label("Navigate between siblings");
            ui.end_row();

            ui.label("Arrow Right");
            ui.label("Select first child");
            ui.end_row();

            ui.label("Arrow Left");
            ui.label("Select parent");
            ui.end_row();
        });

    ui.add_space(8.0);

    ui.heading("Canvas");
    ui.label("Click a BUI element to select it");
    ui.label("Drag absolute-positioned nodes to move them");
    ui.label("Selected node shows green border");

    ui.add_space(8.0);

    ui.heading("Hierarchy");
    ui.label("Right-click for context menu (Add/Delete)");
    ui.label("Click + Add button to add child to root");

    ui.add_space(8.0);

    ui.heading("Inspector");
    ui.label("Edit CSS fields directly (Box Model, Flexbox, Grid, etc.)");
    ui.label("Changes are pushed to undo stack");
}
