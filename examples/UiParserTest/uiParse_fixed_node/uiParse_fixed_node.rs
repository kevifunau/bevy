//! Renders the official `examples/ui/layout/fixed_node.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_fixed_node`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

const ROOT_IDLE: Color = Color::srgba(0.0, 0.0, 1.0, 1.0);
const ROOT_HOVER: Color = Color::srgba(1.0, 0.0, 0.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path("fixed_node.json")))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, install_hover_observers_system)
        .run();
}

#[derive(Resource, Default)]
struct HoverObserversInstalled(bool);

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_fixed_node")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(HoverObserversInstalled::default());
}

fn install_hover_observers_system(
    mut commands: Commands,
    mut installed: ResMut<HoverObserversInstalled>,
    ids: Query<(Entity, &BuiId)>,
) {
    if installed.0 {
        return;
    }

    let mut root = None;
    let mut fixed_child = None;

    for (entity, id) in &ids {
        match id.0.as_str() {
            "fixed_node_root" => root = Some(entity),
            "fixed_square" => fixed_child = Some(entity),
            _ => {}
        }
    }

    let (Some(root), Some(fixed_child)) = (root, fixed_child) else {
        return;
    };

    commands
        .entity(fixed_child)
        .observe(
            move |_over: On<Pointer<Over>>, mut colors: Query<&mut BackgroundColor>| {
                if let Ok(mut color) = colors.get_mut(root) {
                    color.0 = ROOT_HOVER;
                }
            },
        )
        .observe(
            move |_leave: On<Pointer<Leave>>, mut colors: Query<&mut BackgroundColor>| {
                if let Ok(mut color) = colors.get_mut(root) {
                    color.0 = ROOT_IDLE;
                }
            },
        );

    installed.0 = true;
}
