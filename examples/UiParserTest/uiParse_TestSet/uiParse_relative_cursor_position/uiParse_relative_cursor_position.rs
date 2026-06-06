//! Renders the official `examples/ui/relative_cursor_position.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_relative_cursor_position`

use std::path::PathBuf;

use bevy::{camera::Viewport, prelude::*, ui::RelativeCursorPosition};
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "relative_cursor_position.ir.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, relative_cursor_position_system);
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_relative_cursor_position")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_position: [200, 100].into(),
                physical_size: [600, 600].into(),
                ..default()
            }),
            ..default()
        },
    ));
}

fn relative_cursor_position_system(
    trackers: Query<(&BuiId, &RelativeCursorPosition)>,
    mut outputs: Query<(&BuiId, &mut Text, &mut TextColor)>,
) {
    let mut normalized = None;
    let mut over = false;

    for (id, tracker) in &trackers {
        if id.0 == "cursor_box" {
            normalized = tracker.normalized;
            over = tracker.cursor_over();
            break;
        }
    }

    for (id, mut text, mut text_color) in &mut outputs {
        if id.0 != "cursor_output" {
            continue;
        }

        text.0 = if let Some(relative) = normalized {
            format!("({:.1}, {:.1})", relative.x, relative.y)
        } else {
            "unknown".to_string()
        };

        text_color.0 = if over {
            Color::srgb(0.1, 0.9, 0.1)
        } else {
            Color::srgb(0.9, 0.1, 0.1)
        };
    }
}
