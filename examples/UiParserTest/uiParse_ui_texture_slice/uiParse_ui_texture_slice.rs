//! Renders the official `examples/ui/images/ui_texture_slice.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_ui_texture_slice`

use std::path::PathBuf;

use bevy::{
    color::palettes::css::{GOLD, ORANGE},
    prelude::*,
};
use bevy_ai_ui_parser::AiUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "ui_texture_slice.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, button_system)
        .run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_ui_texture_slice")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &Children, &mut ImageNode),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children, mut image) in &mut interaction_query {
        let Some(first_child) = children.first() else {
            continue;
        };
        let Ok(mut text) = text_query.get_mut(*first_child) else {
            continue;
        };

        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();
                image.color = GOLD.into();
            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
                image.color = ORANGE.into();
            }
            Interaction::None => {
                **text = "Button".to_string();
                image.color = Color::WHITE;
            }
        }
    }
}
