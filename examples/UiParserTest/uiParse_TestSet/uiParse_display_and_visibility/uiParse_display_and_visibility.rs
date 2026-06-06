//! Renders the official `examples/ui/layout/display_and_visibility.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_display_and_visibility`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId, BuiLogicTags};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

const HIDDEN_COLOR: Color = Color::srgb(1.0, 0.7, 0.7);
const BUTTON_IDLE: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);
const BUTTON_HOVER: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const BUTTON_TEXT_DEFAULT: Color = Color::WHITE;
const BUTTON_TEXT_HOVER: Color = Color::srgb(1.0, 1.0, 0.0);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "display_and_visibility.ir.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (toggle_target_system, button_hover_system));
    auto_screenshot::install(&mut app);
    app.run();
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_display_and_visibility")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn toggle_target_system(
    mut interactions: Query<
        (&Interaction, &Children, &BuiLogicTags),
        (Changed<Interaction>, With<Button>),
    >,
    mut node_targets: Query<(&BuiId, &mut Node)>,
    mut visibility_targets: Query<(&BuiId, &mut Visibility)>,
    mut texts: Query<(&BuiId, &mut Text, &mut TextColor)>,
) {
    for (interaction, children, tags) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(target_id) = logic_value(&tags.0, "Target_") else {
            continue;
        };

        if tags.0.iter().any(|tag| tag == "Action_Toggle_Display") {
            if let Some(next_label) = toggle_display_target(target_id, &mut node_targets) {
                update_button_label(children, &mut texts, next_label);
            }
        } else if tags.0.iter().any(|tag| tag == "Action_Toggle_Visibility")
            && let Some((next_label, hidden)) =
                toggle_visibility_target(target_id, &mut visibility_targets)
        {
            update_button_label_with_color(children, &mut texts, next_label, hidden);
        }
    }
}

fn button_hover_system(
    mut buttons: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut texts: Query<(&mut TextColor, &Text)>,
) {
    for (interaction, children, mut background) in &mut buttons {
        match *interaction {
            Interaction::Hovered => {
                background.0 = BUTTON_HOVER;
                for child in children.iter() {
                    if let Ok((mut text_color, _)) = texts.get_mut(child) {
                        text_color.0 = BUTTON_TEXT_HOVER;
                    }
                }
            }
            _ => {
                background.0 = BUTTON_IDLE;
                for child in children.iter() {
                    if let Ok((mut text_color, text)) = texts.get_mut(child) {
                        text_color.0 = if text.contains("None") || text.contains("Hidden") {
                            HIDDEN_COLOR
                        } else {
                            BUTTON_TEXT_DEFAULT
                        };
                    }
                }
            }
        }
    }
}

fn logic_value<'a>(tags: &'a [String], prefix: &str) -> Option<&'a str> {
    tags.iter().find_map(|tag| tag.strip_prefix(prefix))
}

fn toggle_display_target(
    target_id: &str,
    targets: &mut Query<(&BuiId, &mut Node)>,
) -> Option<String> {
    for (id, mut node) in targets.iter_mut() {
        if id.0 != target_id {
            continue;
        }

        node.display = match node.display {
            Display::Flex => Display::None,
            Display::None => Display::Flex,
            Display::Block => Display::None,
            Display::Grid => Display::None,
        };

        return Some(format!("Display::{:?}", node.display));
    }

    None
}

fn toggle_visibility_target(
    target_id: &str,
    targets: &mut Query<(&BuiId, &mut Visibility)>,
) -> Option<(String, bool)> {
    for (id, mut visibility) in targets.iter_mut() {
        if id.0 != target_id {
            continue;
        }

        *visibility = match *visibility {
            Visibility::Inherited => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Inherited,
        };

        let hidden = matches!(*visibility, Visibility::Hidden);
        return Some((format!("Visibility::{:?}", *visibility), hidden));
    }

    None
}

fn update_button_label(
    children: &Children,
    texts: &mut Query<(&BuiId, &mut Text, &mut TextColor)>,
    label: String,
) {
    update_button_label_with_color(children, texts, label, false);
}

fn update_button_label_with_color(
    children: &Children,
    texts: &mut Query<(&BuiId, &mut Text, &mut TextColor)>,
    label: String,
    hidden: bool,
) {
    for child in children.iter() {
        if let Ok((_, mut text, mut text_color)) = texts.get_mut(child) {
            text.0 = label.clone();
            text_color.0 = if hidden {
                HIDDEN_COLOR
            } else {
                BUTTON_TEXT_DEFAULT
            };
        }
    }
}
