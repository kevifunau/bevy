use bevy_ecs::prelude::*;
use bevy_input::{
    gamepad::{Gamepad, GamepadButton},
    keyboard::KeyCode,
    ButtonInput,
};
use bevy_input_focus::InputFocus;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::{
        components::{
            BuiActions, BuiDisabled, BuiDropdownGroupDefinition, BuiDropdownItem, BuiVisualState,
        },
        types::{
            BuiActionTrigger, BuiActionTriggered, BuiBindingValue, BuiStateSet, BuiStateStore,
        },
    },
    runtime::components::BuiId,
};

pub(crate) fn dispatch_bui_dropdown_selection_system(
    groups: Query<&BuiDropdownGroupDefinition>,
    items: Query<
        (
            Entity,
            &Interaction,
            &BuiId,
            &BuiDropdownItem,
            Option<&BuiActions>,
            Has<BuiDisabled>,
        ),
        Changed<Interaction>,
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (entity, interaction, id, item, actions, disabled) in &items {
        if disabled || *interaction != Interaction::Pressed {
            continue;
        }

        let Some(group) = groups.iter().find(|group| group.group == item.group) else {
            continue;
        };

        state_writer.write(BuiStateSet {
            key: group.source.clone(),
            value: BuiBindingValue::Text(item.value.clone()),
        });

        if let Some(actions) = actions {
            for action in &actions.0 {
                if !matches!(
                    action.event.as_str(),
                    "selection_changed" | "selection-changed" | "select" | "selected"
                ) {
                    continue;
                }
                action_writer.write(BuiActionTriggered {
                    entity,
                    id: id.0.clone(),
                    action: action.emit.clone(),
                    trigger: BuiActionTrigger::SelectionChanged,
                });
            }
        }
    }
}

pub(crate) fn sync_bui_dropdown_selected_state_system(
    groups: Query<&BuiDropdownGroupDefinition>,
    items: Query<(Entity, &BuiDropdownItem)>,
    state_store: Res<BuiStateStore>,
    mut commands: Commands,
) {
    if !state_store.is_changed() {
        return;
    }

    for (entity, item) in &items {
        let Some(group) = groups.iter().find(|group| group.group == item.group) else {
            continue;
        };

        let selected = matches!(
            state_store.0.get(&group.source),
            Some(BuiBindingValue::Text(value)) if value == &item.value
        );

        if selected {
            commands
                .entity(entity)
                .insert(BuiVisualState("selected".to_string()));
        } else {
            commands.entity(entity).remove::<BuiVisualState>();
        }
    }
}

pub(crate) fn focused_dropdown_confirm_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    input_focus: Res<InputFocus>,
    groups: Query<&BuiDropdownGroupDefinition>,
    items: Query<(
        Entity,
        &BuiId,
        &BuiDropdownItem,
        Option<&BuiActions>,
        Has<BuiDisabled>,
    )>,
    mut action_writer: MessageWriter<BuiActionTriggered>,
    mut state_writer: MessageWriter<BuiStateSet>,
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
    let Ok((entity, id, item, actions, disabled)) = items.get(focused_entity) else {
        return;
    };
    if disabled {
        return;
    }
    let Some(group) = groups.iter().find(|group| group.group == item.group) else {
        return;
    };

    state_writer.write(BuiStateSet {
        key: group.source.clone(),
        value: BuiBindingValue::Text(item.value.clone()),
    });

    if let Some(actions) = actions {
        for action in &actions.0 {
            if !matches!(
                action.event.as_str(),
                "selection_changed" | "selection-changed" | "select" | "selected"
            ) {
                continue;
            }
            action_writer.write(BuiActionTriggered {
                entity,
                id: id.0.clone(),
                action: action.emit.clone(),
                trigger: BuiActionTrigger::SelectionChanged,
            });
        }
    }
}
