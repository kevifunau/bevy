use super::background::{
    css_background_fallback_color, css_embedded_oklch_color, css_gradient_first_color,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct CssFilterColorAdjustment {
    pub(crate) brightness: f32,
    pub(crate) contrast: f32,
    pub(crate) saturate: f32,
}

pub(crate) fn css_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(color) = css_background_fallback_color(value) {
        return Some(color);
    }
    if let Some(color) = css_color_mix_with_transparency(value) {
        return Some(color);
    }
    if let Some(color) = oklch_to_hex(value) {
        return Some(color);
    }
    if let Some(color) = css_embedded_oklch_color(value) {
        return Some(color);
    }
    if let Some(color) = css_gradient_first_color(value) {
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
        if token.eq_ignore_ascii_case("transparent") {
            return Some("transparent".to_string());
        }
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

pub(crate) fn css_function_tokens(value: &str) -> Vec<&str> {
    value
        .split(|character: char| character.is_whitespace() || matches!(character, ',' | '(' | ')'))
        .filter(|token| !token.is_empty() && *token != "in" && *token != "oklab")
        .collect()
}

pub(crate) fn css_named_color(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "black" => Some("#000000"),
        "white" => Some("#FFFFFF"),
        "red" => Some("#FF0000"),
        _ => None,
    }
}

pub(crate) fn css_multiply_blend_fallback_color(value: &str) -> Option<String> {
    let hex = value.trim().strip_prefix('#')?;
    let (r, g, b, a) = parse_hex_channels(hex)?;

    let is_cool_tinted = b > g && b > r;
    let (r, g, b, alpha) = if is_cool_tinted {
        let darken = |channel: u8, factor: f32| {
            ((channel as f32) * factor).round().clamp(0.0, 255.0) as u8
        };
        (
            darken(r, 0.72),
            darken(g, 0.82),
            darken(b, 0.9),
            ((a as f32) * 0.94).round().clamp(0.0, 255.0) as u8,
        )
    } else {
        let darken = |channel: u8| ((channel as f32) * 0.78).round().clamp(0.0, 255.0) as u8;
        (
            darken(r),
            darken(g),
            darken(b),
            ((a as f32) * 0.88).round().clamp(0.0, 255.0) as u8,
        )
    };

    if alpha == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, alpha))
    }
}

pub(crate) fn css_adjust_filter_color(
    value: &str,
    adjustment: CssFilterColorAdjustment,
) -> Option<String> {
    let (mut r, mut g, mut b, a) = css_hex_rgba(value)?;

    let apply_channel = |channel: f32| {
        let contrasted = ((channel - 0.5) * adjustment.contrast + 0.5).clamp(0.0, 1.0);
        (contrasted * adjustment.brightness).clamp(0.0, 1.0)
    };

    r = apply_channel(r);
    g = apply_channel(g);
    b = apply_channel(b);

    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    r = (luminance + (r - luminance) * adjustment.saturate).clamp(0.0, 1.0);
    g = (luminance + (g - luminance) * adjustment.saturate).clamp(0.0, 1.0);
    b = (luminance + (b - luminance) * adjustment.saturate).clamp(0.0, 1.0);

    css_rgba_to_hex(r, g, b, a)
}

pub(crate) fn css_hex_rgba(value: &str) -> Option<(f32, f32, f32, f32)> {
    let hex = value.trim().strip_prefix('#')?;
    let (r, g, b, a) = parse_hex_channels(hex)?;

    Some((
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

pub(crate) fn blend_hex_colors(color_a: &str, color_b: &str, ratio: f32) -> Option<String> {
    let (r_a, g_a, b_a, a_a) = css_hex_rgba(color_a)?;
    let (r_b, g_b, b_b, a_b) = css_hex_rgba(color_b)?;
    let t = ratio.clamp(0.0, 1.0);
    let r = ((r_a * (1.0 - t) + r_b * t) * 255.0).round() as u8;
    let g = ((g_a * (1.0 - t) + g_b * t) * 255.0).round() as u8;
    let b = ((b_a * (1.0 - t) + b_b * t) * 255.0).round() as u8;
    let a = ((a_a * (1.0 - t) + a_b * t) * 255.0).round() as u8;
    if a == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }
}

pub(crate) fn scale_hex_alpha(color: &str, factor: f32) -> Option<String> {
    let (r, g, b, a) = css_hex_rgba(color)?;
    css_rgba_to_hex(r, g, b, a * factor.clamp(0.0, 1.0))
}

pub(crate) fn css_rgba_to_hex(r: f32, g: f32, b: f32, a: f32) -> Option<String> {
    let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (a.clamp(0.0, 1.0) * 255.0).round() as u8;

    if a == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }
}

pub(crate) fn css_percentage_value(value: &str) -> Option<f32> {
    value
        .trim()
        .strip_suffix('%')?
        .parse::<f32>()
        .ok()
        .map(|value| value / 100.0)
}

pub(crate) fn append_hex_alpha(color: &str, alpha_percent: f32) -> Option<String> {
    let hex = color.trim().strip_prefix('#')?;
    let rgb = match hex.len() {
        3 | 4 => hex
            .chars()
            .take(3)
            .flat_map(|character| [character, character])
            .collect::<String>(),
        6 | 8 => hex.chars().take(6).collect::<String>(),
        _ => return None,
    };
    let alpha = ((alpha_percent.clamp(0.0, 100.0) / 100.0) * 255.0).round() as u8;
    Some(format!("#{}{alpha:02X}", rgb.to_ascii_uppercase()))
}

pub(super) fn css_color_mix_with_transparency(value: &str) -> Option<String> {
    if !value.trim_start().starts_with("color-mix(") || !value.contains("transparent") {
        return None;
    }

    let tokens = css_function_tokens(value);
    let transparent_index = tokens.iter().position(|token| *token == "transparent")?;
    let transparent_percent = tokens
        .get(transparent_index + 1)
        .and_then(|token| token.strip_suffix('%'))
        .and_then(|percent| percent.parse::<f32>().ok())
        .unwrap_or(50.0)
        .clamp(0.0, 100.0);

    let base_color = tokens
        .iter()
        .take(transparent_index)
        .find_map(|token| {
            if is_hex_color(token) {
                Some((*token).to_string())
            } else {
                css_named_color(token).map(ToString::to_string)
            }
        })
        .or_else(|| {
            tokens.iter().skip(transparent_index + 1).find_map(|token| {
                if is_hex_color(token) {
                    Some((*token).to_string())
                } else {
                    css_named_color(token).map(ToString::to_string)
                }
            })
        })?;

    append_hex_alpha(&base_color, 100.0 - transparent_percent)
}

pub(super) fn is_hex_color(value: &str) -> bool {
    let value = value.trim();
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|character| character.is_ascii_hexdigit())
}

pub(super) fn oklch_to_hex(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.starts_with("oklch(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.strip_prefix("oklch(")?.strip_suffix(")")?;

    let parts: Vec<&str> = inner
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();

    if parts.len() < 3 {
        return None;
    }

    let l_raw = parts[0];
    let l = l_raw
        .strip_suffix('%')
        .and_then(|s| s.parse::<f32>().ok())
        .map(|v| v / 100.0)
        .or_else(|| l_raw.parse::<f32>().ok())?;
    let c: f32 = parts[1].parse::<f32>().ok()?;
    let h: f32 = parts[2].parse::<f32>().ok()?;

    let alpha = if parts.len() >= 5 && parts[3] == "/" {
        parts[4].parse::<f32>().ok()
    } else if parts.len() >= 4 && parts[3].starts_with('/') {
        let raw = parts[3].strip_prefix('/')?;
        if raw.is_empty() {
            parts.get(4).and_then(|value| value.parse::<f32>().ok())
        } else {
            raw.parse::<f32>().ok()
        }
    } else if parts.len() >= 5 {
        parts[4].parse::<f32>().ok()
    } else {
        Some(1.0)
    };

    let alpha = alpha?;

    let h_rad = h * std::f32::consts::PI / 180.0;
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l_c = l_ * l_ * l_;
    let m_c = m_ * m_ * m_;
    let s_c = s_ * s_ * s_;

    let x = 1.2268798737 * l_c - 0.5556238332 * m_c + 0.2811894837 * s_c;
    let y = -0.0405757626 * l_c + 1.1971573648 * m_c - 0.1560437476 * s_c;
    let z = -0.0753452638 * l_c + 0.2413318055 * m_c + 1.8340138286 * s_c;

    let r_lin = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g_lin = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let b_lin = 0.0556432 * x - 0.2040259 * y + 1.0572252 * z;

    let r = srgb_gamma(r_lin);
    let g = srgb_gamma(g_lin);
    let b = srgb_gamma(b_lin);

    let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;

    if a == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }
}

fn parse_hex_channels(hex: &str) -> Option<(u8, u8, u8, u8)> {
    match hex.len() {
        3 => {
            let mut characters = hex.chars();
            let r = characters.next()?;
            let g = characters.next()?;
            let b = characters.next()?;
            Some((
                u8::from_str_radix(&format!("{r}{r}"), 16).ok()?,
                u8::from_str_radix(&format!("{g}{g}"), 16).ok()?,
                u8::from_str_radix(&format!("{b}{b}"), 16).ok()?,
                255,
            ))
        }
        4 => {
            let mut characters = hex.chars();
            let r = characters.next()?;
            let g = characters.next()?;
            let b = characters.next()?;
            let a = characters.next()?;
            Some((
                u8::from_str_radix(&format!("{r}{r}"), 16).ok()?,
                u8::from_str_radix(&format!("{g}{g}"), 16).ok()?,
                u8::from_str_radix(&format!("{b}{b}"), 16).ok()?,
                u8::from_str_radix(&format!("{a}{a}"), 16).ok()?,
            ))
        }
        6 => Some((
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255,
        )),
        8 => Some((
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        )),
        _ => None,
    }
}

fn srgb_gamma(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
