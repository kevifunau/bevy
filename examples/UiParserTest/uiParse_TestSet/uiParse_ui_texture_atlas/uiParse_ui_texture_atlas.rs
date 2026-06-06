//! Renders the official `examples/ui/images/ui_texture_atlas.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_ui_texture_atlas`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "ui_texture_atlas.ir.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, increment_atlas_index);
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_ui_texture_atlas")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn increment_atlas_index(
    mut image_nodes: Query<(&BuiId, &mut ImageNode)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (id, mut image_node) in &mut image_nodes {
            if id.0 == "atlas_image"
                && let Some(atlas) = &mut image_node.texture_atlas
            {
                atlas.index = (atlas.index + 1) % 6;
            }
        }
    }
}
