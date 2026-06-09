//! Renders the official `examples/ui/widgets/tab_navigation.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_tab_navigation`

use std::path::PathBuf;

use bevy::{
    color::palettes::basic::*,
    input_focus::{tab_navigation::TabNavigationPlugin, FocusCause, InputFocus},
    prelude::*,
};
use bevy_ai_ui_parser::{AiUiPlugin, BuiId};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, TabNavigationPlugin))
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "tab_navigation.ir.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (button_system, focus_system, click_focus_system));
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_tab_navigation")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                color.0 = PRESSED_BUTTON;
                *border_color = BorderColor::all(RED);
            }
            Interaction::Hovered => {
                color.0 = HOVERED_BUTTON;
                *border_color = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                color.0 = NORMAL_BUTTON;
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn focus_system(
    mut commands: Commands,
    focus: Res<InputFocus>,
    query: Query<Entity, With<Button>>,
) {
    if !focus.is_changed() {
        return;
    }

    for button in &query {
        if focus.get() == Some(button) {
            commands.entity(button).insert(Outline {
                color: Color::WHITE,
                width: px(2),
                offset: px(2),
            });
        } else {
            commands.entity(button).remove::<Outline>();
        }
    }
}

fn click_focus_system(
    mut focus: ResMut<InputFocus>,
    root_query: Query<&Interaction, (With<Node>, With<BuiId>)>,
    button_query: Query<(Entity, &Interaction), With<Button>>,
) {
    let any_pressed = button_query
        .iter()
        .find(|(_, interaction)| **interaction == Interaction::Pressed)
        .map(|(entity, _)| entity);

    if let Some(entity) = any_pressed {
        focus.set(entity, FocusCause::Pressed);
        return;
    }

    let root_pressed = root_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if root_pressed {
        focus.clear();
    }
}
