use crate::core::{
    model::{BuiBoxShadowConfig, BuiNode, BuiStateVisual, BuiTextShadowConfig},
    style::{
        css_effects::{css_box_shadow, push_box_shadow_layer},
        css_values::{append_hex_alpha, css_adjust_filter_color, CssFilterColorAdjustment},
    },
};

pub(crate) fn css_filter_drop_shadows(value: &str) -> Vec<BuiBoxShadowConfig> {
    let mut shadows = Vec::new();
    let mut rest = value.trim();

    while let Some(start) = rest.find("drop-shadow(") {
        let tail = &rest[start + "drop-shadow(".len()..];
        let Some(end) = css_matching_paren_offset(tail) else {
            break;
        };
        if let Some(shadow) = css_box_shadow(tail[..end].trim()) {
            shadows.push(shadow);
        }
        rest = &tail[end + 1..];
    }

    shadows
}

fn css_matching_paren_offset(value: &str) -> Option<usize> {
    let mut depth = 1usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn css_filter_shadow_length(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
}

pub(crate) fn css_filter_blur_radius(value: &str) -> Option<f32> {
    let value = value.trim();
    let start = value.find("blur(")?;
    let inner = &value[start + "blur(".len()..];
    let end = inner.find(')')?;
    inner[..end].trim().strip_suffix("px")?.parse::<f32>().ok()
}

pub(crate) fn css_filter_color_adjustment(value: &str) -> Option<CssFilterColorAdjustment> {
    let brightness = css_filter_scalar_function(value, "brightness").unwrap_or(1.0);
    let contrast = css_filter_scalar_function(value, "contrast").unwrap_or(1.0);
    let saturate = css_filter_scalar_function(value, "saturate").unwrap_or(1.0);

    ((brightness - 1.0).abs() > 0.001
        || (contrast - 1.0).abs() > 0.001
        || (saturate - 1.0).abs() > 0.001)
        .then_some(CssFilterColorAdjustment {
            brightness,
            contrast,
            saturate,
        })
}

fn css_filter_scalar_function(value: &str, function_name: &str) -> Option<f32> {
    let start = value.find(&format!("{function_name}("))?;
    let inner = &value[start + function_name.len() + 1..];
    let end = inner.find(')')?;
    inner[..end].trim().parse::<f32>().ok()
}

pub(crate) fn apply_filter_blur_fallback(bui_node: &mut BuiNode, blur_radius: f32) {
    let blur_radius = blur_radius.max(0.0);
    if blur_radius <= 0.0 {
        return;
    }

    bui_node
        .children
        .retain(|child| !child.markers.iter().any(|tag| tag == "css-filter-blur"));

    if let Some(text_config) = &mut bui_node.content.text {
        if text_config.text_shadow.is_none() {
            text_config.text_shadow = Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(0.0),
                color: Some(
                    append_hex_alpha(&text_config.font_color, 55.0)
                        .unwrap_or_else(|| text_config.font_color.clone()),
                ),
            });
        }
        return;
    }

    let blur_px = format!("{}px", (blur_radius * 4.0).round());
    let spread_px = format!("{}px", (blur_radius * 1.5).round());
    let fallback_color = bui_node
        .style
        .visuals
        .background_color
        .as_deref()
        .and_then(|color| append_hex_alpha(color, 65.0));
    let blur_shadow = BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some(blur_px),
        spread_radius: Some(spread_px),
        color: fallback_color,
    };
    push_box_shadow_layer(bui_node, blur_shadow, "css-filter-blur", "filter_blur");
}

pub(crate) fn apply_filter_color_adjustment(
    bui_node: &mut BuiNode,
    adjustment: CssFilterColorAdjustment,
) {
    if let Some(color) = &mut bui_node.style.visuals.background_color
        && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
    {
        *color = adjusted;
    }
    if let Some(color) = &mut bui_node.style.visuals.border_color
        && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
    {
        *color = adjusted;
    }
    if let Some(box_shadow) = &mut bui_node.style.visuals.box_shadow
        && let Some(color) = &mut box_shadow.color
        && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
    {
        *color = adjusted;
    }
    if let Some(text_config) = &mut bui_node.content.text {
        if let Some(adjusted) = css_adjust_filter_color(&text_config.font_color, adjustment) {
            text_config.font_color = adjusted;
        }
        if let Some(text_shadow) = &mut text_config.text_shadow
            && let Some(color) = &mut text_shadow.color
            && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
        {
            *color = adjusted;
        }
    }

    for child in &mut bui_node.children {
        apply_filter_color_adjustment(child, adjustment);
    }
}

pub(crate) fn apply_state_filter_color_adjustment(
    state_visual: &mut BuiStateVisual,
    adjustment: CssFilterColorAdjustment,
    base_background_color: Option<&str>,
    base_border_color: Option<&str>,
    base_text_color: Option<&str>,
) {
    if let Some(base) = state_visual
        .visuals
        .background_color
        .clone()
        .or_else(|| base_background_color.map(ToString::to_string))
        && let Some(adjusted) = css_adjust_filter_color(&base, adjustment)
    {
        state_visual.visuals.background_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .visuals
        .border_color
        .clone()
        .or_else(|| base_border_color.map(ToString::to_string))
        && let Some(adjusted) = css_adjust_filter_color(&base, adjustment)
    {
        state_visual.visuals.border_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .text_color
        .clone()
        .or_else(|| base_text_color.map(ToString::to_string))
        && let Some(adjusted) = css_adjust_filter_color(&base, adjustment)
    {
        state_visual.text_color = Some(adjusted);
    }
}

pub(crate) fn apply_state_opacity_fallback(
    state_visual: &mut BuiStateVisual,
    opacity: f32,
    base_background_color: Option<&str>,
    base_border_color: Option<&str>,
    base_text_color: Option<&str>,
) {
    state_visual.styles.ui_opacity = Some(opacity);

    if let Some(base) = state_visual
        .visuals
        .background_color
        .clone()
        .or_else(|| base_background_color.map(ToString::to_string))
        && let Some(adjusted) = append_hex_alpha(&base, opacity * 100.0)
    {
        state_visual.visuals.background_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .visuals
        .border_color
        .clone()
        .or_else(|| base_border_color.map(ToString::to_string))
        && let Some(adjusted) = append_hex_alpha(&base, opacity * 100.0)
    {
        state_visual.visuals.border_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .text_color
        .clone()
        .or_else(|| base_text_color.map(ToString::to_string))
        && let Some(adjusted) = append_hex_alpha(&base, opacity * 100.0)
    {
        state_visual.text_color = Some(adjusted);
    }
}
