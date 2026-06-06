use crate::core::support::viewport::current_opendesign_viewport;

pub(crate) fn css_size_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for character in value.chars() {
        match character {
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            character if character.is_ascii_whitespace() && depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}

pub(crate) fn is_simple_css_size(value: &str) -> bool {
    let value = value.trim();
    if value == "auto" || value == "0" {
        return true;
    }
    value
        .strip_suffix("px")
        .or_else(|| value.strip_suffix('%'))
        .or_else(|| value.strip_suffix("vw"))
        .or_else(|| value.strip_suffix("vh"))
        .is_some_and(|number| number.parse::<f32>().is_ok())
}

pub(crate) fn css_first_size(value: &str) -> Option<String> {
    if let Some(size) = css_eval_length_function(value) {
        return Some(size);
    }
    css_size_tokens(value)
        .into_iter()
        .find_map(|part| css_length_to_bui_val(&part))
}

pub(crate) fn css_size_to_px(value: &str) -> Option<f32> {
    let viewport = current_opendesign_viewport();
    let value = value.trim();
    if let Some(px) = value.strip_suffix("px") {
        px.parse::<f32>().ok()
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|percent| viewport.width * percent / 100.0)
    } else if let Some(vw) = value.strip_suffix("vw") {
        vw.parse::<f32>()
            .ok()
            .map(|vw| viewport.width * vw / 100.0)
    } else if let Some(vh) = value.strip_suffix("vh") {
        vh.parse::<f32>()
            .ok()
            .map(|vh| viewport.height * vh / 100.0)
    } else {
        value.parse::<f32>().ok()
    }
}

pub(crate) fn css_length_to_bui_val(value: &str) -> Option<String> {
    let value = value.trim();
    if value == "auto" || value == "0" || value.ends_with("px") || value.ends_with('%') {
        return Some(value.to_string());
    }
    if value.ends_with("vw") || value.ends_with("vh") {
        return css_size_to_px(value).map(format_css_px);
    }
    None
}

pub(crate) fn css_eval_length_function(value: &str) -> Option<String> {
    let value = value.trim();
    let (name, args) = css_function_call(value)?;
    let args = split_css_function_args(args);
    match name {
        "min" => args
            .iter()
            .filter_map(|arg| css_size_to_px(arg))
            .reduce(f32::min)
            .map(format_css_px),
        "max" => args
            .iter()
            .filter_map(|arg| css_size_to_px(arg))
            .reduce(f32::max)
            .map(format_css_px),
        "clamp" if args.len() == 3 => {
            let min = css_size_to_px(args[0])?;
            let preferred = css_size_to_px(args[1])?;
            let max = css_size_to_px(args[2])?;
            Some(format_css_px(preferred.clamp(min, max)))
        }
        _ => None,
    }
}

pub(crate) fn css_function_call(value: &str) -> Option<(&str, &str)> {
    let (name, rest) = value.split_once('(')?;
    let args = rest.strip_suffix(')')?;
    Some((name.trim(), args.trim()))
}

pub(crate) fn split_css_function_args(value: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(value[start..].trim());
    args.into_iter().filter(|arg| !arg.is_empty()).collect()
}

pub(crate) fn format_css_px(value: f32) -> String {
    if (value.fract()).abs() < f32::EPSILON {
        format!("{}px", value as i32)
    } else {
        let number = format!("{value:.2}");
        let number = number.trim_end_matches('0').trim_end_matches('.');
        format!("{number}px")
    }
}
