use bevy_camera::visibility::Visibility;
use bevy_input_focus::tab_navigation::TabGroup;
use bevy_ui::prelude::*;

use super::{normalize_token, parse_integer, parse_number, parse_val};

pub(crate) fn parse_display(value: &str) -> Result<Display, String> {
    match normalize_token(value).as_str() {
        "flex" => Ok(Display::Flex),
        "grid" => Ok(Display::Grid),
        "block" => Ok(Display::Block),
        "none" => Ok(Display::None),
        _ => Err(format!("Invalid display '{value}'.")),
    }
}

pub(crate) fn parse_visibility(value: &str) -> Result<Visibility, String> {
    match normalize_token(value).as_str() {
        "inherited" => Ok(Visibility::Inherited),
        "visible" => Ok(Visibility::Visible),
        "hidden" => Ok(Visibility::Hidden),
        _ => Err(format!("Invalid visibility '{value}'.")),
    }
}

pub(crate) fn parse_position_type(value: &str) -> Result<PositionType, String> {
    match normalize_token(value).as_str() {
        "relative" => Ok(PositionType::Relative),
        "absolute" => Ok(PositionType::Absolute),
        _ => Err(format!("Invalid position_type '{value}'.")),
    }
}

pub(crate) fn parse_flex_direction(value: &str) -> Result<FlexDirection, String> {
    match normalize_token(value).as_str() {
        "row" => Ok(FlexDirection::Row),
        "row_reverse" => Ok(FlexDirection::RowReverse),
        "column" => Ok(FlexDirection::Column),
        "column_reverse" => Ok(FlexDirection::ColumnReverse),
        _ => Err(format!("Invalid flex_direction '{value}'.")),
    }
}

pub(crate) fn parse_flex_wrap(value: &str) -> Result<FlexWrap, String> {
    match normalize_token(value).as_str() {
        "no_wrap" | "nowrap" => Ok(FlexWrap::NoWrap),
        "wrap" => Ok(FlexWrap::Wrap),
        "wrap_reverse" => Ok(FlexWrap::WrapReverse),
        _ => Err(format!("Invalid flex_wrap '{value}'.")),
    }
}

pub(crate) fn parse_justify_content(value: &str) -> Result<JustifyContent, String> {
    match normalize_token(value).as_str() {
        "flex_start" | "start" => Ok(JustifyContent::FlexStart),
        "flex_end" | "end" => Ok(JustifyContent::FlexEnd),
        "center" => Ok(JustifyContent::Center),
        "space_between" => Ok(JustifyContent::SpaceBetween),
        "space_around" => Ok(JustifyContent::SpaceAround),
        "space_evenly" => Ok(JustifyContent::SpaceEvenly),
        _ => Err(format!("Invalid justify_content '{value}'.")),
    }
}

pub(crate) fn parse_justify_items(value: &str) -> Result<JustifyItems, String> {
    match normalize_token(value).as_str() {
        "default" => Ok(JustifyItems::Default),
        "start" => Ok(JustifyItems::Start),
        "end" => Ok(JustifyItems::End),
        "center" => Ok(JustifyItems::Center),
        "baseline" => Ok(JustifyItems::Baseline),
        "stretch" => Ok(JustifyItems::Stretch),
        _ => Err(format!("Invalid justify_items '{value}'.")),
    }
}

pub(crate) fn parse_align_content(value: &str) -> Result<AlignContent, String> {
    match normalize_token(value).as_str() {
        "default" => Ok(AlignContent::Default),
        "flex_start" | "start" => Ok(AlignContent::FlexStart),
        "flex_end" | "end" => Ok(AlignContent::FlexEnd),
        "center" => Ok(AlignContent::Center),
        "stretch" => Ok(AlignContent::Stretch),
        "space_between" => Ok(AlignContent::SpaceBetween),
        "space_around" => Ok(AlignContent::SpaceAround),
        "space_evenly" => Ok(AlignContent::SpaceEvenly),
        _ => Err(format!("Invalid align_content '{value}'.")),
    }
}

pub(crate) fn parse_align_items(value: &str) -> Result<AlignItems, String> {
    match normalize_token(value).as_str() {
        "default" => Ok(AlignItems::Default),
        "flex_start" | "start" => Ok(AlignItems::FlexStart),
        "flex_end" | "end" => Ok(AlignItems::FlexEnd),
        "center" => Ok(AlignItems::Center),
        "baseline" => Ok(AlignItems::Baseline),
        "stretch" => Ok(AlignItems::Stretch),
        _ => Err(format!("Invalid align_items '{value}'.")),
    }
}

pub(crate) fn parse_align_self(value: &str) -> Result<AlignSelf, String> {
    match normalize_token(value).as_str() {
        "auto" => Ok(AlignSelf::Auto),
        "flex_start" | "start" => Ok(AlignSelf::FlexStart),
        "flex_end" | "end" => Ok(AlignSelf::FlexEnd),
        "center" => Ok(AlignSelf::Center),
        "baseline" => Ok(AlignSelf::Baseline),
        "stretch" => Ok(AlignSelf::Stretch),
        _ => Err(format!("Invalid align_self '{value}'.")),
    }
}

pub(crate) fn parse_justify_self(value: &str) -> Result<JustifySelf, String> {
    match normalize_token(value).as_str() {
        "auto" => Ok(JustifySelf::Auto),
        "start" => Ok(JustifySelf::Start),
        "end" => Ok(JustifySelf::End),
        "center" => Ok(JustifySelf::Center),
        "baseline" => Ok(JustifySelf::Baseline),
        "stretch" => Ok(JustifySelf::Stretch),
        _ => Err(format!("Invalid justify_self '{value}'.")),
    }
}

pub(crate) fn parse_overflow(value: &str) -> Result<Overflow, String> {
    match normalize_token(value).as_str() {
        "visible" => Ok(Overflow::visible()),
        "clip" => Ok(Overflow::clip()),
        "clip_x" => Ok(Overflow::clip_x()),
        "clip_y" => Ok(Overflow::clip_y()),
        "hidden" => Ok(Overflow::hidden()),
        "hidden_x" => Ok(Overflow::hidden_x()),
        "hidden_y" => Ok(Overflow::hidden_y()),
        "scroll" => Ok(Overflow::scroll()),
        "scroll_x" => Ok(Overflow::scroll_x()),
        "scroll_y" => Ok(Overflow::scroll_y()),
        _ => Err(format!("Invalid overflow '{value}'.")),
    }
}

pub(crate) fn parse_overflow_clip_margin(value: &str) -> Result<OverflowClipMargin, String> {
    let normalized = value.trim().replace(' ', "");

    if normalized.eq_ignore_ascii_case("border_box") {
        return Ok(OverflowClipMargin::border_box());
    }
    if normalized.eq_ignore_ascii_case("padding_box") {
        return Ok(OverflowClipMargin::padding_box());
    }
    if normalized.eq_ignore_ascii_case("content_box") {
        return Ok(OverflowClipMargin::content_box());
    }

    if let Some(argument) = normalized
        .strip_prefix("border_box(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return parse_number(argument)
            .map(|margin| OverflowClipMargin::border_box().with_margin(margin));
    }
    if let Some(argument) = normalized
        .strip_prefix("padding_box(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return parse_number(argument)
            .map(|margin| OverflowClipMargin::padding_box().with_margin(margin));
    }
    if let Some(argument) = normalized
        .strip_prefix("content_box(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return parse_number(argument)
            .map(|margin| OverflowClipMargin::content_box().with_margin(margin));
    }

    Err(format!("Invalid overflow_clip_margin '{value}'."))
}

pub(crate) fn parse_tab_group(value: &str) -> Result<TabGroup, String> {
    let normalized = normalize_token(value);
    if normalized == "modal" {
        return Ok(TabGroup::modal());
    }

    parse_integer(value).map(TabGroup::new)
}

pub(crate) fn parse_border_radius(value: &str) -> Result<BorderRadius, String> {
    Ok(BorderRadius::all(parse_val(value)?))
}
