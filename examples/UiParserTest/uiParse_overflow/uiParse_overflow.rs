//! Renders the official `examples/ui/scroll_and_overflow/overflow.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_overflow`

use std::path::PathBuf;

use bevy::color::palettes::css::{RED, WHITE};
use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path("overflow.json")))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, update_highlight_borders)
        .run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_overflow")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_highlight_borders(
    mut query: Query<(&BuiId, &Interaction, &mut BorderColor), Changed<Interaction>>,
) {
    for (id, interaction, mut border) in &mut query {
        if !id.0.ends_with("_image") {
            continue;
        }

        *border = BorderColor::all(match *interaction {
            Interaction::Pressed => RED.into(),
            Interaction::Hovered => WHITE.into(),
            Interaction::None => Color::NONE,
        });
    }
}
