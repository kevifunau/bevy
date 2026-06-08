use bevy_color::{Color, Srgba};
use bevy_math::{Rot2, Vec2};
use bevy_text::{Justify, LineBreak, LineHeight};
use bevy_ui::prelude::*;

use crate::core::style::css_sizing::{css_eval_length_function, css_size_to_px};

use super::normalize_token;

pub(crate) fn parse_number(value: &str) -> Result<f32, String> {
    value
        .trim()
        .parse::<f32>()
        .map_err(|error| format!("Invalid number '{}': {error}", value.trim()))
}

pub(crate) fn parse_integer(value: &str) -> Result<i32, String> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|error| format!("Invalid integer '{}': {error}", value.trim()))
}

pub(crate) fn parse_val(value: &str) -> Result<Val, String> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("auto") {
        return Ok(Val::Auto);
    }

    if let Some(number) = value.strip_suffix("px") {
        return parse_number(number).map(Val::Px);
    }

    if let Some(number) = value.strip_suffix('%') {
        return parse_number(number).map(Val::Percent);
    }

    if value.ends_with("vw") || value.ends_with("vh") {
        return css_size_to_px(value)
            .map(Val::Px)
            .ok_or_else(|| format!("Invalid viewport length '{value}'."));
    }

    if let Some(px) = css_eval_length_function(value)
        .and_then(|resolved| resolved.strip_suffix("px").map(str::to_string))
    {
        return parse_number(&px).map(Val::Px);
    }

    parse_number(value).map(Val::Px)
}

pub(crate) fn parse_ui_rect(value: &str) -> Result<UiRect, String> {
    let values = value
        .split_whitespace()
        .map(parse_val)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [all] => Ok(UiRect::all(*all)),
        [vertical, horizontal] => Ok(UiRect::axes(*horizontal, *vertical)),
        [top, horizontal, bottom] => Ok(UiRect {
            left: *horizontal,
            right: *horizontal,
            top: *top,
            bottom: *bottom,
        }),
        [top, right, bottom, left] => Ok(UiRect {
            left: *left,
            right: *right,
            top: *top,
            bottom: *bottom,
        }),
        _ => Err(format!("Invalid UiRect shorthand '{value}'.")),
    }
}

pub(crate) fn parse_val2(value: &str) -> Result<Val2, String> {
    let values = value
        .split_whitespace()
        .map(parse_val)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [x, y] => Ok(Val2::new(*x, *y)),
        _ => Err(format!(
            "Invalid UiTransform translation '{value}'. Expected two values."
        )),
    }
}

pub(crate) fn parse_vec2(value: &str) -> Result<Vec2, String> {
    let values = value
        .split_whitespace()
        .map(parse_number)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [all] => Ok(Vec2::splat(*all)),
        [x, y] => Ok(Vec2::new(*x, *y)),
        _ => Err(format!(
            "Invalid UiTransform scale '{value}'. Expected one or two numbers."
        )),
    }
}

pub(crate) fn parse_rotation(value: &str) -> Result<Rot2, String> {
    let value = value.trim();

    if let Some(degrees) = value.strip_suffix("deg") {
        return parse_number(degrees).map(Rot2::degrees);
    }

    if let Some(radians) = value.strip_suffix("rad") {
        return parse_number(radians).map(Rot2::radians);
    }

    parse_number(value).map(Rot2::radians)
}

pub(crate) fn parse_color(value: &str) -> Result<Color, String> {
    if value.eq_ignore_ascii_case("transparent") {
        return Ok(Color::NONE);
    }

    Srgba::hex(value)
        .map(Color::from)
        .map_err(|error| format!("Invalid color '{value}': {error}"))
}

pub(crate) fn parse_linebreak(value: &str) -> Result<LineBreak, String> {
    match normalize_token(value).as_str() {
        "word_boundary" => Ok(LineBreak::WordBoundary),
        "any_character" => Ok(LineBreak::AnyCharacter),
        "word_or_character" => Ok(LineBreak::WordOrCharacter),
        "no_wrap" => Ok(LineBreak::NoWrap),
        _ => Err(format!("Invalid text_config.linebreak '{value}'.")),
    }
}

pub(crate) fn parse_text_line_height(value: &str) -> Result<LineHeight, String> {
    let value = value.trim();
    if let Some(px) = value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
    {
        return Ok(LineHeight::Px(px));
    }

    let scale = value
        .parse::<f32>()
        .map_err(|_| format!("Invalid text_config.line_height '{value}'."))?;
    if scale <= 0.0 {
        return Err("text_config.line_height must be greater than 0.".to_string());
    }

    Ok(LineHeight::RelativeToFont(scale))
}

pub(crate) fn parse_text_justify(value: &str) -> Result<Justify, String> {
    match normalize_token(value).as_str() {
        "left" => Ok(Justify::Left),
        "center" => Ok(Justify::Center),
        "right" => Ok(Justify::Right),
        "justified" | "justify" => Ok(Justify::Justified),
        "start" => Ok(Justify::Start),
        "end" => Ok(Justify::End),
        _ => Err(format!(
            "Invalid text justify '{value}'. Supported values are left, center, right, justified, start, end."
        )),
    }
}
