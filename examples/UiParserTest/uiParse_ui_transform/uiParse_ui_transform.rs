//! Renders the official `examples/ui/ui_transform.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_ui_transform`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId, BuiLogicTags};

const NORMAL_BUTTON: Color = Color::WHITE;
const HOVERED_BUTTON: Color = Color::srgba(1.0, 1.0, 0.0, 1.0);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 0.0, 0.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path("ui_transform.json")))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (button_system, translation_system))
        .run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_ui_transform")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&BuiLogicTags>),
        (Changed<Interaction>, With<Button>),
    >,
    mut target_query: Query<(&BuiId, &mut UiTransform)>,
) {
    for (interaction, mut color, maybe_tags) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                color.0 = PRESSED_BUTTON;
                let Some(tags) = maybe_tags else {
                    continue;
                };

                for (id, mut transform) in &mut target_query {
                    if id.0 != "target_panel" {
                        continue;
                    }

                    if tags.0.iter().any(|tag| tag == "Rotate_Neg22_5") {
                        transform.rotation *= Rot2::radians(-std::f32::consts::FRAC_PI_8);
                    }
                    if tags.0.iter().any(|tag| tag == "Rotate_Pos22_5") {
                        transform.rotation *= Rot2::radians(std::f32::consts::FRAC_PI_8);
                    }
                    if tags.0.iter().any(|tag| tag == "Scale_Neg0_25") {
                        transform.scale += -0.25;
                        transform.scale =
                            transform.scale.clamp(Vec2::splat(0.25), Vec2::splat(3.0));
                    }
                    if tags.0.iter().any(|tag| tag == "Scale_Pos0_25") {
                        transform.scale += 0.25;
                        transform.scale =
                            transform.scale.clamp(Vec2::splat(0.25), Vec2::splat(3.0));
                    }
                }
            }
            Interaction::Hovered => {
                color.0 = HOVERED_BUTTON;
            }
            Interaction::None => {
                color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn translation_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut target_query: Query<(&BuiId, &mut UiTransform)>,
) {
    let controls = [
        (KeyCode::ArrowLeft, -Vec2::X),
        (KeyCode::ArrowRight, Vec2::X),
        (KeyCode::ArrowUp, -Vec2::Y),
        (KeyCode::ArrowDown, Vec2::Y),
    ];

    for &(key_code, direction) in &controls {
        if !input.pressed(key_code) {
            continue;
        }

        for (id, mut transform) in &mut target_query {
            if id.0 != "target_panel" {
                continue;
            }

            let d = direction * 50.0 * time.delta_secs();
            let (Val::Px(x), Val::Px(y)) = (transform.translation.x, transform.translation.y)
            else {
                continue;
            };

            let x = (x + d.x).clamp(-150.0, 150.0);
            let y = (y + d.y).clamp(-150.0, 150.0);

            transform.translation = Val2::px(x, y);
        }
    }
}
