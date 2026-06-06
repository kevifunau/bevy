use crate::core::{
    model::{BuiNode, BuiNodeType, bui_node},
    style::{
        css_gradients::css_gradient_stops,
        css_parser::normalize_token,
        css_sizing::split_css_function_args,
        css_values::{append_hex_alpha, css_percentage_value},
    },
};

#[derive(Clone, Copy)]
pub(crate) enum MaskFadeDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

pub(crate) struct MaskFadeSpec {
    pub(crate) direction: MaskFadeDirection,
    pub(crate) fade_ratio: f32,
}

pub(crate) struct ClipPolygonContourSpec {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) bottom: f32,
    pub(crate) fill_left: f32,
    pub(crate) fill_right: f32,
    pub(crate) fill_top: f32,
    pub(crate) fill_bottom: f32,
    pub(crate) accent_left: f32,
    pub(crate) accent_top: f32,
    pub(crate) accent_width: f32,
    pub(crate) accent_height: f32,
}

pub(crate) fn apply_mask_image_fallback(node: &mut BuiNode, value: &str) {
    if normalize_token(value) == "none" {
        node.children
            .retain(|child| !child.custom_tags.iter().any(|tag| tag == "css-mask-fade"));
        return;
    }

    let Some(spec) = css_simple_mask_fade(value) else {
        return;
    };

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    let band_count = 20usize;
    for index in 0..band_count {
        let overlay_id = format!("{}_mask_fade_{}", node.id, index + 1);
        if node.children.iter().any(|child| child.id == overlay_id) {
            continue;
        }

        let band_start = spec.fade_ratio * (index as f32 / band_count as f32);
        let band_end = spec.fade_ratio * ((index + 1) as f32 / band_count as f32);
        let alpha = css_mask_fade_band_alpha(index, band_count);

        let mut overlay = bui_node(&overlay_id, BuiNodeType::Node);
        overlay.custom_tags.push("css-mask-fade".to_string());
        overlay.styles.position_type = Some("absolute".to_string());
        overlay.styles.z_index = Some("12".to_string());
        overlay.visuals.background_color =
            append_hex_alpha("#47362B", alpha).or(Some("#47362B52".to_string()));

        match spec.direction {
            MaskFadeDirection::LeftToRight => {
                overlay.styles.left = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.width = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.top = Some("0".to_string());
                overlay.styles.bottom = Some("0".to_string());
            }
            MaskFadeDirection::RightToLeft => {
                overlay.styles.right = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.width = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.top = Some("0".to_string());
                overlay.styles.bottom = Some("0".to_string());
            }
            MaskFadeDirection::TopToBottom => {
                overlay.styles.top = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.height = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.left = Some("0".to_string());
                overlay.styles.right = Some("0".to_string());
            }
            MaskFadeDirection::BottomToTop => {
                overlay.styles.bottom = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.height = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.left = Some("0".to_string());
                overlay.styles.right = Some("0".to_string());
            }
        }

        node.children.push(overlay);
    }
}

fn css_mask_fade_band_alpha(index: usize, band_count: usize) -> f32 {
    if band_count <= 1 {
        return 14.0;
    }

    let progress = 1.0 - index as f32 / (band_count - 1) as f32;
    (1.0 + progress.powf(1.85) * 15.0).clamp(1.0, 16.0)
}

pub(crate) fn apply_clip_path_fallback(node: &mut BuiNode, value: &str) {
    let Some(spec) = css_simple_clip_polygon_contour(value) else {
        return;
    };

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    if node.children.iter().any(|child| {
        child
            .custom_tags
            .iter()
            .any(|tag| tag == "css-clip-contour")
    }) {
        return;
    }

    for child in &mut node.children {
        if !child
            .custom_tags
            .iter()
            .any(|tag| tag == "css-filter-drop-shadow")
        {
            continue;
        }

        child.styles.left = Some(format!("{:.1}%", spec.left * 100.0));
        child.styles.right = Some(format!("{:.1}%", spec.right * 100.0));
        child.styles.top = Some(format!("{:.1}%", spec.top * 100.0));
        child.styles.bottom = Some(format!("{:.1}%", spec.bottom * 100.0));
        child.visuals.border_radius = Some("44%".to_string());
    }

    let fill_color = node.visuals.background_color.clone();
    let contour_color = node.visuals.border_color.clone().or_else(|| {
        node.visuals
            .background_color
            .as_deref()
            .and_then(|color| append_hex_alpha(color, 42.0))
    });
    let accent_color = node
        .visuals
        .background_color
        .as_deref()
        .and_then(|color| append_hex_alpha(color, 58.0));

    if fill_color.is_none() && contour_color.is_none() && accent_color.is_none() {
        let mut guide = bui_node(&format!("{}_clip_bounds", node.id), BuiNodeType::Node);
        guide.custom_tags.push("css-clip-contour".to_string());
        guide.styles.position_type = Some("absolute".to_string());
        guide.styles.left = Some(format!("{:.1}%", spec.left * 100.0));
        guide.styles.right = Some(format!("{:.1}%", spec.right * 100.0));
        guide.styles.top = Some(format!("{:.1}%", spec.top * 100.0));
        guide.styles.bottom = Some(format!("{:.1}%", spec.bottom * 100.0));
        guide.styles.z_index = Some("0".to_string());
        guide.visuals.background_color = Some("transparent".to_string());
        guide.visuals.border_radius = Some("46%".to_string());
        node.children.push(guide);
        return;
    }

    if fill_color.is_some() {
        let mut fill = bui_node(&format!("{}_clip_fill", node.id), BuiNodeType::Node);
        fill.custom_tags.push("css-clip-contour".to_string());
        fill.styles.position_type = Some("absolute".to_string());
        fill.styles.left = Some(format!("{:.1}%", spec.fill_left * 100.0));
        fill.styles.right = Some(format!("{:.1}%", spec.fill_right * 100.0));
        fill.styles.top = Some(format!("{:.1}%", spec.fill_top * 100.0));
        fill.styles.bottom = Some(format!("{:.1}%", spec.fill_bottom * 100.0));
        fill.styles.z_index = Some("1".to_string());
        fill.visuals.background_color = fill_color;
        fill.visuals.border_radius = Some("42%".to_string());
        node.children.push(fill);

        node.visuals.background_color = Some("transparent".to_string());
        node.visuals.border_color = Some("transparent".to_string());
    }

    if let Some(contour_color) = contour_color {
        let mut outer = bui_node(&format!("{}_clip_contour", node.id), BuiNodeType::Node);
        outer.custom_tags.push("css-clip-contour".to_string());
        outer.styles.position_type = Some("absolute".to_string());
        outer.styles.left = Some(format!("{:.1}%", spec.left * 100.0));
        outer.styles.right = Some(format!("{:.1}%", spec.right * 100.0));
        outer.styles.top = Some(format!("{:.1}%", spec.top * 100.0));
        outer.styles.bottom = Some(format!("{:.1}%", spec.bottom * 100.0));
        outer.styles.z_index = Some("3".to_string());
        outer.visuals.background_color = Some("transparent".to_string());
        outer.visuals.border_color = Some(contour_color);
        outer.visuals.border_width = Some("1px".to_string());
        outer.visuals.border_radius = Some("46%".to_string());
        node.children.push(outer);
    }

    if let Some(accent_color) = accent_color {
        let mut accent = bui_node(&format!("{}_clip_contour_accent", node.id), BuiNodeType::Node);
        accent.custom_tags.push("css-clip-contour".to_string());
        accent.styles.position_type = Some("absolute".to_string());
        accent.styles.left = Some(format!("{:.1}%", spec.accent_left * 100.0));
        accent.styles.top = Some(format!("{:.1}%", spec.accent_top * 100.0));
        accent.styles.width = Some(format!("{:.1}%", spec.accent_width * 100.0));
        accent.styles.height = Some(format!("{:.1}%", spec.accent_height * 100.0));
        accent.styles.z_index = Some("4".to_string());
        accent.visuals.background_color = Some(accent_color);
        accent.visuals.border_radius = Some("40%".to_string());
        node.children.push(accent);
    }
}

pub(crate) fn css_simple_mask_fade(value: &str) -> Option<MaskFadeSpec> {
    let value = value.trim();
    let inner = value.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let args = split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let (direction, stop_start_index) = css_simple_mask_fade_direction(&args)?;
    let stops = css_gradient_stops(&args[stop_start_index..])?;
    let mut transparent_seen = false;
    for stop in stops {
        if stop.color == "transparent" {
            transparent_seen = true;
            continue;
        }
        if transparent_seen && stop.start_ratio > 0.0 {
            return Some(MaskFadeSpec {
                direction,
                fade_ratio: stop.start_ratio.clamp(0.04, 0.35),
            });
        }
    }

    None
}

fn css_simple_mask_fade_direction(args: &[&str]) -> Option<(MaskFadeDirection, usize)> {
    let first = args.first()?.trim();
    if let Some(direction) = css_mask_fade_direction_from_token(first) {
        return Some((direction, 1));
    }

    Some((MaskFadeDirection::TopToBottom, 0))
}

fn css_mask_fade_direction_from_token(token: &str) -> Option<MaskFadeDirection> {
    let token = token.trim().to_ascii_lowercase();
    match token.as_str() {
        "90deg" | "to right" => Some(MaskFadeDirection::LeftToRight),
        "270deg" | "to left" => Some(MaskFadeDirection::RightToLeft),
        "180deg" | "to bottom" => Some(MaskFadeDirection::TopToBottom),
        "0deg" | "360deg" | "to top" => Some(MaskFadeDirection::BottomToTop),
        _ => None,
    }
}

pub(crate) fn css_simple_clip_polygon_contour(value: &str) -> Option<ClipPolygonContourSpec> {
    let value = value.trim();
    let inner = value.strip_prefix("polygon(")?.strip_suffix(')')?;
    let points = split_css_function_args(inner)
        .into_iter()
        .filter_map(|point| {
            let mut parts = point.split_whitespace();
            let x = parts.next().and_then(css_clip_path_coordinate_value)?;
            let y = parts.next().and_then(css_clip_path_coordinate_value)?;
            Some((x, y))
        })
        .collect::<Vec<_>>();

    if points.len() < 3 {
        return None;
    }

    let min_x = points.iter().map(|(x, _)| *x).fold(1.0f32, f32::min);
    let max_x = points.iter().map(|(x, _)| *x).fold(0.0f32, f32::max);
    let min_y = points.iter().map(|(_, y)| *y).fold(1.0f32, f32::min);
    let max_y = points.iter().map(|(_, y)| *y).fold(0.0f32, f32::max);

    let width = (max_x - min_x).clamp(0.08, 1.0);
    let height = (max_y - min_y).clamp(0.08, 1.0);
    let fill_inset_x = (width * 0.06).clamp(0.02, 0.08);
    let fill_inset_top = (height * 0.05).clamp(0.015, 0.06);
    let fill_inset_bottom = (height * 0.03).clamp(0.015, 0.05);

    Some(ClipPolygonContourSpec {
        left: min_x.clamp(0.0, 0.92),
        right: (1.0 - max_x).clamp(0.0, 0.92),
        top: min_y.clamp(0.0, 0.92),
        bottom: (1.0 - max_y).clamp(0.0, 0.92),
        fill_left: (min_x + fill_inset_x).clamp(0.0, 0.95),
        fill_right: ((1.0 - max_x) + fill_inset_x).clamp(0.0, 0.95),
        fill_top: (min_y + fill_inset_top).clamp(0.0, 0.95),
        fill_bottom: ((1.0 - max_y) + fill_inset_bottom).clamp(0.0, 0.95),
        accent_left: (min_x + width * 0.10).clamp(0.0, 0.94),
        accent_top: (min_y + height * 0.06).clamp(0.0, 0.94),
        accent_width: (width * 0.28).clamp(0.08, 0.34),
        accent_height: (height * 0.16).clamp(0.06, 0.24),
    })
}

fn css_clip_path_coordinate_value(value: &str) -> Option<f32> {
    css_percentage_value(value).or_else(|| {
        let value = value.trim();
        let number = value.parse::<f32>().ok()?;
        (number.abs() <= f32::EPSILON).then_some(0.0)
    })
}
