use bevy_ecs::prelude::*;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::components::{BuiActions, BuiDisabled},
    interaction::types::{BuiActionTrigger, BuiActionTriggered},
    runtime::components::BuiId,
    style::css_parser::normalize_token,
};

fn parse_action_trigger(value: &str) -> Result<BuiActionTrigger, String> {
    match normalize_token(value).as_str() {
        "press" | "pressed" => Ok(BuiActionTrigger::Press),
        "hover_enter" | "hovered" => Ok(BuiActionTrigger::HoverEnter),
        "hover_exit" | "unhovered" => Ok(BuiActionTrigger::HoverExit),
        _ => Err(format!(
            "Unsupported action event '{}'. Expected one of: press, hover_enter, hover_exit.",
            value
        )),
    }
}

pub(crate) fn dispatch_bui_actions_system(
    interactions: Query<
        (Entity, &Interaction, &BuiId, &BuiActions, Has<BuiDisabled>),
        Changed<Interaction>,
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
) {
    for (entity, interaction, id, actions, disabled) in &interactions {
        if disabled {
            continue;
        }

        for action in &actions.0 {
            let Ok(trigger) = parse_action_trigger(&action.event) else {
                continue;
            };

            let matched = match trigger {
                BuiActionTrigger::Press => *interaction == Interaction::Pressed,
                BuiActionTrigger::HoverEnter => *interaction == Interaction::Hovered,
                BuiActionTrigger::HoverExit => *interaction == Interaction::None,
            };

            if !matched {
                continue;
            }

            action_writer.write(BuiActionTriggered {
                entity,
                id: id.0.clone(),
                action: action.emit.clone(),
                trigger,
            });
        }
    }
}
