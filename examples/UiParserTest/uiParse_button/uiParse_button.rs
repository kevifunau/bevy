//! Renders the official `examples/ui/widgets/button.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_button`

use std::path::PathBuf;

use bevy::{
    color::palettes::basic::*,
    input_focus::{FocusCause, InputFocus},
    prelude::*,
};
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputFocus>()
        .add_plugins(AiUiPlugin::from_path(bui_json_path("button.json")))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, button_system)
        .run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_button")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &BuiId,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<(&BuiId, &mut Text)>,
) {
    for (entity, id, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        if id.0 != "main_button" {
            continue;
        }

        let Some(text_entity) = children.first() else {
            continue;
        };

        let Ok((_, mut text)) = text_query.get_mut(*text_entity) else {
            continue;
        };

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity, FocusCause::Pressed);
                text.0 = "Press".to_string();
                color.0 = PRESSED_BUTTON;
                *border_color = BorderColor::all(RED);
                button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity, FocusCause::Pressed);
                text.0 = "Hover".to_string();
                color.0 = HOVERED_BUTTON;
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                text.0 = "Button".to_string();
                color.0 = NORMAL_BUTTON;
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}
