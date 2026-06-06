//! Renders the official `examples/ui/layout/size_constraints.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_size_constraints`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiId, BuiLogicTags};

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

const ACTIVE_BORDER_COLOR: Color = Color::srgba(0.98039216, 0.92156863, 0.84313726, 1.0);
const INACTIVE_BORDER_COLOR: Color = Color::BLACK;
const ACTIVE_INNER_COLOR: Color = Color::WHITE;
const INACTIVE_INNER_COLOR: Color = Color::srgba(0.0, 0.0, 0.5019608, 1.0);
const ACTIVE_TEXT_COLOR: Color = Color::BLACK;
const HOVERED_TEXT_COLOR: Color = Color::WHITE;
const UNHOVERED_TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_message::<ButtonActivated>()
        .add_plugins(AiUiPlugin::from_path(bui_json_path(
            "size_constraints.json",
        )))
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (update_buttons_system, update_radio_buttons_colors_system),
        );
    auto_screenshot::install(&mut app);
    app.run();
}

#[derive(Message, Clone, Copy)]
struct ButtonActivated {
    button: Entity,
    constraint: ConstraintKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstraintKind {
    FlexBasis,
    Width,
    MinWidth,
    MaxWidth,
}

#[derive(Debug, Clone, Copy)]
enum ConstraintValue {
    Auto,
    Percent(f32),
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_size_constraints")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_buttons_system(
    mut interactions: Query<(Entity, &Interaction, &BuiLogicTags, &Children), Changed<Interaction>>,
    mut nodes: Query<(&BuiId, &mut Node)>,
    mut texts: Query<&mut TextColor>,
    children_query: Query<&Children>,
    mut activated: MessageWriter<ButtonActivated>,
) {
    for (entity, interaction, tags, children) in &mut interactions {
        let Some((constraint, value)) = parse_button_metadata(&tags.0) else {
            continue;
        };

        match *interaction {
            Interaction::Pressed => {
                set_bar_constraint(&mut nodes, constraint, value);
                activated.write(ButtonActivated {
                    button: entity,
                    constraint,
                });
            }
            Interaction::Hovered => {
                set_button_text_color(
                    children,
                    &children_query,
                    &mut texts,
                    HOVERED_TEXT_COLOR,
                    true,
                );
            }
            Interaction::None => {
                set_button_text_color(
                    children,
                    &children_query,
                    &mut texts,
                    UNHOVERED_TEXT_COLOR,
                    true,
                );
            }
        }
    }
}

fn update_radio_buttons_colors_system(
    mut activated: MessageReader<ButtonActivated>,
    buttons: Query<(Entity, &BuiLogicTags, &Interaction, &Children), With<Button>>,
    children_query: Query<&Children>,
    mut borders: Query<&mut BorderColor>,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut texts: Query<&mut TextColor>,
) {
    for ButtonActivated { button, constraint } in activated.read().copied() {
        for (entity, tags, interaction, children) in &buttons {
            if parse_constraint(&tags.0) != Some(constraint) {
                continue;
            }

            let active = entity == button;
            if let Ok(mut border) = borders.get_mut(entity) {
                *border = BorderColor::all(if active {
                    ACTIVE_BORDER_COLOR
                } else {
                    INACTIVE_BORDER_COLOR
                });
            }

            for &child in children {
                if let Ok(mut background) = backgrounds.get_mut(child) {
                    background.0 = if active {
                        ACTIVE_INNER_COLOR
                    } else {
                        INACTIVE_INNER_COLOR
                    };
                }
            }

            let label_color = if active {
                ACTIVE_TEXT_COLOR
            } else if matches!(*interaction, Interaction::Hovered) {
                HOVERED_TEXT_COLOR
            } else {
                UNHOVERED_TEXT_COLOR
            };

            set_button_text_color(children, &children_query, &mut texts, label_color, false);
        }
    }
}

fn parse_button_metadata(tags: &[String]) -> Option<(ConstraintKind, ConstraintValue)> {
    Some((parse_constraint(tags)?, parse_value(tags)?))
}

fn parse_constraint(tags: &[String]) -> Option<ConstraintKind> {
    if tags.iter().any(|tag| tag == "Constraint_FlexBasis") {
        Some(ConstraintKind::FlexBasis)
    } else if tags.iter().any(|tag| tag == "Constraint_Width") {
        Some(ConstraintKind::Width)
    } else if tags.iter().any(|tag| tag == "Constraint_MinWidth") {
        Some(ConstraintKind::MinWidth)
    } else if tags.iter().any(|tag| tag == "Constraint_MaxWidth") {
        Some(ConstraintKind::MaxWidth)
    } else {
        None
    }
}

fn parse_value(tags: &[String]) -> Option<ConstraintValue> {
    if tags.iter().any(|tag| tag == "Value_Auto") {
        return Some(ConstraintValue::Auto);
    }

    tags.iter()
        .find_map(|tag| tag.strip_prefix("Value_Percent_"))
        .and_then(|raw| raw.parse::<f32>().ok())
        .map(ConstraintValue::Percent)
}

fn set_bar_constraint(
    nodes: &mut Query<(&BuiId, &mut Node)>,
    constraint: ConstraintKind,
    value: ConstraintValue,
) {
    for (id, mut node) in nodes.iter_mut() {
        if id.0 != "bar_fill" {
            continue;
        }

        let value = match value {
            ConstraintValue::Auto => Val::Auto,
            ConstraintValue::Percent(percent) => Val::Percent(percent),
        };

        match constraint {
            ConstraintKind::FlexBasis => node.flex_basis = value,
            ConstraintKind::Width => node.width = value,
            ConstraintKind::MinWidth => node.min_width = value,
            ConstraintKind::MaxWidth => node.max_width = value,
        }
    }
}

fn set_button_text_color(
    button_children: &Children,
    children_query: &Query<&Children>,
    texts: &mut Query<&mut TextColor>,
    color: Color,
    skip_active_text: bool,
) {
    for &child in button_children {
        let Ok(grand_children) = children_query.get(child) else {
            continue;
        };

        for &grandchild in grand_children {
            let Ok(mut text_color) = texts.get_mut(grandchild) else {
                continue;
            };

            if skip_active_text && text_color.0 == ACTIVE_TEXT_COLOR {
                continue;
            }

            text_color.0 = color;
        }
    }
}
