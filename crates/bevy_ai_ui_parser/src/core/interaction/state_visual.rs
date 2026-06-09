use bevy_camera::visibility::Visibility;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::*;
use bevy_input_focus::InputFocus;
use bevy_text::TextColor;
use bevy_ui::prelude::*;
use bevy_ui::Checked;

use crate::core::interaction::components::{
    BuiDisabled, BuiVisualState, BuiVisualStateDefinitions,
};
use crate::core::style::css_parser::{
    parse_color, parse_rotation, parse_val2, parse_vec2, parse_visibility,
};

fn inherited_visual_state(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    explicit_states: &Query<&BuiVisualState>,
    self_state: Option<&BuiVisualState>,
) -> Option<String> {
    if self_state.is_some() {
        return None;
    }

    let mut current = entity;

    while let Ok(child_of) = child_of_query.get(current) {
        let parent = child_of.parent();
        if let Ok(state) = explicit_states.get(parent) {
            return Some(state.0.clone());
        }
        current = parent;
    }

    None
}

fn resolve_visual_state_name(
    definitions: &BuiVisualStateDefinitions,
    base_state: Option<&str>,
    auto_state: Option<&str>,
) -> Option<String> {
    if let (Some(base), Some(auto)) = (base_state, auto_state) {
        let combined = format!("{base}_{auto}");
        if definitions.states.contains_key(&combined) {
            return Some(combined);
        }

        let reversed = format!("{auto}_{base}");
        if definitions.states.contains_key(&reversed) {
            return Some(reversed);
        }
    }

    if let Some(base) = base_state
        && definitions.states.contains_key(base)
    {
        return Some(base.to_string());
    }

    if let Some(auto) = auto_state
        && definitions.states.contains_key(auto)
    {
        return Some(auto.to_string());
    }

    None
}

pub(crate) fn apply_bui_visual_states_system(
    input_focus: Res<InputFocus>,
    child_of_query: Query<&ChildOf>,
    explicit_states: Query<&BuiVisualState>,
    visual_states: Query<(
        Entity,
        &BuiVisualStateDefinitions,
        Option<&BuiVisualState>,
        Option<&Interaction>,
        Has<Checked>,
        Has<BuiDisabled>,
    )>,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut borders: Query<&mut BorderColor>,
    mut text_colors: Query<&mut TextColor>,
    mut ui_transforms: Query<&mut UiTransform>,
    mut visibilities: Query<&mut Visibility>,
) {
    for (entity, definitions, explicit_state, interaction, checked, disabled) in &visual_states {
        let inherited_state =
            inherited_visual_state(entity, &child_of_query, &explicit_states, explicit_state);
        let base_state = explicit_state
            .map(|state| state.0.clone())
            .or(inherited_state);
        let auto_state = disabled
            .then_some("disabled")
            .or_else(|| (input_focus.get() == Some(entity)).then_some("focused"))
            .or_else(|| checked.then_some("checked"))
            .or_else(|| {
                interaction.and_then(|interaction| match *interaction {
                    Interaction::Pressed => Some("pressed"),
                    Interaction::Hovered => Some("hovered"),
                    Interaction::None => Some("normal"),
                })
            })
            .or_else(|| {
                definitions
                    .states
                    .contains_key("normal")
                    .then_some("normal")
            });

        let Some(state_name) =
            resolve_visual_state_name(definitions, base_state.as_deref(), auto_state)
        else {
            continue;
        };

        let Some(state_visual) = definitions.states.get(&state_name) else {
            continue;
        };

        if let Some(color) = &state_visual.visuals.background_color
            && let Ok(mut background) = backgrounds.get_mut(entity)
            && let Ok(parsed) = parse_color(color)
        {
            background.0 = parsed;
        }

        if let Some(color) = &state_visual.visuals.border_color
            && let Ok(mut border) = borders.get_mut(entity)
            && let Ok(parsed) = parse_color(color)
        {
            *border = BorderColor::all(parsed);
        }

        if let Some(color) = &state_visual.text_color
            && let Ok(mut text_color) = text_colors.get_mut(entity)
            && let Ok(parsed) = parse_color(color)
        {
            text_color.0 = parsed;
        }

        if let Ok(mut ui_transform) = ui_transforms.get_mut(entity) {
            if let Some(value) = &state_visual.styles.ui_translation
                && let Ok(parsed) = parse_val2(value)
            {
                ui_transform.translation = parsed;
            }
            if let Some(value) = &state_visual.styles.ui_scale
                && let Ok(parsed) = parse_vec2(value)
            {
                ui_transform.scale = parsed;
            }
            if let Some(value) = &state_visual.styles.ui_rotation
                && let Ok(parsed) = parse_rotation(value)
            {
                ui_transform.rotation = parsed;
            }
        }

        if let Some(value) = &state_visual.styles.visibility
            && let Ok(mut visibility) = visibilities.get_mut(entity)
            && let Ok(parsed) = parse_visibility(value)
        {
            *visibility = parsed;
        }
    }
}
