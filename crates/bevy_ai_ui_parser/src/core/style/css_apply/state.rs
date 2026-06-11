use crate::core::{
    model::{BuiNode, BuiStateVisual, BuiStyles, BuiVisuals},
    style::{
        css_effects::{
            apply_state_filter_color_adjustment, apply_state_opacity_fallback,
            css_filter_color_adjustment,
        },
        css_parser::{
            css_transform_functions, css_transform_rotation, css_transform_scale,
            css_transform_translation,
        },
        css_values::css_color,
    },
};

use super::helpers::normalize_css_value;

fn direct_text_child_font_color(node: &BuiNode) -> Option<&str> {
    node.children.iter().find_map(|child| {
        (child.kind == "text")
            .then_some(child.content.text.as_ref())
            .flatten()
            .map(|text_config| text_config.font_color.as_str())
    })
}

fn ensure_opendesign_normal_state(bui_node: &mut BuiNode) -> &mut BuiStateVisual {
    bui_node
        .state_visuals
        .entry("normal".to_string())
        .or_insert_with(|| BuiStateVisual {
            styles: BuiStyles::default(),
            visuals: BuiVisuals::default(),
            text_color: None,
            image: None,
        })
}

pub(crate) fn apply_opendesign_state_declaration(
    bui_node: &mut BuiNode,
    state: &str,
    name: &str,
    value: &str,
) {
    let value = normalize_css_value(value);
    if value.is_empty() || value.contains("!important") {
        return;
    }
    if bui_node.kind == "text" && !matches!(name, "color" | "opacity" | "filter") {
        return;
    }

    let mut needs_normal_scale_reset = false;
    let base_background_color = bui_node.style.visuals.background_color.clone();
    let base_border_color = bui_node.style.visuals.border_color.clone();
    let base_text_color = bui_node
        .content
        .text
        .as_ref()
        .map(|text_config| text_config.font_color.clone())
        .or_else(|| direct_text_child_font_color(bui_node).map(ToString::to_string));
    let became_empty;
    {
        let state_visual = bui_node
            .state_visuals
            .entry(state.to_string())
            .or_insert_with(|| BuiStateVisual {
                styles: BuiStyles::default(),
                visuals: BuiVisuals::default(),
                text_color: None,
                image: None,
            });

        match name {
            "background" | "background-color" => {
                if let Some(color) = css_color(&value) {
                    state_visual.visuals.background_color = Some(color);
                }
            }
            "border" => {
                if let Some(color) = css_color(&value) {
                    state_visual.visuals.border_color = Some(color);
                }
                if let Some(width) = crate::core::style::css_sizing::css_first_size(&value) {
                    state_visual.visuals.border_width = Some(width);
                }
            }
            "border-color" => {
                if let Some(color) = css_color(&value) {
                    state_visual.visuals.border_color = Some(color);
                }
            }
            "color" => {
                if let Some(color) = css_color(&value) {
                    state_visual.text_color = Some(color);
                }
            }
            "opacity" => {
                if let Ok(opacity) = value.parse::<f32>() {
                    apply_state_opacity_fallback(
                        state_visual,
                        opacity,
                        base_background_color.as_deref(),
                        base_border_color.as_deref(),
                        base_text_color.as_deref(),
                    );
                }
            }
            "transform" => {
                let functions = css_transform_functions(&value);
                for func in &functions {
                    match func.name {
                        "translate" | "translateX" | "translateY" => {
                            if let Some(translation) = css_transform_translation(func) {
                                state_visual.styles.ui_translation = Some(translation);
                            }
                        }
                        "rotate" => {
                            if let Some(rotation) = css_transform_rotation(func) {
                                state_visual.styles.ui_rotation = Some(rotation);
                            }
                        }
                        "scale" => {
                            if let Some(scale) = css_transform_scale(&func.raw) {
                                state_visual.styles.ui_scale = Some(scale);
                                needs_normal_scale_reset = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            "filter" => {
                if let Some(adjustment) = css_filter_color_adjustment(&value) {
                    apply_state_filter_color_adjustment(
                        state_visual,
                        adjustment,
                        base_background_color.as_deref(),
                        base_border_color.as_deref(),
                        base_text_color.as_deref(),
                    );
                }
            }
            _ => {}
        }

        became_empty = state_visual.is_empty();
    }

    if needs_normal_scale_reset {
        ensure_opendesign_normal_state(bui_node).styles.ui_scale = Some("1 1".to_string());
    }
    if became_empty {
        bui_node.state_visuals.remove(state);
    }
}
