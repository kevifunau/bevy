//! Renders the official `examples/ui/window_fallthrough.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_window_fallthrough`

use std::path::PathBuf;

use bevy::{prelude::*, window::CursorOptions};
use bevy_ai_ui_parser::AiUiPlugin;

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::NONE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                transparent: true,
                decorations: true,
                window_level: bevy::window::WindowLevel::AlwaysOnTop,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "window_fallthrough.ir.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, toggle_mouse_passthrough);
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_window_fallthrough")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn toggle_mouse_passthrough(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        cursor_options.hit_test = !cursor_options.hit_test;
    }
}
