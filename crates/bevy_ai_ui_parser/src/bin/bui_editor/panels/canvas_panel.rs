use bevy::prelude::*;
use bevy_ai_ui_parser::BuiDocumentResource;
use bevy_egui::egui;

/// Canvas panel. Returns `Some((zoom, pan))` if the user requested a zoom/pan change.
pub fn canvas_panel(
    ui: &mut egui::Ui,
    world: &World,
    zoom: f32,
    pan: egui::Vec2,
) -> Option<(f32, egui::Vec2)> {
    let clip_rect = ui.clip_rect();
    let mut new_zoom = zoom;
    let mut new_pan = pan;

    // Read zoom and scroll from egui context.
    let (zoom_delta, scroll) = ui.ctx().input(|i| {
        let hover_pos = i.pointer.hover_pos();
        let cursor_in_canvas = hover_pos
            .map(|pos| clip_rect.contains(pos))
            .unwrap_or(false);

        if !cursor_in_canvas {
            return (1.0, egui::Vec2::ZERO);
        }

        let ctrl = i.modifiers.ctrl || i.modifiers.command;
        let zd = i.zoom_delta();
        let sd = i.smooth_scroll_delta();

        if ctrl {
            (zd, egui::Vec2::ZERO)
        } else {
            (1.0, sd)
        }
    });

    if zoom_delta != 1.0 {
        new_zoom = (zoom * zoom_delta).clamp(0.2, 3.0);
    }
    if scroll != egui::Vec2::ZERO {
        new_pan.x += scroll.x;
        new_pan.y += scroll.y;
    }

    let result = if new_zoom != zoom || new_pan != pan {
        Some((new_zoom, new_pan))
    } else {
        None
    };

    // Simple toolbar
    let mut reset_clicked = false;
    let mut zoom_out_clicked = false;
    let mut zoom_in_clicked = false;

    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            zoom_out_clicked = true;
        }
        ui.label(format!("{:.0}%", new_zoom * 100.0));
        if ui.button("+").clicked() {
            zoom_in_clicked = true;
        }
        if new_zoom != 1.0 {
            if ui.button("100%").clicked() {
                reset_clicked = true;
            }
        }
        ui.separator();
        ui.colored_label(
            egui::Color32::DARK_GRAY,
            "Ctrl+Scroll: \u{7F29}\u{653E} | Scroll: \u{5E73}\u{79FB}",
        );
    });

    ui.separator();

    let doc = world.get_resource::<BuiDocumentResource>();
    if let Some(doc) = doc {
        ui.label(format!(
            "{} | {} [{}] | v{}",
            doc.0.scene_name, doc.0.root.id, doc.0.root.kind, doc.0.version
        ));
    }

    if reset_clicked {
        return Some((1.0, egui::Vec2::ZERO));
    }
    if zoom_out_clicked {
        return Some(((new_zoom * 0.8).max(0.2), new_pan));
    }
    if zoom_in_clicked {
        return Some(((new_zoom * 1.25).min(3.0), new_pan));
    }

    result
}
