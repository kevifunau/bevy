//! Renders the official `examples/ui/text/text_input.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_text_input`

use std::path::PathBuf;

use bevy::{
    input_focus::{tab_navigation::TabNavigationPlugin, InputFocus},
    prelude::*,
    text::EditableText,
};
use bevy_ai_ui_parser::{AiUiPlugin, BuiLogicTags};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(TabNavigationPlugin)
        .add_plugins(AiUiPlugin::from_path(bui_json_path("text_input.ir.json")))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, text_submission);
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_text_input")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn text_submission(
    input_focus: Res<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_input: Query<(&mut EditableText, &Name)>,
    mut text_output: Query<(&BuiLogicTags, &mut Text)>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Some(focused_entity) = input_focus.get()
        && let Ok((mut text_input, name)) = text_input.get_mut(focused_entity)
    {
        for (tags, mut text) in &mut text_output {
            if tags.0.iter().any(|tag| tag == "TextOutput") {
                text.0 = format!("{}: {}", name.as_str(), text_input.value());
            }
        }
        text_input.clear();
    }
}
