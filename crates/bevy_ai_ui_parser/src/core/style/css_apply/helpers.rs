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
        "block" | "inline-block" => Some("block"),
        "none" => Some("none"),
        _ => None,
    }
}

pub(super) fn css_overflow(value: &str) -> Option<&'static str> {
    match value {
        "visible" => Some("visible"),
        "clip" => Some("clip"),
        "hidden" => Some("hidden"),
        "auto" | "scroll" => Some("scroll"),
        _ => None,
    }
}

pub(super) fn css_overflow_axis(value: &str) -> Option<&'static str> {
    match value {
        "visible" => Some("visible"),
        "clip" => Some("clip"),
        "hidden" => Some("hidden"),
        "auto" | "scroll" => Some("scroll"),
        _ => None,
    }
}

pub(super) fn set_css_overflow_axis(target: &mut Option<String>, axis: CssAxis, value: &str) {
    let Some(axis_value) = css_overflow_axis(value) else {
        return;
    };
    let (mut x, mut y) = target
        .as_deref()
        .and_then(css_overflow_axes)
        .unwrap_or(("visible", "visible"));
    match axis {
        CssAxis::X => x = axis_value,
        CssAxis::Y => y = axis_value,
    }
    *target = Some(css_overflow_pair(x, y).to_string());
}

pub(super) enum CssAxis {
    X,
    Y,
}

fn css_overflow_axes(value: &str) -> Option<(&'static str, &'static str)> {
    match value {
        "visible" => Some(("visible", "visible")),
        "clip" => Some(("clip", "clip")),
        "hidden" => Some(("hidden", "hidden")),
        "scroll" => Some(("scroll", "scroll")),
        "clip_x" => Some(("clip", "visible")),
        "clip_y" => Some(("visible", "clip")),
        "hidden_x" => Some(("hidden", "visible")),
        "hidden_y" => Some(("visible", "hidden")),
        "scroll_x" => Some(("scroll", "visible")),
        "scroll_y" => Some(("visible", "scroll")),
        _ => {
            let mut parts = value.split_whitespace();
            let x = parts.next().and_then(css_overflow_axis)?;
            let y = parts.next().and_then(css_overflow_axis).unwrap_or(x);
            Some((x, y))
        }
    }
}

fn css_overflow_pair(x: &str, y: &str) -> &'static str {
    match (x, y) {
        ("visible", "visible") => "visible",
        ("clip", "clip") => "clip",
        ("hidden", "hidden") => "hidden",
        ("scroll", "scroll") => "scroll",
        ("clip", "visible") => "clip_x",
        ("visible", "clip") => "clip_y",
        ("hidden", "visible") => "hidden_x",
        ("visible", "hidden") => "hidden_y",
        ("scroll", "visible") => "scroll_x",
        ("visible", "scroll") => "scroll_y",
        ("hidden", "scroll") => "hidden scroll",
        ("scroll", "hidden") => "scroll hidden",
        ("clip", "scroll") => "clip scroll",
        ("scroll", "clip") => "scroll clip",
        ("hidden", "clip") => "hidden clip",
        ("clip", "hidden") => "clip hidden",
        _ => "visible",
    }
}

pub(super) fn css_axis_pair(value: &str) -> Option<(String, String)> {
    let sizes = normalized_css_sizes(value);
    match sizes.as_slice() {
        [single] => Some((single.clone(), single.clone())),
        [first, second, ..] => Some((first.clone(), second.clone())),
        _ => None,
    }
}

pub(super) fn css_flex_shorthand(value: &str) -> Option<(String, String, String)> {
    match value {
        "none" => {
            return Some(("0".to_string(), "0".to_string(), "auto".to_string()));
        }
        "auto" => {
            return Some(("1".to_string(), "1".to_string(), "auto".to_string()));
        }
        "initial" => {
            return Some(("0".to_string(), "1".to_string(), "auto".to_string()));
        }
        _ => {}
    }

    let parts = css_size_tokens(value);
    match parts.as_slice() {
        [grow] if grow.parse::<f32>().is_ok() => {
            Some((grow.clone(), "1".to_string(), "0%".to_string()))
        }
        [grow, second] if grow.parse::<f32>().is_ok() => {
            if second.parse::<f32>().is_ok() {
                Some((grow.clone(), second.clone(), "0%".to_string()))
            } else {
                normalized_css_size(second).map(|basis| (grow.clone(), "1".to_string(), basis))
            }
        }
        [grow, shrink, basis] if grow.parse::<f32>().is_ok() && shrink.parse::<f32>().is_ok() => {
            normalized_css_size(basis).map(|basis| (grow.clone(), shrink.clone(), basis))
        }
        _ => None,
    }
}

fn normalized_css_sizes(value: &str) -> Vec<String> {
    css_size_tokens(value)
        .into_iter()
        .filter_map(|part| normalized_css_size(&part))
        .collect()
}

fn normalized_css_size(value: &str) -> Option<String> {
    if let Some(size) = css_eval_length_function(value) {
        Some(size)
    } else if is_simple_css_size(value) {
        Some(value.to_string())
    } else {
        None
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

pub(super) fn css_grid_placement(value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some("auto".to_string());
    }
    if let Some(span) = value.strip_prefix("span").map(str::trim)
        && span.parse::<u16>().ok().is_some_and(|span| span > 0)
    {
        return Some(format!("span({span})"));
    }
    if let Ok(index) = value.parse::<u16>()
        && index > 0
    {
        return Some(format!("start({index})"));
    }
    None
}

pub(super) fn css_stage_fit_mode(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "contain" => Some("contain"),
        "cover" => Some("cover"),
        "fill" | "stretch" => Some("fill"),
        "none" => Some("none"),
        "scale-down" | "scale_down" => Some("scale-down"),
        _ => None,
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
        .strip_prefix("fit-content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let content = content.trim();
        if let Some(px) = content.strip_suffix("px") {
            return px
                .parse::<f32>()
                .ok()
                .map(|_| format!("fit_content_px({px})"));
        }
        if let Some(percent) = content.strip_suffix('%') {
            return percent
                .parse::<f32>()
                .ok()
                .map(|_| format!("fit_content_percent({percent})"));
        }
        return None;
    }
    if let Some(content) = value
        .strip_prefix("repeat(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let repetition_arg = args[0].trim();
            // Support numeric repetition (e.g. repeat(3, 100px)) and auto-fill/auto-fit
            if let Ok(repetition) = repetition_arg.parse::<u16>() {
                return css_grid_track_token_to_bui_repeat(repetition, args[1].trim());
            }
            // For auto-fill/auto-fit, we can't know the count at compile time.
            // Parse the track size and emit it as a single track — Bevy's grid
            // engine will handle auto-fill at runtime if it supports it.
            if repetition_arg.eq_ignore_ascii_case("auto-fill")
                || repetition_arg.eq_ignore_ascii_case("auto-fit")
            {
                return css_grid_track_token_to_bui(args[1].trim());
            }
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
    if let Some(content) = value
        .strip_prefix("fit-content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let content = content.trim();
        if let Some(px) = content.strip_suffix("px") {
            return px
                .parse::<f32>()
                .ok()
                .map(|_| format!("fit_content_px({repetition}, {px})"));
        }
        if let Some(percent) = content.strip_suffix('%') {
            return percent
                .parse::<f32>()
                .ok()
                .map(|_| format!("fit_content_percent({repetition}, {percent})"));
        }
    }
    None
}
