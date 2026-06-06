use bevy_ecs::prelude::*;
use bevy_input_focus::{FocusCause, InputFocus};
use bevy_text::EditableText;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::components::{BuiTextInput, BuiTextInputMirror, BuiTextInputProxy},
    model::BuiTextConfig,
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
