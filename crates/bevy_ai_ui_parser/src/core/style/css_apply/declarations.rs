use crate::core::{
    model::{BuiImageConfig, BuiNode, BuiTextShadowConfig},
    style::{
        css_effects::{
            apply_box_shadow_fallback, apply_clip_path_fallback, apply_css_border,
            apply_css_edge_border, apply_css_edge_border_color, apply_css_edge_border_width,
            apply_filter_blur_fallback, apply_filter_color_adjustment, apply_mask_image_fallback,
            apply_mix_blend_mode_fallback, css_filter_blur_radius, css_filter_color_adjustment,
            css_filter_drop_shadows, css_filter_shadow_length, css_text_shadow,
            node_has_shadow_casting_paint, push_box_shadow_layer, scale_helper_child_opacity,
        },
        css_gradients::apply_simple_gradient_overlays,
        css_parser::{
            apply_css_transform, css_font_size, css_letter_spacing, css_line_height, css_text_align,
        },
        css_sizing::css_first_size,
        css_values::{
            adjust_font_path_for_content, append_hex_alpha, apply_css_font_shorthand,
            apply_css_white_space, css_aspect_ratio, css_background_base_color,
            css_background_image_url, css_color, css_font_family_to_path, css_font_weight,
        },
    },
};

use super::helpers::{
    css_axis_pair, css_display, css_flex_shorthand, css_grid_placement, css_grid_tracks,
    css_overflow, css_stage_fit_mode, normalize_css_value, set_css_overflow_axis, set_css_rect,
    set_simple_css_val, CssAxis,
};

pub(crate) fn apply_opendesign_declaration(bui_node: &mut BuiNode, name: &str, value: &str) {
    let value = normalize_css_value(value);
    if value.is_empty() || value.contains("!important") {
        return;
    }
    if bui_node.kind == "text"
        && !matches!(
            name,
            "color"
                | "font-size"
                | "font-family"
                | "font-weight"
                | "font"
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
                bui_node.layout.styles.display = Some(display.to_string());
            }
        }
        "box-sizing" => {
            if matches!(value.as_str(), "border-box" | "content-box") {
                bui_node.layout.styles.box_sizing = Some(value);
            }
        }
        "visibility" => {
            if matches!(value.as_str(), "visible" | "hidden" | "inherited") {
                bui_node.layout.styles.visibility = Some(value);
            }
        }
        "position" => {
            if matches!(value.as_str(), "absolute" | "relative") {
                bui_node.layout.styles.position_type = Some(value);
            } else if value == "fixed" {
                bui_node.layout.styles.position_type = Some("absolute".to_string());
                bui_node.layout.styles.fixed_node = Some(true);
            }
        }
        "width" => set_simple_css_val(&mut bui_node.layout.styles.width, &value),
        "height" => set_simple_css_val(&mut bui_node.layout.styles.height, &value),
        "min-width" => set_simple_css_val(&mut bui_node.layout.styles.min_width, &value),
        "min-height" => set_simple_css_val(&mut bui_node.layout.styles.min_height, &value),
        "max-width" => set_simple_css_val(&mut bui_node.layout.styles.max_width, &value),
        "max-height" => set_simple_css_val(&mut bui_node.layout.styles.max_height, &value),
        "inset" => {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            match parts.as_slice() {
                [all] => {
                    set_simple_css_val(&mut bui_node.layout.styles.top, all);
                    set_simple_css_val(&mut bui_node.layout.styles.right, all);
                    set_simple_css_val(&mut bui_node.layout.styles.bottom, all);
                    set_simple_css_val(&mut bui_node.layout.styles.left, all);
                }
                [vertical, horizontal] => {
                    set_simple_css_val(&mut bui_node.layout.styles.top, vertical);
                    set_simple_css_val(&mut bui_node.layout.styles.bottom, vertical);
                    set_simple_css_val(&mut bui_node.layout.styles.left, horizontal);
                    set_simple_css_val(&mut bui_node.layout.styles.right, horizontal);
                }
                [top, horizontal, bottom] => {
                    set_simple_css_val(&mut bui_node.layout.styles.top, top);
                    set_simple_css_val(&mut bui_node.layout.styles.left, horizontal);
                    set_simple_css_val(&mut bui_node.layout.styles.right, horizontal);
                    set_simple_css_val(&mut bui_node.layout.styles.bottom, bottom);
                }
                [top, right, bottom, left] => {
                    set_simple_css_val(&mut bui_node.layout.styles.top, top);
                    set_simple_css_val(&mut bui_node.layout.styles.right, right);
                    set_simple_css_val(&mut bui_node.layout.styles.bottom, bottom);
                    set_simple_css_val(&mut bui_node.layout.styles.left, left);
                }
                _ => {}
            }
        }
        "left" => set_simple_css_val(&mut bui_node.layout.styles.left, &value),
        "right" => set_simple_css_val(&mut bui_node.layout.styles.right, &value),
        "top" => set_simple_css_val(&mut bui_node.layout.styles.top, &value),
        "bottom" => set_simple_css_val(&mut bui_node.layout.styles.bottom, &value),
        "margin" => set_css_rect(&mut bui_node.layout.styles.margin, &value),
        "margin-left" => set_simple_css_val(&mut bui_node.layout.styles.margin_left, &value),
        "margin-right" => set_simple_css_val(&mut bui_node.layout.styles.margin_right, &value),
        "margin-top" => set_simple_css_val(&mut bui_node.layout.styles.margin_top, &value),
        "margin-bottom" => set_simple_css_val(&mut bui_node.layout.styles.margin_bottom, &value),
        "padding" => set_css_rect(&mut bui_node.layout.styles.padding, &value),
        "padding-left" => set_simple_css_val(&mut bui_node.layout.styles.padding_left, &value),
        "padding-right" => set_simple_css_val(&mut bui_node.layout.styles.padding_right, &value),
        "padding-top" => set_simple_css_val(&mut bui_node.layout.styles.padding_top, &value),
        "padding-bottom" => set_simple_css_val(&mut bui_node.layout.styles.padding_bottom, &value),
        "padding-inline" => {
            if let Some((left, right)) = css_axis_pair(&value) {
                bui_node.layout.styles.padding_left = Some(left);
                bui_node.layout.styles.padding_right = Some(right);
            }
        }
        "padding-block" => {
            if let Some((top, bottom)) = css_axis_pair(&value) {
                bui_node.layout.styles.padding_top = Some(top);
                bui_node.layout.styles.padding_bottom = Some(bottom);
            }
        }
        "margin-inline" => {
            if let Some((left, right)) = css_axis_pair(&value) {
                bui_node.layout.styles.margin_left = Some(left);
                bui_node.layout.styles.margin_right = Some(right);
            }
        }
        "margin-block" => {
            if let Some((top, bottom)) = css_axis_pair(&value) {
                bui_node.layout.styles.margin_top = Some(top);
                bui_node.layout.styles.margin_bottom = Some(bottom);
            }
        }
        "gap" => {
            if let Some((row_gap, column_gap)) = css_axis_pair(&value) {
                bui_node.layout.styles.row_gap = Some(row_gap);
                bui_node.layout.styles.column_gap = Some(column_gap);
            }
        }
        "row-gap" => set_simple_css_val(&mut bui_node.layout.styles.row_gap, &value),
        "column-gap" => set_simple_css_val(&mut bui_node.layout.styles.column_gap, &value),
        "flex-direction" => bui_node.layout.styles.flex_direction = Some(value),
        "flex-wrap" => bui_node.layout.styles.flex_wrap = Some(value),
        "flex-grow" => bui_node.layout.styles.flex_grow = Some(value),
        "flex-shrink" => bui_node.layout.styles.flex_shrink = Some(value),
        "flex-basis" => set_simple_css_val(&mut bui_node.layout.styles.flex_basis, &value),
        "flex" => {
            if let Some((grow, shrink, basis)) = css_flex_shorthand(&value) {
                bui_node.layout.styles.flex_grow = Some(grow);
                bui_node.layout.styles.flex_shrink = Some(shrink);
                bui_node.layout.styles.flex_basis = Some(basis);
            }
        }
        "align-items" => bui_node.layout.styles.align_items = Some(value),
        "align-self" => bui_node.layout.styles.align_self = Some(value),
        "align-content" => bui_node.layout.styles.align_content = Some(value),
        "justify-content" => bui_node.layout.styles.justify_content = Some(value),
        "justify-items" => bui_node.layout.styles.justify_items = Some(value),
        "justify-self" => bui_node.layout.styles.justify_self = Some(value),
        "place-items" => {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            let align = parts.first().copied();
            let justify = parts.get(1).copied().or(align);
            if let Some(align) = align {
                bui_node.layout.styles.align_items = Some(align.to_string());
            }
            if let Some(justify) = justify {
                bui_node.layout.styles.justify_items = Some(justify.to_string());
                if justify == "center" {
                    bui_node.layout.styles.justify_content = Some("center".to_string());
                }
            }
        }
        "place-content" => {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            let align = parts.first().copied();
            let justify = parts.get(1).copied().or(align);
            if let Some(align) = align {
                bui_node.layout.styles.align_content = Some(align.to_string());
            }
            if let Some(justify) = justify {
                bui_node.layout.styles.justify_content = Some(justify.to_string());
            }
        }
        "place-self" => {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            let align = parts.first().copied();
            let justify = parts.get(1).copied().or(align);
            if let Some(align) = align {
                bui_node.layout.styles.align_self = Some(align.to_string());
            }
            if let Some(justify) = justify {
                bui_node.layout.styles.justify_self = Some(justify.to_string());
            }
        }
        "overflow" => {
            if let Some(overflow) = css_overflow(&value) {
                bui_node.layout.styles.overflow = Some(overflow.to_string());
            }
        }
        "overflow-x" => {
            set_css_overflow_axis(&mut bui_node.layout.styles.overflow, CssAxis::X, &value);
        }
        "overflow-y" => {
            set_css_overflow_axis(&mut bui_node.layout.styles.overflow, CssAxis::Y, &value);
        }
        "grid-template-columns" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.layout.styles.grid_template_columns = Some(tracks);
            }
        }
        "grid-template-rows" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.layout.styles.grid_template_rows = Some(tracks);
            }
        }
        "grid-auto-columns" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.layout.styles.grid_auto_columns = Some(tracks);
            }
        }
        "grid-auto-rows" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.layout.styles.grid_auto_rows = Some(tracks);
            }
        }
        "grid-column" => {
            if let Some(placement) = css_grid_placement(&value) {
                bui_node.layout.styles.grid_column = Some(placement);
            }
        }
        "grid-row" => {
            if let Some(placement) = css_grid_placement(&value) {
                bui_node.layout.styles.grid_row = Some(placement);
            }
        }
        "grid-area" => apply_grid_area(bui_node, &value),
        "border-radius" => {
            if let Some(radius) = css_first_size(&value) {
                bui_node.style.visuals.border_radius = Some(radius);
            }
        }
        "border-width" => set_css_rect(&mut bui_node.style.visuals.border_width, &value),
        "border" => apply_css_border(bui_node, &value),
        "border-top" => apply_css_edge_border(bui_node, "top", &value),
        "border-bottom" => apply_css_edge_border(bui_node, "bottom", &value),
        "border-left" => apply_css_edge_border(bui_node, "left", &value),
        "border-right" => apply_css_edge_border(bui_node, "right", &value),
        "border-color" => {
            if let Some(color) = css_color(&value) {
                bui_node.style.visuals.border_color = Some(color);
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
                bui_node.content.image = Some(BuiImageConfig {
                    texture_path,
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    background_repeat: None,
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
                    bui_node.style.visuals.background_color = Some(color);
                }
                apply_simple_gradient_overlays(bui_node, &value);
            } else if let Some(color) = css_color(&value) {
                bui_node.style.visuals.background_color = Some(color);
            }
            if let Some(texture_path) = css_background_image_url(&value) {
                bui_node.content.image = Some(BuiImageConfig {
                    texture_path,
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    background_repeat: None,
                    atlas: None,
                    slicer: None,
                    flip_x: false,
                    flip_y: false,
                });
            }
        }
        "background-size" => {
            if let Some(image_config) = &mut bui_node.content.image {
                apply_background_size_image_mode(image_config, &value);
                image_config.background_size = Some(value);
            }
        }
        "background-position" => {
            if let Some(image_config) = &mut bui_node.content.image {
                image_config.background_position = Some(value);
            }
        }
        "background-repeat" => {
            if let Some(image_config) = &mut bui_node.content.image {
                image_config.background_repeat = Some(background_repeat_fallback(&value));
            }
        }
        "bui-stage-fit" => {
            if let Some(mode) = css_stage_fit_mode(&value) {
                bui_node
                    .markers
                    .retain(|marker| !marker.starts_with("bui-stage-fit-mode:"));
                bui_node.markers.push(format!("bui-stage-fit-mode:{mode}"));
            }
        }
        "color" => {
            if let Some(color) = css_color(&value)
                && let Some(text_config) = &mut bui_node.content.text
            {
                text_config.font_color = color;
            }
        }
        "font-size" => {
            if let Some(font_size) = css_font_size(&value)
                && let Some(text_config) = &mut bui_node.content.text
            {
                text_config.font_size = font_size;
            }
        }
        "font-family" => {
            if let Some(text_config) = &mut bui_node.content.text {
                let mapped = css_font_family_to_path(&value);
                text_config.font_path =
                    Some(adjust_font_path_for_content(&mapped, &text_config.content));
            }
        }
        "font" => {
            if let Some(text_config) = &mut bui_node.content.text {
                apply_css_font_shorthand(text_config, &value);
            }
        }
        "font-weight" => {
            if let Some(text_config) = &mut bui_node.content.text
                && let Some(font_weight) = css_font_weight(&value)
            {
                text_config.font_weight = Some(font_weight);
            }
        }
        "line-height" => {
            if let Some(text_config) = &mut bui_node.content.text
                && let Some(line_height) = css_line_height(&value)
            {
                text_config.line_height = Some(line_height);
            }
        }
        "letter-spacing" => {
            if let Some(text_config) = &mut bui_node.content.text
                && let Some(letter_spacing) = css_letter_spacing(&value)
            {
                text_config.letter_spacing = Some(letter_spacing);
            }
        }
        "text-align" => {
            if let Some(text_config) = &mut bui_node.content.text
                && css_text_align(&value).is_some()
            {
                text_config.text_align = Some(value);
            }
        }
        "white-space" => {
            if let Some(text_config) = &mut bui_node.content.text {
                apply_css_white_space(text_config, &value);
            }
        }
        "aspect-ratio" => {
            if let Some(aspect_ratio) = css_aspect_ratio(&value) {
                bui_node.layout.styles.aspect_ratio = Some(aspect_ratio);
            }
        }
        "text-shadow" => {
            if let Some(text_config) = &mut bui_node.content.text
                && let Some(text_shadow) = css_text_shadow(&value)
            {
                text_config.text_shadow = Some(text_shadow);
            }
        }
        "opacity" => {
            if let Ok(opacity) = value.parse::<f32>() {
                if let Some(color) = &mut bui_node.style.visuals.background_color
                    && let Some(hex) = append_hex_alpha(color, opacity * 100.0)
                {
                    *color = hex;
                }
                if let Some(color) = &mut bui_node.style.visuals.border_color
                    && let Some(hex) = append_hex_alpha(color, opacity * 100.0)
                {
                    *color = hex;
                }
                if let Some(text_config) = &mut bui_node.content.text
                    && let Some(hex) = append_hex_alpha(&text_config.font_color, opacity * 100.0)
                {
                    text_config.font_color = hex;
                }
                scale_helper_child_opacity(bui_node, opacity);
                bui_node.layout.styles.ui_opacity = Some(opacity);
            }
        }
        "z-index" => {
            if let Ok(parsed) = value.parse::<i32>() {
                bui_node.layout.styles.z_index = Some(parsed.to_string());
            }
        }
        "filter" => {
            let drop_shadows = css_filter_drop_shadows(&value);
            if let Some(text_config) = &mut bui_node.content.text {
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
                        .markers
                        .iter()
                        .any(|tag| tag == "css-filter-drop-shadow")
                });
                let has_clip_contour = bui_node
                    .children
                    .iter()
                    .any(|child| child.markers.iter().any(|tag| tag == "css-clip-contour"));
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
        "cursor"
        | "pointer-events"
        | "transition"
        | "content"
        | "isolation"
        | "-webkit-tap-highlight-color" => {}
        _ => {}
    }
}

fn apply_background_size_image_mode(image_config: &mut BuiImageConfig, value: &str) {
    let value = value.trim().to_ascii_lowercase();
    if matches!(value.as_str(), "contain" | "auto") {
        image_config.image_mode = Some("auto".to_string());
    } else if matches!(value.as_str(), "cover" | "100% 100%") {
        image_config.image_mode = Some("stretch".to_string());
    }
}

fn apply_grid_area(bui_node: &mut BuiNode, value: &str) {
    let parts = value.split('/').map(str::trim).collect::<Vec<_>>();
    match parts.as_slice() {
        [single] => {
            if let Some(placement) = css_grid_placement(single) {
                bui_node.layout.styles.grid_row = Some(placement.clone());
                bui_node.layout.styles.grid_column = Some(placement);
            }
        }
        [row_start, column_start, row_end, column_end] => {
            if let Some(row) = css_grid_area_axis(row_start, row_end) {
                bui_node.layout.styles.grid_row = Some(row);
            }
            if let Some(column) = css_grid_area_axis(column_start, column_end) {
                bui_node.layout.styles.grid_column = Some(column);
            }
        }
        _ => {}
    }
}

fn css_grid_area_axis(start: &str, end: &str) -> Option<String> {
    if start == "auto" && end == "auto" {
        return Some("auto".to_string());
    }
    if let Some(span) = end.strip_prefix("span").map(str::trim) {
        return css_grid_placement(&format!("span {span}"));
    }
    css_grid_placement(start)
}

fn background_repeat_fallback(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "repeat" | "repeat-x" | "repeat-y" | "space" | "round" => "no-repeat".to_string(),
        "no-repeat" => "no-repeat".to_string(),
        other => other.to_string(),
    }
}
