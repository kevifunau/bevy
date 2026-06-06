//! Renders the official `examples/ui/ui_target_camera.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_ui_target_camera`

use std::path::PathBuf;

use bevy::color::palettes::css::{BLUE, GREEN, RED};
use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "ui_target_camera.ir.json",
        )))
        .add_systems(Startup, setup_cameras)
        .add_systems(Update, install_press_observers_system);
    auto_screenshot::install(&mut app);
    app.run();
}

#[derive(Resource, Default)]
struct PressObserversInstalled(bool);

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_ui_target_camera")
        .join(file_name)
}

fn setup_cameras(mut commands: Commands) {
    commands.insert_resource(PressObserversInstalled::default());

    for (name, color, order) in [
        ("camera_red", RED, 0isize),
        ("camera_green", GREEN, 1isize),
        ("camera_blue", BLUE, 2isize),
    ] {
        commands.spawn((
            Name::new(name),
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(color.into()),
                order,
                ..default()
            },
        ));
    }
}

fn install_press_observers_system(
    mut commands: Commands,
    mut installed: ResMut<PressObserversInstalled>,
    ids: Query<(Entity, &BuiId)>,
) {
    if installed.0 {
        return;
    }

    let mappings = [
        ("camera_box_red", "camera_red_label", "camera_red"),
        ("camera_box_green", "camera_green_label", "camera_green"),
        ("camera_box_blue", "camera_blue_label", "camera_blue"),
    ];

    let mut resolved = Vec::new();
    for (box_id, label_id, camera_name) in mappings {
        let box_entity = ids
            .iter()
            .find_map(|(entity, id)| (id.0 == box_id).then_some(entity));
        let label_entity = ids
            .iter()
            .find_map(|(entity, id)| (id.0 == label_id).then_some(entity));
        if let (Some(box_entity), Some(label_entity)) = (box_entity, label_entity) {
            resolved.push((box_entity, label_entity, camera_name.to_string()));
        }
    }

    if resolved.len() != 3 {
        return;
    }

    for (box_entity, label_entity, camera_name) in resolved {
        commands.entity(box_entity).observe(
            move |on_pressed: On<Pointer<Press>>,
                  mut label_query: Query<&mut Text>,
                  mut camera_query: Query<(&Name, &mut Camera)>| {
                let Ok(mut label_text) = label_query.get_mut(label_entity) else {
                    return;
                };

                for (name, mut camera) in &mut camera_query {
                    if name.as_str() != camera_name {
                        continue;
                    }

                    camera.order += match on_pressed.button {
                        PointerButton::Primary => 1,
                        _ => -1,
                    };
                    label_text.0 = format!("{}", camera.order);
                    break;
                }
            },
        );
    }

    installed.0 = true;
}
