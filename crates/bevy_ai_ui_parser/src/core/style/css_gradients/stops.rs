use crate::core::style::css_values::{css_color, css_percentage_value};

use super::CssGradientStop;

pub(crate) fn css_gradient_stops(args: &[&str]) -> Option<Vec<CssGradientStop>> {
    let raw_stops = args
        .iter()
        .map(|arg| {
            let arg = arg.trim();
            Some((css_color(arg)?, css_gradient_stop_positions(arg)))
        })
        .collect::<Option<Vec<_>>>()?;
    let anchor_positions = css_resolved_gradient_stop_positions(
        &raw_stops
            .iter()
            .map(|(_, positions)| positions.clone())
            .collect::<Vec<_>>(),
    );

    let mut stops = Vec::new();
    let mut previous_end = 0.0;

    for (index, (color, positions)) in raw_stops.into_iter().enumerate() {
        let (start_ratio, end_ratio) = match positions.as_slice() {
            [] => {
                let end = anchor_positions[index];
                if index == 0 {
                    (0.0, end)
                } else {
                    (previous_end, end)
                }
            }
            [single] => {
                if index == 0 {
                    (0.0, *single)
                } else {
                    (previous_end, *single)
                }
            }
            [start, end, ..] => (*start, *end),
        };

        let start_ratio = start_ratio.clamp(0.0, 1.0);
        let end_ratio = end_ratio.clamp(start_ratio, 1.0);
        previous_end = end_ratio;

        stops.push(CssGradientStop {
            color,
            start_ratio,
            end_ratio,
            is_multi_position: positions.len() >= 2,
        });
    }

    Some(stops)
}

fn css_resolved_gradient_stop_positions(raw_positions: &[Vec<f32>]) -> Vec<f32> {
    let mut anchors = raw_positions
        .iter()
        .map(|positions| match positions.as_slice() {
            [] => None,
            [single] => Some(*single),
            [_, end, ..] => Some(*end),
        })
        .collect::<Vec<_>>();

    let mut index = 0usize;
    while index < anchors.len() {
        if anchors[index].is_some() {
            index += 1;
            continue;
        }

        let run_start = index;
        while index < anchors.len() && anchors[index].is_none() {
            index += 1;
        }
        let run_end = index;
        let run_len = run_end - run_start;

        let previous = run_start
            .checked_sub(1)
            .and_then(|previous_index| anchors[previous_index]);
        let next = anchors.get(run_end).copied().flatten();

        match (previous, next) {
            (Some(left), Some(right)) => {
                for offset in 0..run_len {
                    anchors[run_start + offset] =
                        Some(left + (right - left) * ((offset + 1) as f32 / (run_len + 1) as f32));
                }
            }
            (None, Some(right)) => {
                if run_len == 1 {
                    anchors[run_start] = Some(0.0);
                } else {
                    for offset in 0..run_len {
                        anchors[run_start + offset] = Some(right * (offset as f32 / run_len as f32));
                    }
                }
            }
            (Some(left), None) => {
                for offset in 0..run_len {
                    anchors[run_start + offset] =
                        Some(left + (1.0 - left) * ((offset + 1) as f32 / run_len as f32));
                }
            }
            (None, None) => {
                if run_len == 1 {
                    anchors[run_start] = Some(0.0);
                } else {
                    for offset in 0..run_len {
                        anchors[run_start + offset] = Some(offset as f32 / (run_len - 1) as f32);
                    }
                }
            }
        }
    }

    anchors
        .into_iter()
        .map(|anchor| anchor.unwrap_or(0.0).clamp(0.0, 1.0))
        .collect()
}

fn css_gradient_stop_positions(value: &str) -> Vec<f32> {
    value
        .split_whitespace()
        .filter_map(css_gradient_stop_position_value)
        .collect()
}

fn css_gradient_stop_position_value(value: &str) -> Option<f32> {
    if let Some(percent) = css_percentage_value(value) {
        return Some(percent);
    }

    let value = value.trim();
    let number = value.parse::<f32>().ok()?;
    if number.abs() > f32::EPSILON {
        return None;
    }

    Some(0.0)
}
