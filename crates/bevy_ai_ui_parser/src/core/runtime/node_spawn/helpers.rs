use bevy_color::Color;
use bevy_ecs::prelude::EntityCommands;
use bevy_ui::{
    prelude::*,
    BoxShadow, ShadowStyle,
};

use crate::core::{
    model::{BuiBoxShadowConfig, BuiStyles},
    style::css_parser::{parse_color, parse_rotation, parse_val, parse_val2, parse_vec2},
};

pub(super) fn build_box_shadow(box_shadow: &BuiBoxShadowConfig) -> Result<BoxShadow, String> {
    let color = if let Some(color) = &box_shadow.color {
        parse_color(color)?
    } else {
        Color::NONE
    };

    let x_offset = box_shadow
        .offset_x
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);
    let y_offset = box_shadow
        .offset_y
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);
    let blur_radius = box_shadow
        .blur_radius
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);
    let spread_radius = box_shadow
        .spread_radius
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);

    Ok(BoxShadow::from(ShadowStyle {
        color,
        x_offset: if box_shadow.inset {
            negate_val(x_offset)
        } else {
            x_offset
        },
        y_offset: if box_shadow.inset {
            negate_val(y_offset)
        } else {
            y_offset
        },
        spread_radius: if box_shadow.inset {
            negate_val(spread_radius)
        } else {
            spread_radius
        },
        blur_radius,
    }))
}

pub(super) fn has_ui_transform_styles(styles: &BuiStyles) -> bool {
    styles.ui_translation.is_some() || styles.ui_scale.is_some() || styles.ui_rotation.is_some()
}

pub(super) fn insert_ui_transform(
    entity_commands: &mut EntityCommands,
    styles: &BuiStyles,
) -> Result<(), String> {
    let mut ui_transform = UiTransform::default();
    let mut has_ui_transform = false;

    if let Some(value) = &styles.ui_translation {
        ui_transform.translation = parse_val2(value)?;
        has_ui_transform = true;
    }
    if let Some(value) = &styles.ui_scale {
        ui_transform.scale = parse_vec2(value)?;
        has_ui_transform = true;
    }
    if let Some(value) = &styles.ui_rotation {
        ui_transform.rotation = parse_rotation(value)?;
        has_ui_transform = true;
    }

    if has_ui_transform {
        entity_commands.insert(ui_transform);
    }

    Ok(())
}

pub(super) fn set_val(target: &mut Val, source: &Option<String>) -> Result<(), String> {
    if let Some(value) = source {
        *target = parse_val(value)?;
    }
    Ok(())
}

fn negate_val(value: Val) -> Val {
    match value {
        Val::Px(v) => Val::Px(-v),
        Val::Percent(v) => Val::Percent(-v),
        other => other,
    }
}
