use crate::core::style::css_values::{blend_hex_colors, css_hex_rgba, scale_hex_alpha};

use super::super::{CssGradientStop, SimpleGradientOverlayBand};

pub(crate) fn css_simple_gradient_bands_from_stops(
    stops: &[CssGradientStop],
) -> Vec<SimpleGradientOverlayBand> {
    if !stops.iter().any(|stop| stop.color == "transparent") {
        return css_simple_solid_gradient_bands(stops);
    }

    let mut bands = Vec::new();
    for index in 0..stops.len().saturating_sub(1) {
        let previous = &stops[index];
        let current = &stops[index + 1];

        let transition_start = previous.end_ratio.clamp(0.0, 1.0);
        if previous.color == "transparent" && current.color == "transparent" {
            continue;
        }

        if previous.color == "transparent" {
            if let Some(next_stop) = stops
                .get(index + 2)
                .filter(|stop| stop.color == "transparent")
            {
                bands.push(SimpleGradientOverlayBand {
                    color: current.color.clone(),
                    start_ratio: transition_start,
                    end_ratio: next_stop.end_ratio.max(current.end_ratio),
                });
                continue;
            }
            let transition_end = if current.is_multi_position {
                current.start_ratio
            } else {
                current.end_ratio
            }
            .clamp(transition_start, 1.0);
            if transition_end <= transition_start {
                continue;
            }
            if !should_soften_transparent_leading_segment(
                stops,
                index,
                current,
                transition_start,
                transition_end,
            ) {
                bands.push(SimpleGradientOverlayBand {
                    color: current.color.clone(),
                    start_ratio: transition_start,
                    end_ratio: transition_end,
                });
                continue;
            }
            bands.extend(css_transparent_transition_bands(
                &current.color,
                transition_start,
                transition_end,
                true,
            ));
            continue;
        }

        if current.color == "transparent" {
            let previous_is_multi = previous.is_multi_position;
            let previous_was_already_wrapped_by_transparent = index > 0
                && stops
                    .get(index - 1)
                    .is_some_and(|stop| stop.color == "transparent");
            if previous_was_already_wrapped_by_transparent {
                continue;
            }
            if previous_is_multi {
                bands.push(SimpleGradientOverlayBand {
                    color: previous.color.clone(),
                    start_ratio: previous.start_ratio,
                    end_ratio: current.end_ratio.max(previous.end_ratio),
                });
                continue;
            }
            let transition_end = current.end_ratio.clamp(transition_start, 1.0);
            if transition_end <= transition_start {
                continue;
            }
            bands.extend(css_transparent_transition_bands(
                &previous.color,
                transition_start,
                transition_end,
                false,
            ));
            continue;
        }

        let transition_end = if current.is_multi_position {
            current.start_ratio
        } else {
            current.end_ratio
        }
        .clamp(transition_start, 1.0);

        if transition_end <= transition_start {
            continue;
        }

        let span = transition_end - transition_start;
        let segments = adaptive_gradient_band_count(previous, current, span);
        for index in 0..segments {
            let start_ratio = lerp_ratio(
                transition_start,
                transition_end,
                index as f32 / segments as f32,
            );
            let end_ratio = lerp_ratio(
                transition_start,
                transition_end,
                (index + 1) as f32 / segments as f32,
            );
            if end_ratio <= start_ratio {
                continue;
            }

            let color = if index == 0 {
                previous.color.clone()
            } else if index + 1 == segments {
                current.color.clone()
            } else {
                let color_ratio = index as f32 / (segments - 1) as f32;
                blend_hex_colors(&previous.color, &current.color, color_ratio)
                    .unwrap_or_else(|| current.color.clone())
            };

            bands.push(SimpleGradientOverlayBand {
                color,
                start_ratio,
                end_ratio,
            });
        }
    }

    let mut bands = bands
        .into_iter()
        .filter(|band| band.end_ratio > band.start_ratio)
        .collect::<Vec<_>>();
    bands.sort_by(|left, right| left.start_ratio.total_cmp(&right.start_ratio));
    bands
}

fn should_soften_transparent_leading_segment(
    stops: &[CssGradientStop],
    index: usize,
    current: &CssGradientStop,
    transition_start: f32,
    transition_end: f32,
) -> bool {
    let Some((_, _, _, current_alpha)) = css_hex_rgba(&current.color) else {
        return true;
    };
    if current_alpha < 0.3 {
        return true;
    }

    let transition_span = (transition_end - transition_start).clamp(0.0, 1.0);
    if transition_span <= 0.0 {
        return false;
    }

    let Some(next_stop) = stops
        .get(index + 2)
        .filter(|stop| stop.color != "transparent")
    else {
        return true;
    };
    let Some((_, _, _, next_alpha)) = css_hex_rgba(&next_stop.color) else {
        return true;
    };

    let opaque_tail_span = (next_stop.end_ratio - transition_end).clamp(0.0, 1.0);
    if transition_span <= 0.18 && opaque_tail_span >= 0.45 && next_alpha >= current_alpha * 0.85 {
        return false;
    }

    true
}

fn css_transparent_transition_bands(
    color: &str,
    start_ratio: f32,
    end_ratio: f32,
    fade_in: bool,
) -> Vec<SimpleGradientOverlayBand> {
    let span = (end_ratio - start_ratio).clamp(0.0, 1.0);
    if span <= 0.0 {
        return Vec::new();
    }

    let mut segments = if span >= 0.2 { 4 } else { 3 };
    let alpha = css_hex_rgba(color)
        .map(|(_, _, _, alpha)| alpha)
        .unwrap_or(1.0);
    if alpha < 0.35 {
        segments += 1;
    }

    let mut bands = Vec::new();
    for index in 0..segments {
        let start = lerp_ratio(start_ratio, end_ratio, index as f32 / segments as f32);
        let end = lerp_ratio(start_ratio, end_ratio, (index + 1) as f32 / segments as f32);
        if end <= start {
            continue;
        }

        let edge_ratio = if fade_in {
            (index + 1) as f32 / segments as f32
        } else {
            1.0 - (index as f32 / segments as f32)
        };
        let Some(color) = scale_hex_alpha(color, edge_ratio) else {
            continue;
        };
        let Some((_, _, _, alpha)) = css_hex_rgba(&color) else {
            continue;
        };
        if alpha <= 0.01 {
            continue;
        }

        bands.push(SimpleGradientOverlayBand {
            color,
            start_ratio: start,
            end_ratio: end,
        });
    }

    bands
}

fn css_simple_solid_gradient_bands(stops: &[CssGradientStop]) -> Vec<SimpleGradientOverlayBand> {
    let non_transparent: Vec<&CssGradientStop> = stops
        .iter()
        .filter(|stop| stop.color != "transparent")
        .collect();

    let mut bands = Vec::new();
    for window in non_transparent.windows(2) {
        let [previous, current] = window else {
            continue;
        };

        let gradient_start = previous.start_ratio.min(previous.end_ratio).clamp(0.0, 1.0);
        let gradient_end = current
            .end_ratio
            .max(current.start_ratio)
            .clamp(gradient_start, 1.0);
        if gradient_end <= gradient_start {
            continue;
        }

        let segments =
            adaptive_gradient_band_count(previous, current, gradient_end - gradient_start);
        for index in 0..segments {
            let start_ratio =
                lerp_ratio(gradient_start, gradient_end, index as f32 / segments as f32);
            let end_ratio = lerp_ratio(
                gradient_start,
                gradient_end,
                (index + 1) as f32 / segments as f32,
            );
            if end_ratio <= start_ratio {
                continue;
            }

            let color = if index == 0 {
                previous.color.clone()
            } else if index + 1 == segments {
                current.color.clone()
            } else {
                let color_ratio = index as f32 / (segments - 1) as f32;
                blend_hex_colors(&previous.color, &current.color, color_ratio)
                    .unwrap_or_else(|| current.color.clone())
            };

            bands.push(SimpleGradientOverlayBand {
                color,
                start_ratio,
                end_ratio,
            });
        }
    }

    if bands.is_empty()
        && let Some(stop) = non_transparent.last()
    {
        bands.push(SimpleGradientOverlayBand {
            color: stop.color.clone(),
            start_ratio: stop.start_ratio.clamp(0.0, 1.0),
            end_ratio: stop.end_ratio.clamp(stop.start_ratio, 1.0),
        });
    }

    bands
}

fn adaptive_gradient_band_count(
    previous: &CssGradientStop,
    current: &CssGradientStop,
    span: f32,
) -> usize {
    let span = span.clamp(0.0, 1.0);
    let alpha_delta = gradient_alpha_delta(&previous.color, &current.color);
    let color_delta = gradient_color_distance(&previous.color, &current.color);

    if span >= 0.45 && color_delta < 0.14 && alpha_delta < 0.12 {
        return 4;
    }

    let mut segments = if span >= 0.75 {
        7
    } else if span >= 0.45 {
        7
    } else if span >= 0.2 {
        5
    } else {
        3
    };

    if color_delta > 0.55 || alpha_delta > 0.35 {
        segments += 2;
    }

    segments.clamp(3, 9)
}

fn gradient_color_distance(color_a: &str, color_b: &str) -> f32 {
    let Some((r_a, g_a, b_a, _)) = css_hex_rgba(color_a) else {
        return 0.0;
    };
    let Some((r_b, g_b, b_b, _)) = css_hex_rgba(color_b) else {
        return 0.0;
    };

    let dr = r_a - r_b;
    let dg = g_a - g_b;
    let db = b_a - b_b;
    ((dr * dr + dg * dg + db * db) / 3.0).sqrt()
}

fn gradient_alpha_delta(color_a: &str, color_b: &str) -> f32 {
    let Some((_, _, _, alpha_a)) = css_hex_rgba(color_a) else {
        return 0.0;
    };
    let Some((_, _, _, alpha_b)) = css_hex_rgba(color_b) else {
        return 0.0;
    };

    (alpha_a - alpha_b).abs()
}

fn lerp_ratio(start: f32, end: f32, ratio: f32) -> f32 {
    start + (end - start) * ratio.clamp(0.0, 1.0)
}
