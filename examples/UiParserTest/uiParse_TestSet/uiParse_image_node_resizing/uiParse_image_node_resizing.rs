//! Renders the official `examples/ui/images/image_node_resizing.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_image_node_resizing`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

const MIN_RESIZE_VAL: f32 = 1.0;
const IMAGE_GROUP_BOX_MIN_WIDTH: f32 = 50.0;
const IMAGE_GROUP_BOX_MAX_WIDTH: f32 = 100.0;
const IMAGE_GROUP_BOX_MIN_HEIGHT: f32 = 10.0;
const IMAGE_GROUP_BOX_MAX_HEIGHT: f32 = 50.0;
const IMAGE_GROUP_BOX_INIT_WIDTH: f32 =
    (IMAGE_GROUP_BOX_MIN_WIDTH + IMAGE_GROUP_BOX_MAX_WIDTH) / 2.;
const IMAGE_GROUP_BOX_INIT_HEIGHT: f32 =
    (IMAGE_GROUP_BOX_MIN_HEIGHT + IMAGE_GROUP_BOX_MAX_HEIGHT) / 2.;
const TEXT_PREFIX: &str = "Compare NodeImageMode(Auto, Stretch) press `Up`/`Down` to resize height, press `Left`/`Right` to resize width\n";

#[derive(Resource, Debug)]
struct ImageGroupSize {
    height: f32,
    width: f32,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ImageGroupSize {
            height: IMAGE_GROUP_BOX_INIT_HEIGHT,
            width: IMAGE_GROUP_BOX_INIT_WIDTH,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "image_node_resizing.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, resize_image_groups);
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_image_node_resizing")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn resize_image_groups(
    keycode: Res<ButtonInput<KeyCode>>,
    mut image_group_size: ResMut<ImageGroupSize>,
    mut nodes: Query<(&BuiId, &mut Node)>,
    mut texts: Query<(&BuiId, &mut Text)>,
) {
    let mut changed = false;

    if keycode.pressed(KeyCode::ArrowUp) {
        image_group_size.height =
            (image_group_size.height + MIN_RESIZE_VAL).min(IMAGE_GROUP_BOX_MAX_HEIGHT);
        changed = true;
    }
    if keycode.pressed(KeyCode::ArrowDown) {
        image_group_size.height =
            (image_group_size.height - MIN_RESIZE_VAL).max(IMAGE_GROUP_BOX_MIN_HEIGHT);
        changed = true;
    }
    if keycode.pressed(KeyCode::ArrowLeft) {
        image_group_size.width =
            (image_group_size.width - MIN_RESIZE_VAL).max(IMAGE_GROUP_BOX_MIN_WIDTH);
        changed = true;
    }
    if keycode.pressed(KeyCode::ArrowRight) {
        image_group_size.width =
            (image_group_size.width + MIN_RESIZE_VAL).min(IMAGE_GROUP_BOX_MAX_WIDTH);
        changed = true;
    }

    if !changed {
        return;
    }

    for (id, mut node) in &mut nodes {
        if id.0 == "auto_image_group" || id.0 == "stretch_image_group" {
            node.height = percent(image_group_size.height);
            node.width = percent(image_group_size.width);
        }
    }

    for (id, mut text) in &mut texts {
        if id.0 == "resize_status_text" {
            text.0 = format!(
                "{TEXT_PREFIX}height : {}%, width : {}%",
                image_group_size.height, image_group_size.width
            );
        }
    }
}
