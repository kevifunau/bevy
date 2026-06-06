use super::color::{css_color_mix_with_transparency, css_named_color, is_hex_color, oklch_to_hex};

pub(crate) fn css_background_fallback_color(value: &str) -> Option<String> {
    let layers = split_css_layers(value);
    if layers.len() <= 1 {
        return None;
    }

    for layer in layers.iter().rev() {
        if let Some(color) = css_simple_color(layer)
            && color != "transparent"
        {
            return Some(color);
        }
    }

    for layer in layers.iter().rev() {
        if let Some(color) = css_gradient_representative_color(layer)
            && color != "transparent"
        {
            return Some(color);
        }
    }

    None
}

pub(crate) fn css_background_base_color(value: &str) -> Option<String> {
    if let Some(color) = css_background_fallback_color(value) {
        return Some(color);
    }

    if css_contains_gradient(value) {
        return None;
    }

    super::color::css_color(value)
}

pub(crate) fn css_contains_gradient(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("linear-gradient(")
        || value.contains("radial-gradient(")
        || value.contains("conic-gradient(")
}

pub(crate) fn css_simple_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(color) = css_color_mix_with_transparency(value) {
        return Some(color);
    }
    if let Some(color) = oklch_to_hex(value) {
        return Some(color);
    }
    if let Some(color) = css_embedded_oklch_color(value) {
        return Some(color);
    }
    if value == "transparent" {
        return Some("transparent".to_string());
    }
    if is_hex_color(value) {
        return Some(value.to_string());
    }
    for token in value
        .split(|character: char| character.is_whitespace() || matches!(character, ',' | '(' | ')'))
    {
        if let Some(color) = oklch_to_hex(token) {
            return Some(color);
        }
        if is_hex_color(token) {
            return Some(token.to_string());
        }
        if let Some(color) = css_named_color(token) {
            return Some(color.to_string());
        }
    }
    css_named_color(value).map(ToString::to_string)
}

pub(crate) fn css_embedded_oklch_color(value: &str) -> Option<String> {
    let start = value.find("oklch(")?;
    let slice = &value[start..];
    let mut depth = 0usize;
    let mut end_index = None;

    for (index, character) in slice.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_index = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }

    let end = end_index?;
    oklch_to_hex(&slice[..=end])
}

pub(crate) fn split_css_layers(value: &str) -> Vec<&str> {
    let mut layers = Vec::new();
    let mut depth: usize = 0;
    let mut start = 0;

    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let layer = value[start..index].trim();
                if !layer.is_empty() {
                    layers.push(layer);
                }
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    let tail = value[start..].trim();
    if !tail.is_empty() {
        layers.push(tail);
    }

    layers
}

pub(crate) fn css_gradient_first_color(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.starts_with("radial-gradient(")
        && !value.starts_with("linear-gradient(")
        && !value.starts_with("conic-gradient(")
    {
        return None;
    }

    let inner = if value.starts_with("radial-gradient(") {
        value.strip_prefix("radial-gradient(")?.strip_suffix(")")
    } else if value.starts_with("linear-gradient(") {
        value.strip_prefix("linear-gradient(")?.strip_suffix(")")
    } else {
        value.strip_prefix("conic-gradient(")?.strip_suffix(")")
    };
    let Some(inner) = inner else {
        return None;
    };

    let mut depth = 0;
    let mut token_start = 0;
    let mut tokens: Vec<&str> = Vec::new();
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                if i > token_start {
                    tokens.push(inner[token_start..i].trim());
                }
                token_start = i + 1;
            }
            _ => {}
        }
    }
    if token_start < inner.len() {
        tokens.push(inner[token_start..].trim());
    }

    for token in &tokens {
        let stripped = token.trim();
        if stripped.starts_with("oklch(")
            || stripped.starts_with("#")
            || stripped.starts_with("rgb(")
            || stripped.starts_with("rgba(")
        {
            return super::color::css_color(stripped);
        }
        if let Some(hex) = css_named_color(stripped) {
            return Some(hex.to_string());
        }
        if stripped.contains("oklch(") || stripped.contains("#") || stripped.contains("rgb(") {
            for sub in stripped.split(|c: char| c.is_whitespace()) {
                if let Some(color) = super::color::css_color(sub) {
                    return Some(color);
                }
            }
        }
    }

    None
}

fn css_gradient_representative_color(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.contains("-gradient(") {
        return None;
    }

    let gradient_color_stops = split_css_layers(value.split_once('(')?.1.strip_suffix(')')?.trim());

    let mut colors = Vec::new();
    for stop in gradient_color_stops {
        if let Some(color) = css_simple_color(stop)
            && color != "transparent"
        {
            colors.push(color);
        }
    }

    colors
        .last()
        .cloned()
        .or_else(|| css_gradient_first_color(value))
}

pub(crate) fn css_background_image_url(value: &str) -> Option<String> {
    let value = value.trim();
    let url_start = value.find("url(")?;
    let rest = &value[url_start + 4..];
    let url_end = rest.find(')')?;
    let raw = rest[..url_end].trim().trim_matches('"').trim_matches('\'');
    (!raw.is_empty()).then(|| raw.to_string())
}

pub(crate) fn css_aspect_ratio(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some((left, right)) = value.split_once('/') {
        let left = left.trim().parse::<f32>().ok()?;
        let right = right.trim().parse::<f32>().ok()?;
        if right == 0.0 {
            return None;
        }
        return Some((left / right).to_string());
    }

    value.parse::<f32>().ok().map(|ratio| ratio.to_string())
}
