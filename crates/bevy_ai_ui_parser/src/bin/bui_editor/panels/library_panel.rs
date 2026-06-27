use bevy::prelude::*;
use bevy_egui::egui;

use crate::app_state::LibraryItem;

/// Library panel. Lists draggable node templates.
pub fn library_panel(
    ui: &mut egui::Ui,
    world: &mut World,
    dragging_library_item: &mut Option<LibraryItem>,
    _drag_hover_node_id: &mut Option<String>,
) {
    ui.heading("Library");
    ui.label("Drag to Hierarchy to add");
    ui.add_space(8.0);

    let basic_items = [
        LibraryItem::Node,
        LibraryItem::Text,
        LibraryItem::Button,
        LibraryItem::Image,
        LibraryItem::TextInput,
        LibraryItem::Toggle,
        LibraryItem::Slider,
    ];

    let template_items = [
        LibraryItem::Row,
        LibraryItem::Column,
        LibraryItem::ButtonWithText,
    ];

    ui.label("Components:");
    for item in &basic_items {
        render_library_item(ui, world, item, dragging_library_item);
    }

    ui.add_space(8.0);
    ui.label("Templates:");
    for item in &template_items {
        render_library_item(ui, world, item, dragging_library_item);
    }

    // Clear drag if mouse released outside hierarchy
    let released = ui.input(|i| !i.pointer.primary_down());
    if released && dragging_library_item.is_some() {
        // Only clear if not over hierarchy (hierarchy handles its own drop)
        // The hierarchy panel will clear this if it processes the drop
        // If we get here and it's still set, the drop was outside hierarchy
        // We give hierarchy one frame to process first
    }
}

fn render_library_item(
    ui: &mut egui::Ui,
    world: &mut World,
    item: &LibraryItem,
    dragging_library_item: &mut Option<LibraryItem>,
) {
    let label = format!("{} {}", item.icon(), item.label());

    let is_being_dragged = dragging_library_item.as_ref() == Some(item);

    let response = ui.selectable_label(is_being_dragged, &label);

    if response.drag_started() {
        *dragging_library_item = Some(item.clone());
    }

    if is_being_dragged {
        ui.painter().rect_filled(
            response.rect,
            2.0,
            egui::Color32::from_rgba_premultiplied(40, 120, 200, 80),
        );
    }

    let _ = world;
}
