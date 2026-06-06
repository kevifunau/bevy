mod bands;
mod direction;

use crate::core::style::css_gradients::{
    css_gradient_stops, SimpleGradientOverlayBand, SimpleGradientOverlayDirection,
};

pub(crate) use bands::css_simple_gradient_bands_from_stops;
pub(crate) use direction::css_simple_linear_gradient_direction;
#[cfg(test)]
pub(crate) use direction::css_linear_gradient_direction_from_degrees;

pub(crate) fn css_simple_linear_gradient_overlays(
    layer: &str,
) -> Vec<super::SimpleGradientOverlaySpec> {
    let Some((direction, diagonal_angle, bands)) = css_simple_linear_gradient_bands(layer) else {
        return Vec::new();
    };

    bands
        .into_iter()
        .take(8)
        .map(|band| super::SimpleGradientOverlaySpec {
            color: band.color,
            kind: super::SimpleGradientOverlayKind::Linear {
                direction,
                diagonal_angle,
                start_ratio: band.start_ratio,
                end_ratio: band.end_ratio,
            },
        })
        .collect()
}

pub(crate) fn css_simple_linear_gradient_bands(
    layer: &str,
) -> Option<(
    SimpleGradientOverlayDirection,
    Option<f32>,
    Vec<SimpleGradientOverlayBand>,
)> {
    let layer = layer.trim();
    let inner = layer.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let args = crate::core::style::css_sizing::split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let (direction, diagonal_angle, stop_start_index) =
        css_simple_linear_gradient_direction(&args)?;
    let stops = css_gradient_stops(&args[stop_start_index..])?;
    let bands = css_simple_gradient_bands_from_stops(&stops);
    if bands.is_empty() {
        return None;
    }

    Some((direction, diagonal_angle, bands))
}
