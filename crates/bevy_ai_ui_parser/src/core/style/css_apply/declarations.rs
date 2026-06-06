use crate::core::{
    model::{
        BuiImageConfig, BuiNode, BuiNodeType, BuiTextShadowConfig,
    },
    style::{
        css_effects::{
            apply_box_shadow_fallback, apply_clip_path_fallback, apply_css_border,
            apply_css_edge_border, apply_css_edge_border_color, apply_css_edge_border_width,
            apply_filter_blur_fallback, apply_filter_color_adjustment, apply_mask_image_fallback,
            apply_mix_blend_mode_fallback, css_aspect_ratio, css_background_image_url,
            css_filter_blur_radius, css_filter_color_adjustment, css_filter_drop_shadows,
            css_filter_shadow_length, css_text_shadow, node_has_shadow_casting_paint,
            push_box_shadow_layer, scale_helper_child_opacity,
        },
        css_gradients::apply_simple_gradient_overlays,
        css_parser::{
            apply_css_transform, css_font_size, css_letter_spacing, css_line_height,
            css_text_align,
        },
        css_sizing::css_first_size,
        css_values::{
            adjust_font_path_for_content, append_hex_alpha, apply_css_white_space,
            css_background_base_color, css_color, css_font_family_to_path, css_font_weight,
        },
    },
};

use super::helpers::{
    css_display, css_grid_tracks, css_overflow, normalize_css_value, set_css_rect,
    set_simple_css_val,
};

pub(crate) fn apply_opendesign_declaration(bui_node: &mut BuiNode, name: &str, value: &str) {
    let value = normalize_css_value(value);
    if value.is_empty() || value.contains("!important") {
        return;
    }
    if matches!(bui_node.node_type, BuiNodeType::Text)
        && !matches!(
            name,
            "color"
                | "font-size"
                | "font-family"
                | "font-weight"
                | "line-height"
                | "letter-spacing"
                | "text-align"
                | "text-shadow"
                | "white-space"
                | "opacity"
        )
    {
        return;
    }

    match name {
        "display" => {
            if let Some(display) = css_display(&value) {
                bui_node.styles.display = Some(display.to_string());
            }
        }
        "position" => {
            if matches!(value.as_str(), "absolute" | "relative") {
                bui_node.styles.position_type = Some(value);
            } else if value == "fixed" {
                bui_node.styles.position_type = Some("absolute".to_string());
                bui_node.styles.fixed_node = Some(true);
            }
        }
        "width" => set_simple_css_val(&mut bui_node.styles.width, &value),
        "height" => set_simple_css_val(&mut bui_node.styles.height, &value),
        "min-width" => set_simple_css_val(&mut bui_node.styles.min_width, &value),
        "min-height" => set_simple_css_val(&mut bui_node.styles.min_height, &value),
        "max-width" => set_simple_css_val(&mut bui_node.styles.max_width, &value),
        "max-height" => set_simple_css_val(&mut bui_node.styles.max_height, &value),
        "inset" => {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            match parts.as_slice() {
                [all] => {
                    set_simple_css_val(&mut bui_node.styles.top, all);
                    set_simple_css_val(&mut bui_node.styles.right, all);
                    set_simple_css_val(&mut bui_node.styles.bottom, all);
                    set_simple_css_val(&mut bui_node.styles.left, all);
                }
                [vertical, horizontal] => {
                    set_simple_css_val(&mut bui_node.styles.top, vertical);
                    set_simple_css_val(&mut bui_node.styles.bottom, vertical);
                    set_simple_css_val(&mut bui_node.styles.left, horizontal);
                    set_simple_css_val(&mut bui_node.styles.right, horizontal);
                }
                [top, horizontal, bottom] => {
                    set_simple_css_val(&mut bui_node.styles.top, top);
                    set_simple_css_val(&mut bui_node.styles.left, horizontal);
                    set_simple_css_val(&mut bui_node.styles.right, horizontal);
                    set_simple_css_val(&mut bui_node.styles.bottom, bottom);
                }
                [top, right, bottom, left] => {
                    set_simple_css_val(&mut bui_node.styles.top, top);
                    set_simple_css_val(&mut bui_node.styles.right, right);
                    set_simple_css_val(&mut bui_node.styles.bottom, bottom);
                    set_simple_css_val(&mut bui_node.styles.left, left);
                }
                _ => {}
            }
        }
        "left" => set_simple_css_val(&mut bui_node.styles.left, &value),
        "right" => set_simple_css_val(&mut bui_node.styles.right, &value),
        "top" => set_simple_css_val(&mut bui_node.styles.top, &value),
        "bottom" => set_simple_css_val(&mut bui_node.styles.bottom, &value),
        "margin" => set_css_rect(&mut bui_node.styles.margin, &value),
        "margin-left" => set_simple_css_val(&mut bui_node.styles.margin_left, &value),
        "margin-right" => set_simple_css_val(&mut bui_node.styles.margin_right, &value),
        "margin-top" => set_simple_css_val(&mut bui_node.styles.margin_top, &value),
        "margin-bottom" => set_simple_css_val(&mut bui_node.styles.margin_bottom, &value),
        "padding" => set_css_rect(&mut bui_node.styles.padding, &value),
        "padding-left" => set_simple_css_val(&mut bui_node.styles.padding_left, &value),
        "padding-right" => set_simple_css_val(&mut bui_node.styles.padding_right, &value),
        "padding-top" => set_simple_css_val(&mut bui_node.styles.padding_top, &value),
        "padding-bottom" => set_simple_css_val(&mut bui_node.styles.padding_bottom, &value),
        "padding-inline" => {
            set_simple_css_val(&mut bui_node.styles.padding_left, &value);
            set_simple_css_val(&mut bui_node.styles.padding_right, &value);
        }
        "padding-block" => {
            set_simple_css_val(&mut bui_node.styles.padding_top, &value);
            set_simple_css_val(&mut bui_node.styles.padding_bottom, &value);
        }
        "gap" => {
            if let Some(size) = css_first_size(&value) {
                bui_node.styles.row_gap = Some(size.clone());
                bui_node.styles.column_gap = Some(size);
            }
        }
        "row-gap" => set_simple_css_val(&mut bui_node.styles.row_gap, &value),
        "column-gap" => set_simple_css_val(&mut bui_node.styles.column_gap, &value),
        "flex-direction" => bui_node.styles.flex_direction = Some(value),
        "flex-wrap" => bui_node.styles.flex_wrap = Some(value),
        "flex-grow" => bui_node.styles.flex_grow = Some(value),
        "flex-shrink" => bui_node.styles.flex_shrink = Some(value),
        "flex-basis" => set_simple_css_val(&mut bui_node.styles.flex_basis, &value),
        "align-items" => bui_node.styles.align_items = Some(value),
        "align-self" => bui_node.styles.align_self = Some(value),
        "align-content" => bui_node.styles.align_content = Some(value),
        "justify-content" => bui_node.styles.justify_content = Some(value),
        "justify-items" => bui_node.styles.justify_items = Some(value),
        "justify-self" => bui_node.styles.justify_self = Some(value),
        "place-items" => {
            if value == "center" {
                bui_node.styles.align_items = Some("center".to_string());
                bui_node.styles.justify_items = Some("center".to_string());
                bui_node.styles.justify_content = Some("center".to_string());
            }
        }
        "overflow" => {
            if let Some(overflow) = css_overflow(&value) {
                bui_node.styles.overflow = Some(overflow.to_string());
            }
        }
        "overflow-x" => {
            if value == "auto" || value == "scroll" {
                bui_node.styles.overflow = Some("scroll_x".to_string());
            }
        }
        "overflow-y" => {
            if value == "auto" || value == "scroll" {
                bui_node.styles.overflow = Some("scroll_y".to_string());
            }
        }
        "grid-template-columns" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.styles.grid_template_columns = Some(tracks);
            }
        }
        "grid-template-rows" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.styles.grid_template_rows = Some(tracks);
            }
        }
        "border-radius" => {
            if let Some(radius) = css_first_size(&value) {
                bui_node.visuals.border_radius = Some(radius);
            }
        }
        "border-width" => set_css_rect(&mut bui_node.visuals.border_width, &value),
        "border" => apply_css_border(bui_node, &value),
        "border-top" => apply_css_edge_border(bui_node, "top", &value),
        "border-bottom" => apply_css_edge_border(bui_node, "bottom", &value),
        "border-left" => apply_css_edge_border(bui_node, "left", &value),
        "border-right" => apply_css_edge_border(bui_node, "right", &value),
        "border-color" => {
            if let Some(color) = css_color(&value) {
                bui_node.visuals.border_color = Some(color);
            }
        }
        "border-top-color" => apply_css_edge_border_color(bui_node, "top", &value),
        "border-bottom-color" => apply_css_edge_border_color(bui_node, "bottom", &value),
        "border-left-color" => apply_css_edge_border_color(bui_node, "left", &value),
        "border-right-color" => apply_css_edge_border_color(bui_node, "right", &value),
        "border-top-width" => apply_css_edge_border_width(bui_node, "top", &value),
        "border-bottom-width" => apply_css_edge_border_width(bui_node, "bottom", &value),
        "border-left-width" => apply_css_edge_border_width(bui_node, "left", &value),
        "border-right-width" => apply_css_edge_border_width(bui_node, "right", &value),
        "box-shadow" => apply_box_shadow_fallback(bui_node, &value),
        "background-image" => {
            if let Some(texture_path) = css_background_image_url(&value) {
                bui_node.image_config = Some(BuiImageConfig {
                    texture_path,
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    atlas: None,
                    slicer: None,
                    flip_x: false,
                    flip_y: false,
                });
            }
        }
        "background" | "background-color" => {
            if name == "background" {
                if let Some(color) = css_background_base_color(&value) {
                    bui_node.visuals.background_color = Some(color);
                }
                apply_simple_gradient_overlays(bui_node, &value);
            } else if let Some(color) = css_color(&value) {
                bui_node.visuals.background_color = Some(color);
            }
            if let Some(texture_path) = css_background_image_url(&value) {
                bui_node.image_config = Some(BuiImageConfig {
                    texture_path,
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    atlas: None,
                    slicer: None,
                    flip_x: false,
                    flip_y: false,
                });
            }
        }
        "background-size" => {
            if let Some(image_config) = &mut bui_node.image_config {
                image_config.background_size = Some(value);
            }
        }
        "background-position" => {
            if let Some(image_config) = &mut bui_node.image_config {
                image_config.background_position = Some(value);
            }
        }
        "color" => {
            if let Some(color) = css_color(&value)
                && let Some(text_config) = &mut bui_node.text_config
            {
                text_config.font_color = color;
            }
        }
        "font-size" => {
            if let Some(font_size) = css_font_size(&value)
                && let Some(text_config) = &mut bui_node.text_config
            {
                text_config.font_size = font_size;
            }
        }
        "font-family" => {
            if let Some(text_config) = &mut bui_node.text_config {
                let mapped = css_font_family_to_path(&value);
                text_config.font_path =
                    Some(adjust_font_path_for_content(&mapped, &text_config.content));
            }
        }
        "font-weight" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(font_weight) = css_font_weight(&value)
            {
                text_config.font_weight = Some(font_weight);
            }
        }
        "line-height" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(line_height) = css_line_height(&value)
            {
                text_config.line_height = Some(line_height);
            }
        }
        "letter-spacing" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(letter_spacing) = css_letter_spacing(&value)
            {
                text_config.letter_spacing = Some(letter_spacing);
            }
        }
        "text-align" => {
            if let Some(text_config) = &mut bui_node.text_config && css_text_align(&value).is_some()
            {
                text_config.text_align = Some(value);
            }
        }
        "white-space" => {
            if let Some(text_config) = &mut bui_node.text_config {
                apply_css_white_space(text_config, &value);
            }
        }
        "aspect-ratio" => {
            if let Some(aspect_ratio) = css_aspect_ratio(&value) {
                bui_node.styles.aspect_ratio = Some(aspect_ratio);
            }
        }
        "text-shadow" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(text_shadow) = css_text_shadow(&value)
            {
                text_config.text_shadow = Some(text_shadow);
            }
        }
        "opacity" => {
            if let Ok(opacity) = value.parse::<f32>() {
                if let Some(color) = &mut bui_node.visuals.background_color
                    && let Some(hex) = append_hex_alpha(color, opacity * 100.0)
                {
                    *color = hex;
                }
                if let Some(color) = &mut bui_node.visuals.border_color
                    && let Some(hex) = append_hex_alpha(color, opacity * 100.0)
                {
                    *color = hex;
                }
                if let Some(text_config) = &mut bui_node.text_config
                    && let Some(hex) = append_hex_alpha(&text_config.font_color, opacity * 100.0)
                {
                    text_config.font_color = hex;
                }
                scale_helper_child_opacity(bui_node, opacity);
                bui_node.styles.ui_opacity = Some(opacity);
            }
        }
        "z-index" => {
            if let Ok(parsed) = value.parse::<i32>() {
                bui_node.styles.z_index = Some(parsed.to_string());
            }
        }
        "filter" => {
            let drop_shadows = css_filter_drop_shadows(&value);
            if let Some(text_config) = &mut bui_node.text_config {
                if text_config.text_shadow.is_none()
                    && let Some(drop_shadow) = drop_shadows.first()
                {
                    text_config.text_shadow = Some(BuiTextShadowConfig {
                        offset_x: drop_shadow
                            .offset_x
                            .as_deref()
                            .and_then(css_filter_shadow_length),
                        offset_y: drop_shadow
                            .offset_y
                            .as_deref()
                            .and_then(css_filter_shadow_length),
                        color: drop_shadow.color.clone(),
                    });
                }
            } else {
                bui_node.children.retain(|child| {
                    !child
                        .custom_tags
                        .iter()
                        .any(|tag| tag == "css-filter-drop-shadow")
                });
                let has_clip_contour = bui_node
                    .children
                    .iter()
                    .any(|child| child.custom_tags.iter().any(|tag| tag == "css-clip-contour"));
                let allow_transparent_clip_shadow = has_clip_contour && drop_shadows.len() > 1;
                if node_has_shadow_casting_paint(bui_node) || allow_transparent_clip_shadow {
                    for (index, drop_shadow) in drop_shadows.into_iter().enumerate() {
                        push_box_shadow_layer(
                            bui_node,
                            drop_shadow,
                            "css-filter-drop-shadow",
                            &format!("filter_drop_shadow_{}", index + 1),
                        );
                    }
                }
            }
            if let Some(blur_radius) = css_filter_blur_radius(&value) {
                apply_filter_blur_fallback(bui_node, blur_radius);
            }
            if let Some(adjustment) = css_filter_color_adjustment(&value) {
                apply_filter_color_adjustment(bui_node, adjustment);
            }
        }
        "mask-image" => apply_mask_image_fallback(bui_node, &value),
        "mix-blend-mode" => apply_mix_blend_mode_fallback(bui_node, &value),
        "clip-path" => apply_clip_path_fallback(bui_node, &value),
        "transform" => apply_css_transform(bui_node, &value),
        "cursor" | "pointer-events" | "transition" | "content" | "isolation"
        | "-webkit-tap-highlight-color" => {}
        _ => {}
    }
}
