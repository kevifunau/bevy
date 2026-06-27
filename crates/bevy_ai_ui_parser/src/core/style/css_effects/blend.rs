use crate::core::{
    model::BuiNode,
    style::{
        css_parser::normalize_token,
        css_values::{css_hex_rgba, css_multiply_blend_fallback_color, scale_hex_alpha},
    },
};

pub(crate) fn scale_helper_child_opacity(node: &mut BuiNode, opacity: f32) {
    let opacity = opacity.clamp(0.0, 1.0);
    for child in &mut node.children {
        let is_effect_helper = child.markers.iter().any(|tag| {
            tag == "css-gradient-overlay"
                || tag == "css-box-shadow-layer"
                || tag == "css-filter-drop-shadow"
                || tag == "css-filter-blur"
                || tag == "css-clip-contour"
        });
        if !is_effect_helper {
            continue;
        }

        if let Some(color) = &mut child.style.visuals.background_color
            && let Some(scaled) = scale_hex_alpha(color, opacity)
        {
            *color = scaled;
        }
        if let Some(color) = &mut child.style.visuals.border_color
            && let Some(scaled) = scale_hex_alpha(color, opacity)
        {
            *color = scaled;
        }
        if let Some(box_shadow) = &mut child.style.visuals.box_shadow
            && let Some(color) = &mut box_shadow.color
            && let Some(scaled) = scale_hex_alpha(color, opacity)
        {
            *color = scaled;
        }
    }
}

pub(crate) fn apply_mix_blend_mode_fallback(bui_node: &mut BuiNode, value: &str) {
    let mode = normalize_token(value);
    if mode != "multiply" {
        return;
    }

    if let Some(color) = &mut bui_node.style.visuals.background_color
        && let Some(mixed) = css_multiply_blend_fallback_color(color)
    {
        *color = mixed;
    }

    if let Some(color) = &mut bui_node.style.visuals.border_color
        && let Some(mixed) = css_multiply_blend_fallback_color(color)
    {
        *color = mixed;
    }

    if let Some(box_shadow) = &mut bui_node.style.visuals.box_shadow
        && let Some(color) = &mut box_shadow.color
        && let Some(mixed) = css_multiply_blend_fallback_color(color)
    {
        *color = mixed;
    }

    for child in &mut bui_node.children {
        let is_effect_helper = child.markers.iter().any(|tag| {
            tag == "css-gradient-overlay"
                || tag == "css-box-shadow-layer"
                || tag == "css-filter-drop-shadow"
                || tag == "css-filter-blur"
        });
        if !is_effect_helper {
            continue;
        }

        let soften_scene_wash_factor = child
            .style
            .visuals
            .background_color
            .as_deref()
            .and_then(css_multiply_blend_fallback_color)
            .and_then(|mixed| multiply_scene_wash_soften_factor(child, &mixed));

        if let Some(color) = &mut child.style.visuals.background_color
            && let Some(mixed) = css_multiply_blend_fallback_color(color)
        {
            *color = mixed;
            if let Some(factor) = soften_scene_wash_factor
                && let Some(softened) = scale_hex_alpha(color, factor)
            {
                *color = softened;
            }
        }
        if let Some(color) = &mut child.style.visuals.border_color
            && let Some(mixed) = css_multiply_blend_fallback_color(color)
        {
            *color = mixed;
        }
        if let Some(box_shadow) = &mut child.style.visuals.box_shadow
            && let Some(color) = &mut box_shadow.color
            && let Some(mixed) = css_multiply_blend_fallback_color(color)
        {
            *color = mixed;
        }
    }
}

fn multiply_scene_wash_soften_factor(node: &BuiNode, color: &str) -> Option<f32> {
    if node.layout.styles.ui_rotation.is_some() {
        return None;
    }

    let Some((_, _, _, alpha)) = css_hex_rgba(color) else {
        return None;
    };
    if alpha > 0.18 {
        return None;
    }

    let is_round_overlay = matches!(
        node.style.visuals.border_radius.as_deref(),
        Some("50%") | Some("999px")
    );

    let horizontal_full_span = style_is_zero(node.layout.styles.left.as_deref())
        && style_is_zero(node.layout.styles.right.as_deref());
    let vertical_full_span = style_is_zero(node.layout.styles.top.as_deref())
        && style_is_zero(node.layout.styles.bottom.as_deref());
    let width_coverage = overlay_axis_coverage(
        node.layout.styles.left.as_deref(),
        node.layout.styles.right.as_deref(),
        node.layout.styles.width.as_deref(),
    );
    let height_coverage = overlay_axis_coverage(
        node.layout.styles.top.as_deref(),
        node.layout.styles.bottom.as_deref(),
        node.layout.styles.height.as_deref(),
    );

    let dominant_coverage = if is_round_overlay {
        width_coverage.max(height_coverage)
    } else if vertical_full_span {
        width_coverage
    } else if horizontal_full_span {
        height_coverage
    } else {
        return None;
    };
    if dominant_coverage < 0.24 || (is_round_overlay && width_coverage < 0.14) {
        return None;
    }

    let base_factor = if is_round_overlay && dominant_coverage >= 0.46 {
        0.18_f32
    } else if is_round_overlay && dominant_coverage >= 0.34 {
        0.26_f32
    } else if is_round_overlay {
        0.38_f32
    } else if dominant_coverage >= 0.44 {
        0.32_f32
    } else if dominant_coverage >= 0.34 {
        0.40_f32
    } else {
        0.55_f32
    };
    let alpha_bias = if alpha <= 0.08 {
        0.88_f32
    } else if alpha <= 0.12 {
        0.94_f32
    } else {
        1.0_f32
    };

    Some((base_factor * alpha_bias).clamp(0.24_f32, 0.55_f32))
}

fn style_is_zero(value: Option<&str>) -> bool {
    matches!(value.map(str::trim), Some("0") | Some("0%") | Some("0px"))
}

fn overlay_axis_coverage(start: Option<&str>, end: Option<&str>, size: Option<&str>) -> f32 {
    if let Some(size_ratio) = percent_ratio(size) {
        return size_ratio.clamp(0.0, 1.0);
    }

    let start_ratio = percent_ratio(start).unwrap_or(0.0);
    let end_ratio = percent_ratio(end).unwrap_or(0.0);
    (1.0 - start_ratio - end_ratio).clamp(0.0, 1.0)
}

fn percent_ratio(value: Option<&str>) -> Option<f32> {
    let value = value?.trim();
    let percent = value.strip_suffix('%')?.trim().parse::<f32>().ok()?;
    Some(percent / 100.0)
}
