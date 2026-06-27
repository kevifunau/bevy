mod conic;
mod linear;
mod radial;
mod shared;
mod stops;

use crate::core::model::{bui_node, BuiNode};

pub(crate) use conic::css_simple_conic_gradient_overlays;
pub(crate) use linear::css_simple_linear_gradient_overlays;
#[cfg(test)]
pub(crate) use linear::{
    css_linear_gradient_direction_from_degrees, css_simple_gradient_bands_from_stops,
    css_simple_linear_gradient_bands, css_simple_linear_gradient_direction,
};
pub(crate) use radial::{
    css_simple_radial_gradient_overlays, css_simple_radial_gradient_ring_overlay,
};
pub(crate) use shared::{
    CssGradientStop, SimpleGradientOverlayBand, SimpleGradientOverlayDirection,
    SimpleGradientOverlayKind, SimpleGradientOverlaySpec, SimpleRadialGradientRingOverlay,
};
pub(crate) use stops::css_gradient_stops;

pub(crate) fn apply_simple_gradient_overlays(node: &mut BuiNode, value: &str) {
    let specs = css_simple_gradient_overlays(value);
    if specs.is_empty() {
        return;
    }

    if node.layout.styles.position_type.is_none() {
        node.layout.styles.position_type = Some("relative".to_string());
    }

    for (index, spec) in specs.into_iter().enumerate().rev() {
        let overlay_id = gradient_overlay_id(&node.id, index);
        if node.children.iter().any(|child| child.id == overlay_id) {
            continue;
        }

        let mut overlay = bui_node(&overlay_id, "node");
        overlay.markers.push("css-gradient-overlay".to_string());
        overlay.layout.styles.position_type = Some("absolute".to_string());
        overlay.layout.styles.z_index = Some(format!("-{}", index + 1));
        overlay.style.visuals.background_color = Some(spec.color.clone());

        match spec.kind {
            SimpleGradientOverlayKind::Linear {
                direction,
                diagonal_angle,
                start_ratio,
                end_ratio,
            } => {
                match direction {
                    SimpleGradientOverlayDirection::LeftToRight => {
                        overlay.layout.styles.left = Some(format!("{:.0}%", start_ratio * 100.0));
                        overlay.layout.styles.right =
                            Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                        overlay.layout.styles.top = Some("0".to_string());
                        overlay.layout.styles.bottom = Some("0".to_string());
                    }
                    SimpleGradientOverlayDirection::RightToLeft => {
                        overlay.layout.styles.left =
                            Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                        overlay.layout.styles.right = Some(format!("{:.0}%", start_ratio * 100.0));
                        overlay.layout.styles.top = Some("0".to_string());
                        overlay.layout.styles.bottom = Some("0".to_string());
                    }
                    SimpleGradientOverlayDirection::TopToBottom => {
                        overlay.layout.styles.left = Some("0".to_string());
                        overlay.layout.styles.right = Some("0".to_string());
                        overlay.layout.styles.top = Some(format!("{:.0}%", start_ratio * 100.0));
                        overlay.layout.styles.bottom =
                            Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                    }
                    SimpleGradientOverlayDirection::BottomToTop => {
                        overlay.layout.styles.left = Some("0".to_string());
                        overlay.layout.styles.right = Some("0".to_string());
                        overlay.layout.styles.top =
                            Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                        overlay.layout.styles.bottom = Some(format!("{:.0}%", start_ratio * 100.0));
                    }
                }
                if let Some(css_angle) = diagonal_angle {
                    let dominant_axis_degrees = match direction {
                        SimpleGradientOverlayDirection::LeftToRight => 90.0,
                        SimpleGradientOverlayDirection::RightToLeft => 270.0,
                        SimpleGradientOverlayDirection::TopToBottom => 180.0,
                        SimpleGradientOverlayDirection::BottomToTop => 0.0,
                    };
                    let rotation = css_angle - dominant_axis_degrees;
                    overlay.layout.styles.ui_rotation = Some(format!("{:.1}deg", rotation));
                }
            }
            SimpleGradientOverlayKind::Radial {
                left,
                top,
                width,
                height,
                preserve_circle,
            } => {
                overlay.layout.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.layout.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.layout.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.layout.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.style.visuals.border_radius = Some("50%".to_string());
                if preserve_circle {
                    overlay.markers.push("css-radial-circle".to_string());
                }
            }
            SimpleGradientOverlayKind::RadialRing {
                left,
                top,
                width,
                height,
                border_width,
                preserve_circle,
            } => {
                overlay.layout.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.layout.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.layout.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.layout.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.style.visuals.background_color = Some("transparent".to_string());
                overlay.style.visuals.border_color = Some(spec.color.clone());
                overlay.style.visuals.border_width = Some(format!("{:.1}%", border_width * 100.0));
                overlay.style.visuals.border_radius = Some("50%".to_string());
                if preserve_circle {
                    overlay.markers.push("css-radial-circle".to_string());
                }
            }
            SimpleGradientOverlayKind::ConicArc {
                left,
                top,
                width,
                height,
                rotation_degrees,
            } => {
                overlay.layout.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.layout.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.layout.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.layout.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.layout.styles.ui_rotation = Some(format!("{rotation_degrees:.1}deg"));
                overlay.style.visuals.border_radius = Some("999px".to_string());
            }
        }

        node.children.insert(0, overlay);
    }
}

pub(crate) fn css_simple_gradient_overlays(value: &str) -> Vec<SimpleGradientOverlaySpec> {
    crate::core::style::css_values::split_css_layers(value)
        .into_iter()
        .take(4)
        .flat_map(css_simple_gradient_overlay_layer)
        .take(12)
        .collect()
}

fn css_simple_gradient_overlay_layer(layer: &str) -> Vec<SimpleGradientOverlaySpec> {
    let linear = css_simple_linear_gradient_overlays(layer);
    if !linear.is_empty() {
        return linear;
    }

    let conic = css_simple_conic_gradient_overlays(layer);
    if !conic.is_empty() {
        return conic;
    }

    if let Some(ring) = css_simple_radial_gradient_ring_overlay(layer) {
        return vec![SimpleGradientOverlaySpec {
            color: ring.color,
            kind: SimpleGradientOverlayKind::RadialRing {
                left: ring.left,
                top: ring.top,
                width: ring.width,
                height: ring.height,
                border_width: ring.border_width,
                preserve_circle: ring.preserve_circle,
            },
        }];
    }

    css_simple_radial_gradient_overlays(layer)
}

pub(crate) fn preserve_radial_circle_geometry(root: &mut BuiNode) {
    let root_aspect_ratio = node_aspect_ratio(root, None);
    preserve_radial_circle_geometry_with_parent(root, root_aspect_ratio);
}

fn preserve_radial_circle_geometry_with_parent(
    node: &mut BuiNode,
    parent_aspect_ratio: Option<f32>,
) {
    let node_aspect_ratio = node_aspect_ratio(node, parent_aspect_ratio);

    for child in &mut node.children {
        if child.markers.iter().any(|tag| tag == "css-radial-circle") {
            preserve_circle_overlay_geometry(child, node_aspect_ratio);
        }
        preserve_radial_circle_geometry_with_parent(child, node_aspect_ratio);
    }
}

fn preserve_circle_overlay_geometry(node: &mut BuiNode, parent_aspect_ratio: Option<f32>) {
    let Some(parent_aspect_ratio) = parent_aspect_ratio.filter(|ratio| ratio.is_finite()) else {
        return;
    };

    let width_ratio = percent_ratio(node.layout.styles.width.as_deref());
    let height_ratio = percent_ratio(node.layout.styles.height.as_deref());
    let left_ratio = percent_ratio(node.layout.styles.left.as_deref());
    let top_ratio = percent_ratio(node.layout.styles.top.as_deref());

    let (Some(width_ratio), Some(height_ratio), Some(left_ratio), Some(top_ratio)) =
        (width_ratio, height_ratio, left_ratio, top_ratio)
    else {
        return;
    };

    let center_x = left_ratio + width_ratio * 0.5;
    let center_y = top_ratio + height_ratio * 0.5;

    let (new_width_ratio, new_height_ratio) = if parent_aspect_ratio >= 1.0 {
        (width_ratio, width_ratio * parent_aspect_ratio)
    } else {
        (height_ratio / parent_aspect_ratio.max(0.001), height_ratio)
    };

    let new_left_ratio = center_x - new_width_ratio * 0.5;
    let new_top_ratio = center_y - new_height_ratio * 0.5;

    node.layout.styles.left = Some(format_percent(new_left_ratio));
    node.layout.styles.top = Some(format_percent(new_top_ratio));
    node.layout.styles.width = Some(format_percent(new_width_ratio));
    node.layout.styles.height = Some(format_percent(new_height_ratio));
}

fn node_aspect_ratio(node: &BuiNode, fallback: Option<f32>) -> Option<f32> {
    if let Some(aspect_ratio) = node
        .layout
        .styles
        .aspect_ratio
        .as_deref()
        .and_then(|value| value.trim().parse::<f32>().ok())
        .filter(|ratio| ratio.is_finite() && *ratio > 0.0)
    {
        return Some(aspect_ratio);
    }

    let width_px = px_ratio(node.layout.styles.width.as_deref());
    let height_px = px_ratio(node.layout.styles.height.as_deref());
    if let (Some(width_px), Some(height_px)) = (width_px, height_px)
        && height_px > 0.0
    {
        return Some(width_px / height_px);
    }

    let fills_parent = style_is_zero(node.layout.styles.left.as_deref())
        && style_is_zero(node.layout.styles.right.as_deref())
        && style_is_zero(node.layout.styles.top.as_deref())
        && style_is_zero(node.layout.styles.bottom.as_deref());
    if fills_parent {
        return fallback;
    }

    fallback
}

fn style_is_zero(value: Option<&str>) -> bool {
    matches!(value.map(str::trim), Some("0") | Some("0%") | Some("0px"))
}

fn px_ratio(value: Option<&str>) -> Option<f32> {
    let value = value?.trim();
    let px = value.strip_suffix("px")?.trim().parse::<f32>().ok()?;
    Some(px)
}

fn percent_ratio(value: Option<&str>) -> Option<f32> {
    let value = value?.trim();
    let percent = value.strip_suffix('%')?.trim().parse::<f32>().ok()?;
    Some(percent / 100.0)
}

fn format_percent(value: f32) -> String {
    let percent = value * 100.0;
    if (percent - percent.round()).abs() < 0.05 {
        format!("{:.0}%", percent)
    } else {
        format!("{percent:.1}%")
    }
}

fn gradient_overlay_id(node_id: &str, index: usize) -> String {
    if index == 0 {
        format!("{node_id}_gradient_overlay")
    } else {
        format!("{node_id}_gradient_overlay_{}", index + 1)
    }
}
