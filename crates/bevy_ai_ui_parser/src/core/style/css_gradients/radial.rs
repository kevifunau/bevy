use crate::core::style::css_values::{css_percentage_value, scale_hex_alpha, split_css_layers};

use super::{
    CssGradientStop,
    SimpleGradientOverlayKind,
    SimpleGradientOverlaySpec,
    shared::SimpleRadialGradientOverlay,
    SimpleRadialGradientRingOverlay,
};

pub(crate) fn css_simple_radial_gradient_overlays(value: &str) -> Vec<SimpleGradientOverlaySpec> {
    let Some(radial) = css_simple_radial_gradient_overlay(value) else {
        return Vec::new();
    };

    let is_tinted_scene_wash = radial.color.starts_with("#3")
        || radial.color.starts_with("#4")
        || radial.color.starts_with("#5")
        || radial.color.starts_with("#B")
        || radial.color.starts_with("#C");
    let is_large_background_like = radial.width > 0.78
        || radial.height > 0.62
        || (is_tinted_scene_wash && (radial.width > 0.48 || radial.height > 0.38));

    let layer_factors: &[(f32, f32)] = if is_large_background_like && is_tinted_scene_wash {
        &[(1.08, 0.022), (0.8, 0.04), (0.56, 0.068)]
    } else if is_large_background_like {
        &[(0.9, 0.08), (0.7, 0.13)]
    } else {
        &[(1.0, 0.28), (0.86, 0.44), (0.72, 0.62), (0.56, 0.84)]
    };

    layer_factors
        .iter()
        .filter_map(|(size_scale, alpha_scale)| {
            let color = scale_hex_alpha(&radial.color, *alpha_scale)?;
            let width = (radial.width * *size_scale).clamp(0.12, 1.35);
            let height = (radial.height * *size_scale).clamp(0.12, 1.2);
            let left = (radial.center_x - width * 0.5).clamp(-0.2, 1.0);
            let top = (radial.center_y - height * 0.5).clamp(-0.2, 1.0);

            Some(SimpleGradientOverlaySpec {
                color,
                kind: SimpleGradientOverlayKind::Radial {
                    left,
                    top,
                    width,
                    height,
                },
            })
        })
        .collect()
}

fn css_simple_radial_gradient_overlay(value: &str) -> Option<SimpleRadialGradientOverlay> {
    let value = value.trim();
    let layer = split_css_layers(value).into_iter().next()?;
    let inner = layer.strip_prefix("radial-gradient(")?.strip_suffix(')')?;
    let args = crate::core::style::css_sizing::split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let descriptor = args[0].trim().to_ascii_lowercase();
    let mut center_x = 0.5;
    let mut center_y = 0.5;
    if let Some((_, position)) = descriptor.split_once(" at ") {
        let mut parts = position.split_whitespace();
        if let Some(x) = parts.next().and_then(css_percentage_value) {
            center_x = x;
        }
        if let Some(y) = parts.next().and_then(css_percentage_value) {
            center_y = y;
        }
    }

    let mut color = None;
    let mut stop_ratio = None;
    for arg in args.iter().skip(1) {
        let token = arg.trim();
        let token_color = crate::core::style::css_values::css_simple_color(token);
        if color.is_none() && token_color.as_deref() != Some("transparent") {
            color = token_color;
        }
        if stop_ratio.is_none() && token.contains("transparent") {
            stop_ratio = token
                .split_whitespace()
                .find_map(css_percentage_value)
                .or(Some(0.5));
        }
    }

    let color = color?;
    let stop_ratio = stop_ratio.unwrap_or(0.5_f32).clamp(0.12, 0.72);
    let ellipse_scale_x = if descriptor.contains("ellipse") { 1.35 } else { 1.0 };
    let ellipse_scale_y = if descriptor.contains("ellipse") { 0.78 } else { 1.0 };
    let width = (stop_ratio * 2.0 * ellipse_scale_x).clamp(0.18, 1.25);
    let height = (stop_ratio * 2.0 * ellipse_scale_y).clamp(0.18, 1.1);
    Some(SimpleRadialGradientOverlay {
        center_x,
        center_y,
        width,
        height,
        color,
    })
}

pub(crate) fn css_simple_radial_gradient_ring_overlay(
    value: &str,
) -> Option<SimpleRadialGradientRingOverlay> {
    let value = value.trim();
    let layer = split_css_layers(value).into_iter().next()?;
    let inner = layer.strip_prefix("radial-gradient(")?.strip_suffix(')')?;
    let args = crate::core::style::css_sizing::split_css_function_args(inner);
    if args.len() < 3 {
        return None;
    }

    let descriptor = args[0].trim().to_ascii_lowercase();
    let mut center_x = 0.5;
    let mut center_y = 0.5;
    if let Some((_, position)) = descriptor.split_once(" at ") {
        let mut parts = position.split_whitespace();
        if let Some(x) = parts.next().and_then(css_percentage_value) {
            center_x = x;
        }
        if let Some(y) = parts.next().and_then(css_percentage_value) {
            center_y = y;
        }
    }

    let stops: Vec<CssGradientStop> = super::css_gradient_stops(&args[1..])?;
    let mut inner_ratio = None;
    let mut outer_ratio = None;
    let mut color = None;

    for window in stops.windows(3) {
        let [before, middle, after] = window else {
            continue;
        };
        if before.color == "transparent"
            && middle.color != "transparent"
            && after.color == "transparent"
        {
            inner_ratio = Some(middle.start_ratio.max(before.end_ratio));
            outer_ratio = Some(middle.end_ratio.min(after.start_ratio.max(middle.end_ratio)));
            color = Some(middle.color.clone());
            break;
        }
    }

    let color = color?;
    let inner_ratio = inner_ratio?;
    let outer_ratio = outer_ratio?;
    if outer_ratio <= inner_ratio {
        return None;
    }

    let ellipse_scale_x = if descriptor.contains("ellipse") { 1.35 } else { 1.0 };
    let ellipse_scale_y = if descriptor.contains("ellipse") { 0.78 } else { 1.0 };
    let width = (outer_ratio * 2.0 * ellipse_scale_x).clamp(0.18, 1.25);
    let height = (outer_ratio * 2.0 * ellipse_scale_y).clamp(0.18, 1.1);
    let left = (center_x - width * 0.5).clamp(-0.2, 1.0);
    let top = (center_y - height * 0.5).clamp(-0.2, 1.0);
    let border_width =
        ((outer_ratio - inner_ratio) / outer_ratio.max(0.001) * 0.5).clamp(0.01, 0.1);

    Some(SimpleRadialGradientRingOverlay {
        left,
        top,
        width,
        height,
        border_width,
        color,
    })
}
