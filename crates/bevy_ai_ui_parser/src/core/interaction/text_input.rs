use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, ButtonInput};
use bevy_input_focus::{FocusCause, InputFocus};
use bevy_text::EditableText;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::{
        components::{
            BuiActions, BuiBindings, BuiTextInput, BuiTextInputMirror, BuiTextInputProxy,
        },
        types::{BuiActionTrigger, BuiActionTriggered, BuiBindingValue, BuiStateSet},
    },
    model::BuiTextConfig,
    runtime::components::BuiId,
};

fn current_text_input_display(
    editable_text: &EditableText,
    text_config: &BuiTextConfig,
    is_focused: bool,
) -> String {
    let value = editable_text.value().to_string();

    if value.is_empty() && !is_focused {
        return text_config.placeholder.clone().unwrap_or_default();
    }

    value
}

pub(crate) fn text_input_proxy_focus_system(
    mut input_focus: ResMut<InputFocus>,
    proxies: Query<(&Interaction, &BuiTextInputProxy), Changed<Interaction>>,
) {
    for (interaction, proxy) in &proxies {
        if *interaction == Interaction::Pressed {
            input_focus.set(proxy.target, FocusCause::Pressed);
        }
    }
}

pub(crate) fn sync_text_input_mirror_system(
    input_focus: Res<InputFocus>,
    inputs: Query<(Entity, &EditableText, &BuiTextConfig), With<BuiTextInput>>,
    mut mirrors: Query<(&BuiTextInputMirror, &mut Text)>,
) {
    for (mirror, mut text) in &mut mirrors {
        let Ok((input_entity, editable_text, text_config)) = inputs.get(mirror.target) else {
            continue;
        };

        let is_focused = input_focus.get() == Some(input_entity);
        let display = current_text_input_display(editable_text, text_config, is_focused);

        if text.0 != display {
            text.0 = display;
        }
    }
}

pub(crate) fn dispatch_text_input_value_changed_system(
    inputs: Query<
        (
            Entity,
            &BuiId,
            &EditableText,
            Option<&BuiActions>,
            Option<&BuiBindings>,
        ),
        (With<BuiTextInput>, Changed<EditableText>),
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (entity, id, editable_text, actions, bindings) in &inputs {
        let value = editable_text.value().to_string();
        state_writer.write(BuiStateSet {
            key: id.0.clone(),
            value: BuiBindingValue::Text(value.clone()),
        });
        if let Some(bindings) = bindings {
            for binding in &bindings.0 {
                if binding.target == "text.content" {
                    state_writer.write(BuiStateSet {
                        key: binding.source.clone(),
                        value: BuiBindingValue::Text(value.clone()),
                    });
                }
            }
        }
        dispatch_text_input_actions(
            actions,
            entity,
            id,
            BuiActionTrigger::ValueChanged,
            &mut action_writer,
        );
    }
}

pub(crate) fn dispatch_text_input_submit_system(
    input_focus: Res<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    inputs: Query<(Entity, &BuiId, Option<&BuiActions>), With<BuiTextInput>>,
    mut action_writer: MessageWriter<BuiActionTriggered>,
) {
    if !keyboard_input.just_pressed(KeyCode::Enter) {
        return;
    }
    let Some(focused_entity) = input_focus.get() else {
        return;
    };
    let Ok((entity, id, actions)) = inputs.get(focused_entity) else {
        return;
    };

    dispatch_text_input_actions(
        actions,
        entity,
        id,
        BuiActionTrigger::Submit,
        &mut action_writer,
    );
}

#[derive(Resource, Debug, Default)]
pub(crate) struct BuiTextInputFocusState {
    focused: Option<Entity>,
}

pub(crate) fn dispatch_text_input_focus_events_system(
    input_focus: Res<InputFocus>,
    mut focus_state: ResMut<BuiTextInputFocusState>,
    inputs: Query<(Entity, &BuiId, Option<&BuiActions>), With<BuiTextInput>>,
    mut action_writer: MessageWriter<BuiActionTriggered>,
) {
    if !input_focus.is_changed() {
        return;
    }

    let previous = focus_state.focused;
    let current = input_focus.get();
    if previous == current {
        return;
    }

    if let Some(previous) = previous
        && let Ok((entity, id, actions)) = inputs.get(previous)
    {
        dispatch_text_input_actions(
            actions,
            entity,
            id,
            BuiActionTrigger::Blur,
            &mut action_writer,
        );
    }
    if let Some(current) = current
        && let Ok((entity, id, actions)) = inputs.get(current)
    {
        dispatch_text_input_actions(
            actions,
            entity,
            id,
            BuiActionTrigger::Focus,
            &mut action_writer,
        );
    }

    focus_state.focused = current;
}

fn dispatch_text_input_actions(
    actions: Option<&BuiActions>,
    entity: Entity,
    id: &BuiId,
    trigger: BuiActionTrigger,
    action_writer: &mut MessageWriter<BuiActionTriggered>,
) {
    let Some(actions) = actions else {
        return;
    };

    for action in &actions.0 {
        if text_input_trigger_name(action.event.as_str()) != Some(trigger) {
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

fn text_input_trigger_name(value: &str) -> Option<BuiActionTrigger> {
    match value {
        "value_changed" | "change" | "changed" => Some(BuiActionTrigger::ValueChanged),
        "submit" | "submitted" => Some(BuiActionTrigger::Submit),
        "focus" | "focused" => Some(BuiActionTrigger::Focus),
        "blur" | "unfocused" => Some(BuiActionTrigger::Blur),
        _ => None,
    }
}
