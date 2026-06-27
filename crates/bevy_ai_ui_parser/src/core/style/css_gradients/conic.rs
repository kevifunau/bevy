use crate::core::style::css_values::{css_percentage_value, split_css_layers};

use super::{
    shared::SimpleConicGradientOverlay, SimpleGradientOverlayBand, SimpleGradientOverlayKind,
    SimpleGradientOverlaySpec,
};

pub(crate) fn css_simple_conic_gradient_overlays(value: &str) -> Vec<SimpleGradientOverlaySpec> {
    let Some(overlays) = css_simple_conic_gradient_overlay_specs(value) else {
        return Vec::new();
    };

    overlays
        .into_iter()
        .take(4)
        .map(|overlay| SimpleGradientOverlaySpec {
            color: overlay.color,
            kind: SimpleGradientOverlayKind::ConicArc {
                left: overlay.left,
                top: overlay.top,
                width: overlay.width,
                height: overlay.height,
                rotation_degrees: overlay.rotation_degrees,
            },
        })
        .collect()
}

fn css_simple_conic_gradient_overlay_specs(value: &str) -> Option<Vec<SimpleConicGradientOverlay>> {
    let value = value.trim();
    let layer = split_css_layers(value).into_iter().next()?;
    let inner = layer.strip_prefix("conic-gradient(")?.strip_suffix(')')?;
    let args = crate::core::style::css_sizing::split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let (from_degrees, center_x, center_y, stop_start_index) =
        css_simple_conic_gradient_descriptor(&args)?;
    let stops = super::css_gradient_stops(&args[stop_start_index..])?;
    let bands: Vec<SimpleGradientOverlayBand> = stops
        .iter()
        .filter(|stop| stop.color != "transparent" && stop.end_ratio > stop.start_ratio)
        .map(|stop| SimpleGradientOverlayBand {
            color: stop.color.clone(),
            start_ratio: stop.start_ratio,
            end_ratio: stop.end_ratio,
        })
        .collect();
    if bands.is_empty() {
        return None;
    }

    let radius = 0.30f32;
    let thickness = 0.06f32;

    Some(
        bands
            .into_iter()
            .take(2)
            .map(|band| {
                let span = (band.end_ratio - band.start_ratio).clamp(0.02, 0.24);
                let midpoint_ratio = (band.start_ratio + band.end_ratio) * 0.5;
                let midpoint_degrees = from_degrees + midpoint_ratio * 360.0;
                let radians = (midpoint_degrees - 90.0).to_radians();
                let arc_length = (std::f32::consts::TAU * radius * span).clamp(0.08, 0.24);
                let center_arc_x = center_x + radians.cos() * radius;
                let center_arc_y = center_y + radians.sin() * radius;
                let left = (center_arc_x - arc_length * 0.5).clamp(-0.2, 1.0);
                let top = (center_arc_y - thickness * 0.5).clamp(-0.2, 1.0);

                SimpleConicGradientOverlay {
                    left,
                    top,
                    width: arc_length,
                    height: thickness,
                    rotation_degrees: midpoint_degrees,
                    color: band.color,
                }
            })
            .collect(),
    )
}

fn css_simple_conic_gradient_descriptor(args: &[&str]) -> Option<(f32, f32, f32, usize)> {
    let first = args.first()?.trim().to_ascii_lowercase();
    if !first.contains("from ") && !first.contains(" at ") {
        return Some((0.0, 0.5, 0.5, 0));
    }

    let mut from_degrees = 0.0;
    if let Some(from_section) = first.split("from ").nth(1) {
        let angle_token = from_section
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_end_matches(',');
        if let Some(degrees) = angle_token.strip_suffix("deg")
            && let Ok(parsed) = degrees.trim().parse::<f32>()
        {
            from_degrees = parsed;
        }
    }

    let mut center_x = 0.5;
    let mut center_y = 0.5;
    if let Some((_, position_section)) = first.split_once(" at ") {
        let mut parts = position_section.split_whitespace();
        if let Some(x) = parts.next().and_then(css_percentage_value) {
            center_x = x;
        }
        if let Some(y) = parts.next().and_then(css_percentage_value) {
            center_y = y;
        }
    }

    Some((from_degrees, center_x, center_y, 1))
}
