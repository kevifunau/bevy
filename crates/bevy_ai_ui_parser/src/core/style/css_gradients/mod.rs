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
            } => {
                overlay.layout.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.layout.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.layout.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.layout.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.style.visuals.border_radius = Some("50%".to_string());
            }
            SimpleGradientOverlayKind::RadialRing {
                left,
                top,
                width,
                height,
                border_width,
            } => {
                overlay.layout.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.layout.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.layout.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.layout.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.style.visuals.background_color = Some("transparent".to_string());
                overlay.style.visuals.border_color = Some(spec.color.clone());
                overlay.style.visuals.border_width = Some(format!("{:.1}%", border_width * 100.0));
                overlay.style.visuals.border_radius = Some("50%".to_string());
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
            },
        }];
    }

    css_simple_radial_gradient_overlays(layer)
}

fn gradient_overlay_id(node_id: &str, index: usize) -> String {
    if index == 0 {
        format!("{node_id}_gradient_overlay")
    } else {
        format!("{node_id}_gradient_overlay_{}", index + 1)
    }
}
