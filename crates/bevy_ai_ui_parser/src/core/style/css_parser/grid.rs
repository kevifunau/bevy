use bevy_ui::prelude::*;

use super::parse_number;

pub(crate) fn parse_grid_tracks(value: &str) -> Result<Vec<RepeatedGridTrack>, String> {
    let declarations = split_grid_track_declarations(value)?;

    declarations
        .into_iter()
        .map(|declaration| parse_single_grid_track(&declaration))
        .collect()
}

fn split_grid_track_declarations(value: &str) -> Result<Vec<String>, String> {
    let mut declarations = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for character in value.chars() {
        match character {
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                if depth == 0 {
                    return Err(format!("Invalid grid track declaration '{value}'."));
                }
                depth -= 1;
                current.push(character);
            }
            ',' | ' ' | '\n' | '\t' if depth == 0 => {
                if !current.trim().is_empty() {
                    declarations.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if depth != 0 {
        return Err(format!("Invalid grid track declaration '{value}'."));
    }

    if !current.trim().is_empty() {
        declarations.push(current.trim().to_string());
    }

    if declarations.is_empty() {
        return Err(format!("Invalid grid track declaration '{value}'."));
    }

    Ok(declarations)
}

fn parse_single_grid_track(value: &str) -> Result<RepeatedGridTrack, String> {
    let normalized = value.trim().replace(' ', "");

    if normalized.eq_ignore_ascii_case("auto") {
        return Ok(RepeatedGridTrack::auto(1));
    }

    if normalized.eq_ignore_ascii_case("min_content") {
        return Ok(RepeatedGridTrack::min_content(1));
    }

    if normalized.eq_ignore_ascii_case("max_content") {
        return Ok(RepeatedGridTrack::max_content(1));
    }

    if let Some(argument) = normalized
        .strip_prefix("px(")
        .and_then(|value| value.strip_suffix(')'))
    {
        if let Some((repetition, px_value)) = argument.split_once(',') {
            let repetition = repetition
                .parse::<u16>()
                .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
            let px_value = parse_number(px_value)?;
            return Ok(RepeatedGridTrack::px(repetition, px_value));
        }

        return parse_number(argument).map(|px_value| RepeatedGridTrack::px(1, px_value));
    }

    if let Some(argument) = normalized
        .strip_prefix("auto(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let repetition = argument
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
        return Ok(RepeatedGridTrack::auto(repetition));
    }

    if let Some(argument) = normalized
        .strip_prefix("fr(")
        .and_then(|value| value.strip_suffix(')'))
    {
        if argument.contains(',') {
            let (repetition, fraction) = parse_two_grid_args(argument, value)?;
            return Ok(RepeatedGridTrack::fr(repetition, fraction));
        }

        return parse_number(argument).map(|fraction| RepeatedGridTrack::fr(1, fraction));
    }

    if let Some(args) = normalized
        .strip_prefix("flex(")
        .and_then(|value| value.strip_suffix(')'))
    {
        if args.contains(',') {
            let (repetition, fraction) = parse_two_grid_args(args, value)?;
            return Ok(RepeatedGridTrack::flex(repetition, fraction));
        }

        return parse_number(args).map(|fraction| RepeatedGridTrack::flex(1, fraction));
    }

    if let Some(args) = normalized
        .strip_prefix("min_content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let repetition = args
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
        return Ok(RepeatedGridTrack::min_content(repetition));
    }

    if let Some(args) = normalized
        .strip_prefix("max_content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let repetition = args
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
        return Ok(RepeatedGridTrack::max_content(repetition));
    }

    Err(format!("Invalid grid track declaration '{value}'."))
}

fn parse_two_grid_args(args: &str, original: &str) -> Result<(u16, f32), String> {
    let Some((repetition, value)) = args.split_once(',') else {
        return Err(format!("Invalid grid track declaration '{original}'."));
    };

    let repetition = repetition
        .parse::<u16>()
        .map_err(|error| format!("Invalid grid track repetition in '{original}': {error}"))?;
    let value = parse_number(value)?;

    Ok((repetition, value))
}

pub(crate) fn parse_grid_placement(value: &str) -> Result<GridPlacement, String> {
    let normalized = value.trim().replace(' ', "");

    if normalized.eq_ignore_ascii_case("auto") {
        return Ok(GridPlacement::auto());
    }

    if let Some(span) = normalized
        .strip_prefix("span(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let span = span
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid placement '{value}': {error}"))?;
        return Ok(GridPlacement::span(span));
    }

    Err(format!("Invalid grid placement '{value}'."))
}
