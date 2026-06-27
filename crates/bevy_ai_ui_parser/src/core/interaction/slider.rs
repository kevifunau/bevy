use bevy_ecs::prelude::*;
use bevy_input::{
    gamepad::{Gamepad, GamepadButton},
    keyboard::KeyCode,
    ButtonInput,
};
use bevy_input_focus::InputFocus;
use bevy_ui_widgets::{SliderRange, SliderStep, SliderValue};

use crate::core::{
    interaction::{
        components::{BuiActions, BuiBindings},
        types::{BuiActionTrigger, BuiActionTriggered, BuiBindingValue, BuiStateSet},
    },
    runtime::components::BuiId,
};

pub(crate) fn dispatch_slider_value_changed_system(
    sliders: Query<
        (
            Entity,
            &BuiId,
            &SliderValue,
            Option<&BuiActions>,
            Option<&BuiBindings>,
        ),
        Changed<SliderValue>,
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (entity, id, value, actions, bindings) in &sliders {
        state_writer.write(BuiStateSet {
            key: id.0.clone(),
            value: BuiBindingValue::Number(value.0),
        });
        if let Some(bindings) = bindings {
            for binding in &bindings.0 {
                if binding.target == "value" {
                    state_writer.write(BuiStateSet {
                        key: binding.source.clone(),
                        value: BuiBindingValue::Number(value.0),
                    });
                }
            }
        }

        let Some(actions) = actions else {
            continue;
        };
        for action in &actions.0 {
            if !matches!(
                action.event.as_str(),
                "value_changed" | "change" | "changed"
            ) {
                continue;
            }
            action_writer.write(BuiActionTriggered {
                entity,
                id: id.0.clone(),
                action: action.emit.clone(),
                trigger: BuiActionTrigger::ValueChanged,
            });
        }
    }
}

pub(crate) fn focused_slider_keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    input_focus: Res<InputFocus>,
    sliders: Query<(&SliderValue, Option<&SliderRange>, Option<&SliderStep>)>,
    mut commands: Commands,
) {
    let keyboard_delta = if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::ArrowUp)
    {
        1.0
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
    {
        -1.0
    } else {
        0.0
    };
    let gamepad_delta = if gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::DPadRight))
    {
        1.0
    } else if gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::DPadLeft))
    {
        -1.0
    } else {
        0.0
    };
    let delta = if gamepad_delta != 0.0 {
        gamepad_delta
    } else {
        keyboard_delta
    };
    if delta == 0.0 {
        return;
    }

    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok((value, range, step)) = sliders.get(focused) else {
        return;
    };
    let step = step
        .map(|step| step.0)
        .unwrap_or(1.0)
        .abs()
        .max(f32::EPSILON);
    let next = value.0 + delta * step;
    let next = range.map(|range| range.clamp(next)).unwrap_or(next);
    if next != value.0 {
        commands.entity(focused).insert(SliderValue(next));
    }
}
