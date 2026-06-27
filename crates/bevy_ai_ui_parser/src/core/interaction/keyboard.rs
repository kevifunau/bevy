use bevy_ecs::prelude::*;
use bevy_input::{
    gamepad::{Gamepad, GamepadButton},
    keyboard::KeyCode,
    ButtonInput,
};
use bevy_input_focus::{FocusCause, InputFocus};
use bevy_ui::Interaction;
use bevy_ui_widgets::SliderValue;

use crate::core::{
    interaction::{
        components::{
            BuiActions, BuiDisabled, BuiDropdownItem, BuiFocusOrder, BuiScrollView, BuiTextInput,
            BuiTextInputProxy,
        },
        types::{BuiActionTrigger, BuiActionTriggered},
    },
    runtime::components::BuiId,
};

#[derive(Debug, Clone, Copy)]
enum BuiFocusMove {
    Next,
    Previous,
}

pub(crate) fn keyboard_focus_navigation_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut input_focus: ResMut<InputFocus>,
    focusables: Query<(
        Entity,
        Option<&BuiId>,
        Option<&BuiActions>,
        Option<&SliderValue>,
        Option<&BuiScrollView>,
        Option<&BuiDropdownItem>,
        Option<&BuiFocusOrder>,
        Has<BuiTextInput>,
        Has<BuiTextInputProxy>,
        Has<BuiDisabled>,
    )>,
) {
    let tab_move = if keyboard_input.just_pressed(KeyCode::Tab) {
        if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            Some(BuiFocusMove::Previous)
        } else {
            Some(BuiFocusMove::Next)
        }
    } else {
        None
    };
    let gamepad_move = if gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::DPadDown))
    {
        Some(BuiFocusMove::Next)
    } else if gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::DPadUp))
    {
        Some(BuiFocusMove::Previous)
    } else {
        None
    };
    let keyboard_move = if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        Some(BuiFocusMove::Next)
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        Some(BuiFocusMove::Previous)
    } else {
        None
    };

    let Some(direction) = tab_move.or(gamepad_move).or_else(|| {
        let focused = input_focus.get()?;
        if focused_consumes_vertical_arrows(focused, &focusables) {
            None
        } else {
            keyboard_move
        }
    }) else {
        return;
    };

    let mut ordered = focusable_entities(&focusables);
    if ordered.is_empty() {
        return;
    }
    ordered.sort_unstable_by_key(|(order, entity)| (*order, *entity));

    let current = input_focus.get();
    let current_index =
        current.and_then(|entity| ordered.iter().position(|(_, item)| *item == entity));
    let next_index = match (current_index, direction) {
        (Some(index), BuiFocusMove::Next) => (index + 1) % ordered.len(),
        (Some(index), BuiFocusMove::Previous) => (index + ordered.len() - 1) % ordered.len(),
        (None, BuiFocusMove::Next) => 0,
        (None, BuiFocusMove::Previous) => ordered.len() - 1,
    };
    input_focus.set(ordered[next_index].1, FocusCause::Navigated);
}

pub(crate) fn pointer_focus_system(
    mut input_focus: ResMut<InputFocus>,
    focusables: Query<
        (
            Entity,
            &Interaction,
            Option<&BuiActions>,
            Option<&SliderValue>,
            Option<&BuiScrollView>,
            Option<&BuiDropdownItem>,
            Has<BuiTextInput>,
            Has<BuiTextInputProxy>,
            Has<BuiDisabled>,
        ),
        Changed<Interaction>,
    >,
) {
    for (
        entity,
        interaction,
        actions,
        slider,
        scroll_view,
        dropdown_item,
        is_text_input,
        is_text_input_proxy,
        disabled,
    ) in &focusables
    {
        if disabled || *interaction != Interaction::Pressed {
            continue;
        }
        if actions.is_some()
            || slider.is_some()
            || scroll_view.is_some()
            || dropdown_item.is_some()
            || is_text_input
            || is_text_input_proxy
        {
            input_focus.set(entity, FocusCause::Pressed);
        }
    }
}

pub(crate) fn focused_control_confirm_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    input_focus: Res<InputFocus>,
    controls: Query<(
        Entity,
        &BuiId,
        &BuiActions,
        Has<BuiDisabled>,
        Has<BuiTextInput>,
    )>,
    mut action_writer: MessageWriter<BuiActionTriggered>,
) {
    let keyboard_confirm =
        keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Space);
    let gamepad_confirm = gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::South));
    if !keyboard_confirm && !gamepad_confirm {
        return;
    }

    let Some(focused_entity) = input_focus.get() else {
        return;
    };
    let Ok((entity, id, actions, disabled, is_text_input)) = controls.get(focused_entity) else {
        return;
    };
    if disabled || is_text_input {
        return;
    }

    for action in &actions.0 {
        if !matches!(action.event.as_str(), "press" | "pressed") {
            continue;
        }
        action_writer.write(BuiActionTriggered {
            entity,
            id: id.0.clone(),
            action: action.emit.clone(),
            trigger: BuiActionTrigger::Press,
        });
    }
}

fn focusable_entities(
    focusables: &Query<(
        Entity,
        Option<&BuiId>,
        Option<&BuiActions>,
        Option<&SliderValue>,
        Option<&BuiScrollView>,
        Option<&BuiDropdownItem>,
        Option<&BuiFocusOrder>,
        Has<BuiTextInput>,
        Has<BuiTextInputProxy>,
        Has<BuiDisabled>,
    )>,
) -> Vec<(u32, Entity)> {
    focusables
        .iter()
        .filter_map(
            |(
                entity,
                id,
                actions,
                slider,
                scroll_view,
                dropdown_item,
                focus_order,
                is_text_input,
                is_text_input_proxy,
                disabled,
            )| {
                if disabled {
                    return None;
                }
                let has_semantics = actions.is_some()
                    || slider.is_some()
                    || scroll_view.is_some()
                    || dropdown_item.is_some()
                    || is_text_input
                    || is_text_input_proxy;
                (id.is_some() && has_semantics)
                    .then_some((focus_order.map(|order| order.0).unwrap_or(u32::MAX), entity))
            },
        )
        .collect()
}

fn focused_consumes_vertical_arrows(
    focused: Entity,
    focusables: &Query<(
        Entity,
        Option<&BuiId>,
        Option<&BuiActions>,
        Option<&SliderValue>,
        Option<&BuiScrollView>,
        Option<&BuiDropdownItem>,
        Option<&BuiFocusOrder>,
        Has<BuiTextInput>,
        Has<BuiTextInputProxy>,
        Has<BuiDisabled>,
    )>,
) -> bool {
    let Ok((_, _, _, slider, scroll_view, _, _, _, _, _)) = focusables.get(focused) else {
        return false;
    };
    slider.is_some() || scroll_view.is_some()
}
