use crate::core::style::css_sizing::{
    css_eval_length_function, css_first_size, css_size_tokens, is_simple_css_size,
    split_css_function_args,
};

pub(super) fn normalize_css_value(value: &str) -> String {
    value
        .trim()
        .trim_end_matches("!important")
        .trim()
        .trim_matches('"')
        .replace("  ", " ")
        .replace("solid ", "")
}

pub(super) fn set_simple_css_val(target: &mut Option<String>, value: &str) {
    if let Some(size) = css_eval_length_function(value) {
        *target = Some(size);
    } else if is_simple_css_size(value) {
        *target = Some(value.to_string());
    } else if let Some(size) = css_first_size(value) {
        *target = Some(size);
    }
}

pub(super) fn set_css_rect(target: &mut Option<String>, value: &str) {
    let normalized = css_size_tokens(value)
        .into_iter()
        .filter_map(|part| {
            if let Some(size) = css_eval_length_function(&part) {
                Some(size)
            } else if is_simple_css_size(&part) {
                Some(part)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if !normalized.is_empty() {
        *target = Some(normalized);
    }
}

pub(super) fn css_display(value: &str) -> Option<&'static str> {
    match value {
        "flex" | "inline-flex" => Some("flex"),
        "grid" | "inline-grid" => Some("grid"),
        "none" => Some("none"),
        _ => None,
    }
}

pub(super) fn css_overflow(value: &str) -> Option<&'static str> {
    match value {
        "visible" => Some("visible"),
        "hidden" | "clip" => Some("clip"),
        "auto" | "scroll" => Some("scroll_y"),
        _ => None,
    }
}

pub(super) fn css_grid_tracks(value: &str) -> Option<String> {
    let value = value.trim();
    match value {
        "minmax(0, 1fr) auto" => Some("flex(1) auto".to_string()),
        "minmax(0, 1fr) 140px" => Some("flex(1) px(140)".to_string()),
        "92px minmax(0, 1fr)" => Some("px(92) flex(1)".to_string()),
        "104px minmax(0, 1fr)" => Some("px(104) flex(1)".to_string()),
        "84px minmax(0, 1fr)" => Some("px(84) flex(1)".to_string()),
        "repeat(4, minmax(0, 1fr))" => Some("flex(4, 1)".to_string()),
        _ => {
            let tracks = split_grid_track_tokens(value)?;
            let mut converted = Vec::new();
            for track in tracks {
                converted.push(css_grid_track_token_to_bui(&track)?);
            }
            Some(converted.join(" "))
        }
    }
}

fn split_grid_track_tokens(value: &str) -> Option<Vec<String>> {
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
                if depth == 0 {
                    return None;
                }
                depth -= 1;
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
    if depth != 0 {
        return None;
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }
    (!tokens.is_empty()).then_some(tokens)
}

fn css_grid_track_token_to_bui(value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some("auto".to_string());
    }
    if value.eq_ignore_ascii_case("min-content") {
        return Some("min_content".to_string());
    }
    if value.eq_ignore_ascii_case("max-content") {
        return Some("max_content".to_string());
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.parse::<f32>().ok().map(|_| format!("px({px})"));
    }
    if let Some(fr) = value.strip_suffix("fr") {
        let fr = fr.trim();
        let fraction = if fr.is_empty() { "1" } else { fr };
        return fraction
            .parse::<f32>()
            .ok()
            .map(|_| format!("flex({fraction})"));
    }
    if let Some(content) = value
        .strip_prefix("minmax(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let max = args[1].trim();
            if let Some(fr) = max.strip_suffix("fr") {
                let fr = fr.trim();
                let fraction = if fr.is_empty() { "1" } else { fr };
                return fraction
                    .parse::<f32>()
                    .ok()
                    .map(|_| format!("flex({fraction})"));
            }
            return css_grid_track_token_to_bui(max);
        }
        return None;
    }
    if let Some(content) = value
        .strip_prefix("repeat(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let repetition = args[0].trim().parse::<u16>().ok()?;
            return css_grid_track_token_to_bui_repeat(repetition, args[1].trim());
        }
        return None;
    }
    None
}

fn css_grid_track_token_to_bui_repeat(repetition: u16, value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(format!("auto({repetition})"));
    }
    if value.eq_ignore_ascii_case("min-content") {
        return Some(format!("min_content({repetition})"));
    }
    if value.eq_ignore_ascii_case("max-content") {
        return Some(format!("max_content({repetition})"));
    }
    if let Some(px) = value.strip_suffix("px") {
        return px
            .parse::<f32>()
            .ok()
            .map(|_| format!("px({repetition}, {px})"));
    }
    if let Some(fr) = value.strip_suffix("fr") {
        let fr = fr.trim();
        let fraction = if fr.is_empty() { "1" } else { fr };
        return fraction
            .parse::<f32>()
            .ok()
            .map(|_| format!("flex({repetition}, {fraction})"));
    }
    if let Some(content) = value
        .strip_prefix("minmax(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let max = args[1].trim();
            if let Some(fr) = max.strip_suffix("fr") {
                let fr = fr.trim();
                let fraction = if fr.is_empty() { "1" } else { fr };
                return fraction
                    .parse::<f32>()
                    .ok()
                    .map(|_| format!("flex({repetition}, {fraction})"));
            }
            if let Some(px) = max.strip_suffix("px") {
                return px
                    .parse::<f32>()
                    .ok()
                    .map(|_| format!("px({repetition}, {px})"));
            }
        }
    }
    None
}
