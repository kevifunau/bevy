use std::collections::HashMap;

use crate::core::{
    model::{
        bui_node, text_node, BuiActionBinding, BuiBinding, BuiImageConfig, BuiNode,
        BuiScrollViewSemantics, BuiSliderSemantics, BuiTextConfig,
    },
    opendesign::{
        build::apply_opendesign_styles,
        stylesheet::OpenDesignStylesheet,
        svg::{
            extract_svg_markup, is_svg_tag, svg_asset_key, svg_image_node, svg_render_scale,
            SvgAssetEntry,
        },
    },
    style::css_values::normalize_cjk_linebreak,
    support::ids::sanitize_id,
};

use super::text::{apply_inherited_text_styles, propagate_direct_text_state_visuals};

pub(crate) fn generic_append_children(
    parent: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
    id_counts: &mut HashMap<String, usize>,
    svg_assets: &mut Vec<SvgAssetEntry>,
) {
    let before_decls = stylesheet.matching_pseudo_declarations(dom_node, "before");
    if !before_decls.is_empty() {
        let mut pseudo_node = bui_node(&format!("{}_pseudo_before", parent.id), "node");
        pseudo_node.markers.push("pseudo:before".to_string());
        for (name, value) in &before_decls {
            let value = stylesheet.resolve_value(value);
            crate::core::style::css_apply::apply_opendesign_declaration(
                &mut pseudo_node,
                name,
                &value,
            );
        }
        parent.children.push(pseudo_node);
    }

    let mut direct_text_index = 0;
    let mut svg_fallback_index = 0;

    for child in dom_node.children() {
        if child.is_element() {
            if child.tag_name().name() == "br" {
                append_line_break(parent);
                continue;
            }
            if is_non_visual_html_tag(child.tag_name().name()) {
                continue;
            }
            if is_svg_tag(child.tag_name().name()) {
                if child.tag_name().name() == "svg" {
                    let index = svg_fallback_index + 1;
                    let key = svg_asset_key(parent, child, index);
                    let (render_w, render_h) = svg_render_scale(child);
                    let svg_markup = extract_svg_markup(child);
                    let png_path = format!("assets/png/{key}.png");
                    let image_node = svg_image_node(parent, child, index, &png_path);
                    svg_assets.push(SvgAssetEntry {
                        key,
                        svg_markup,
                        render_width: render_w,
                        render_height: render_h,
                    });
                    svg_fallback_index += 1;
                    parent.children.push(image_node);
                }
                continue;
            }
            let id = generic_dom_id(child, id_counts);
            let kind = generic_node_kind(child);
            let mut child_node = generic_element_node(&id, kind, stylesheet, child);
            apply_slider_child_defaults(&mut child_node);
            apply_button_text_layout_defaults(&mut child_node);
            if child.tag_name().name() == "img"
                && let Some(src) = child.attribute("src").filter(|src| !src.trim().is_empty())
            {
                child_node.content.image = Some(BuiImageConfig {
                    texture_path: src.to_string(),
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    background_repeat: None,
                    atlas: None,
                    slicer: None,
                    flip_x: false,
                    flip_y: false,
                });
                parent.children.push(child_node);
                continue;
            }
            generic_append_children(&mut child_node, child, stylesheet, id_counts, svg_assets);
            parent.children.push(child_node);
        } else if child.node_type() == roxmltree::NodeType::Comment {
            continue;
        } else if let Some(text) = child.text().map(str::trim).filter(|text| !text.is_empty()) {
            if append_text_after_line_break(parent, text) {
                continue;
            }
            direct_text_index += 1;
            let mut text_child = text_node(
                &format!("{}_text_{}", parent.id, direct_text_index),
                text,
                16.0,
                "#3B2818",
                Some("Hiragino Sans GB.ttc"),
            );
            if let Some(source) = dom_node
                .attribute("data-binding")
                .filter(|value| !value.trim().is_empty())
            {
                text_child.bindings.push(BuiBinding {
                    target: "text.content".to_string(),
                    source: source.to_string(),
                });
            }
            apply_inherited_text_styles(stylesheet, &mut text_child, dom_node);
            apply_opendesign_styles(stylesheet, &mut text_child, dom_node);
            if let Some(text_config) = &mut text_child.content.text {
                normalize_cjk_linebreak(text_config);
                if parent.kind == "button" && text_config.line_height.is_none() {
                    text_config.line_height = Some("1".to_string());
                }
            }
            parent.children.push(text_child);
        }
    }

    propagate_direct_text_state_visuals(parent);
    apply_direct_text_alignment_to_container(parent);
    normalize_implicit_grid_layout(parent);
    move_background_padding_to_children(parent);

    let after_decls = stylesheet.matching_pseudo_declarations(dom_node, "after");
    if !after_decls.is_empty() {
        let mut pseudo_node = bui_node(&format!("{}_pseudo_after", parent.id), "node");
        pseudo_node.markers.push("pseudo:after".to_string());
        for (name, value) in &after_decls {
            let value = stylesheet.resolve_value(value);
            crate::core::style::css_apply::apply_opendesign_declaration(
                &mut pseudo_node,
                name,
                &value,
            );
        }
        parent.children.push(pseudo_node);
    }
}

pub(crate) fn generic_element_node(
    id: &str,
    kind: &str,
    stylesheet: &OpenDesignStylesheet,
    dom_node: roxmltree::Node<'_, '_>,
) -> BuiNode {
    let mut node = bui_node(id, kind);
    apply_text_input_content_defaults(&mut node, dom_node);

    if let Some(classes) = dom_node.attribute("class") {
        node.markers.extend(
            classes
                .split_whitespace()
                .filter(|class| !class.is_empty())
                .map(|class| format!("class:{class}")),
        );
    }

    if let Some(value) = dom_node
        .attribute("data-skill")
        .filter(|value| !value.trim().is_empty())
    {
        node.markers.push(format!("data-skill:{value}"));
    }
    if let Some(value) = dom_node
        .attribute("data-equip")
        .filter(|value| !value.trim().is_empty())
    {
        node.markers.push(format!("data-equip:{value}"));
    }
    if let Some(value) = dom_node
        .attribute("data-tab")
        .filter(|value| !value.trim().is_empty())
    {
        node.markers.push(format!("data-tab:{value}"));
    }
    if let Some(value) = dom_node
        .attribute("data-tab-panel")
        .filter(|value| !value.trim().is_empty())
    {
        node.markers.push(format!("data-tab-panel:{value}"));
    }
    if let Some(value) = dom_node
        .attribute("aria-label")
        .filter(|value| !value.trim().is_empty())
    {
        node.markers.push(format!("aria-label:{value}"));
    }
    if dom_node.attribute("aria-selected") == Some("true") {
        node.markers.push("aria-selected:true".to_string());
    }
    if dom_node.attribute("aria-current") == Some("page") {
        node.markers.push("aria-current:page".to_string());
    }
    if dom_node.attribute("selected").is_some() {
        node.markers.push("State_Selected".to_string());
    }
    if dom_node.attribute("disabled").is_some()
        || dom_node.attribute("aria-disabled") == Some("true")
    {
        node.markers.push("State_Disabled".to_string());
    }
    if dom_node.attribute("checked").is_some() || dom_node.attribute("aria-checked") == Some("true")
    {
        node.markers.push("State_Checked".to_string());
    }

    if let Some(action) = dom_node.attribute("data-action") {
        node.actions.push(BuiActionBinding {
            event: "press".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(source) = dom_node
        .attribute("data-binding")
        .or_else(|| dom_node.attribute("name"))
        .filter(|value| !value.trim().is_empty())
        .filter(|_| node.kind == "text_input" || node.kind == "slider" || node.kind == "toggle")
    {
        node.bindings.push(BuiBinding {
            target: match node.kind.as_str() {
                "slider" => "value",
                "toggle" => "checked",
                _ => "text.content",
            }
            .to_string(),
            source: source.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-press")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "press".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-hover-enter")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "hover_enter".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-hover-exit")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "hover_exit".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-change")
        .or_else(|| dom_node.attribute("data-action-value-changed"))
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "value_changed".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-submit")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "submit".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-focus")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "focus".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-blur")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "blur".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-scroll")
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "scroll".to_string(),
            emit: action.to_string(),
        });
    }
    if let Some(action) = dom_node
        .attribute("data-action-selection-changed")
        .or_else(|| dom_node.attribute("data-action-select"))
        .filter(|value| !value.trim().is_empty())
    {
        node.actions.push(BuiActionBinding {
            event: "selection_changed".to_string(),
            emit: action.to_string(),
        });
    }

    if let Some(group) = dom_node
        .attribute("data-tab-group")
        .filter(|value| !value.trim().is_empty())
    {
        node.semantics.tab_group_name = Some(group.to_string());
    }
    if let Some(value) = dom_node
        .attribute("data-tab")
        .filter(|value| !value.trim().is_empty())
    {
        if node.semantics.tab_group_name.is_some() {
            node.semantics.tab_value = Some(value.to_string());
        }
    }
    if dom_node.attribute("role") == Some("progressbar")
        || dom_node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(is_progress_like_class))
    {
        node.semantics.progress_binding_source = Some(
            dom_node
                .attribute("aria-valuenow")
                .or_else(|| dom_node.attribute("data-progress"))
                .unwrap_or("progress")
                .to_string(),
        );
    }
    if dom_node
        .attribute("class")
        .is_some_and(|classes| classes.split_whitespace().any(is_progress_fill_like_class))
    {
        node.semantics.progress_fill = true;
    }
    apply_opendesign_styles(stylesheet, &mut node, dom_node);
    apply_slider_semantics(&mut node, dom_node);
    apply_list_semantics(&mut node, dom_node);
    apply_scroll_view_semantics(&mut node, dom_node);
    apply_dropdown_semantics(&mut node, dom_node);
    apply_attribute_state_aliases(&mut node, dom_node);
    suppress_decorative_gradient_fallbacks(&mut node);
    node
}

fn generic_node_kind(dom_node: roxmltree::Node<'_, '_>) -> &'static str {
    let tag = dom_node.tag_name().name();
    if tag == "img" {
        return "image";
    }
    if tag == "option" || dom_node.attribute("role") == Some("option") {
        return "button";
    }
    if dom_node.attribute("role") == Some("checkbox")
        || dom_node.attribute("role") == Some("switch")
        || dom_node.attribute("data-toggle").is_some()
        || (tag == "input" && dom_node.attribute("type") == Some("checkbox"))
    {
        return "toggle";
    }
    if tag == "textarea" || (tag == "input" && is_text_input_type(dom_node.attribute("type"))) {
        return "text_input";
    }
    if dom_node.attribute("role") == Some("slider")
        || dom_node.attribute("data-slider").is_some()
        || (tag == "input" && dom_node.attribute("type") == Some("range"))
    {
        return "slider";
    }
    if tag == "button"
        || dom_node.attribute("role") == Some("button")
        || dom_node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(is_button_like_class))
    {
        return "button";
    }

    "node"
}

fn is_text_input_type(input_type: Option<&str>) -> bool {
    matches!(
        input_type.unwrap_or("text"),
        "text" | "password" | "search" | "email" | "url" | "tel" | "number"
    )
}

fn apply_slider_semantics(node: &mut BuiNode, dom_node: roxmltree::Node<'_, '_>) {
    if node.kind != "slider" {
        return;
    }

    let min = numeric_attribute(dom_node, &["min", "aria-valuemin"]).unwrap_or(0.0);
    let max = numeric_attribute(dom_node, &["max", "aria-valuemax"]).unwrap_or(1.0);
    let value = numeric_attribute(dom_node, &["value", "aria-valuenow", "data-value"])
        .unwrap_or(min)
        .clamp(min, max);
    let step = numeric_attribute(dom_node, &["step", "data-step"]);
    let orientation = dom_node
        .attribute("aria-orientation")
        .or_else(|| dom_node.attribute("data-orientation"))
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string);

    node.semantics.slider = Some(BuiSliderSemantics {
        value,
        min,
        max,
        step,
        orientation,
    });
}

fn apply_list_semantics(node: &mut BuiNode, dom_node: roxmltree::Node<'_, '_>) {
    let list_source = dom_node
        .attribute("data-bui-list")
        .or_else(|| dom_node.attribute("data-list-binding"))
        .filter(|value| !value.trim().is_empty());
    let json_source = dom_node
        .attribute("data-bui-json-src")
        .or_else(|| dom_node.attribute("data-json-src"))
        .filter(|value| !value.trim().is_empty());

    if let Some(source) = list_source {
        node.semantics.list_binding_source = Some(source.to_string());
    }
    if let Some(source) = json_source {
        node.semantics.list_json_source = Some(source.to_string());
    }
    if let Some(mode) = dom_node
        .attribute("data-bui-json-mode")
        .or_else(|| dom_node.attribute("data-json-mode"))
        .filter(|value| !value.trim().is_empty())
    {
        node.semantics.list_json_mode = Some(mode.to_string());
    }
    if let Some(page_size) = dom_node
        .attribute("data-bui-page-size")
        .or_else(|| dom_node.attribute("data-page-size"))
        .and_then(|value| value.parse::<usize>().ok())
    {
        node.semantics.list_page_size = Some(page_size);
    }
    if let Some(source) = dom_node
        .attribute("data-bui-page-source")
        .or_else(|| dom_node.attribute("data-page-source"))
        .filter(|value| !value.trim().is_empty())
    {
        node.semantics.list_page_source = Some(source.to_string());
    }
}

fn apply_scroll_view_semantics(node: &mut BuiNode, dom_node: roxmltree::Node<'_, '_>) {
    let explicit_scroll_view = dom_node.attribute("data-scroll-view").is_some()
        || dom_node.attribute("role") == Some("scrollbar")
        || dom_node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(is_scroll_view_like_class));
    let has_scroll_overflow = node
        .layout
        .styles
        .overflow
        .as_deref()
        .is_some_and(overflow_has_scroll_axis);

    if !explicit_scroll_view && !has_scroll_overflow {
        return;
    }

    let axis = dom_node
        .attribute("data-axis")
        .or_else(|| dom_node.attribute("aria-orientation"))
        .filter(|value| !value.trim().is_empty())
        .map(normalize_scroll_axis)
        .or_else(|| infer_scroll_axis(node.layout.styles.overflow.as_deref()));

    if node.layout.styles.overflow.is_none() {
        node.layout.styles.overflow = Some(
            match axis.as_deref() {
                Some("x") => "scroll_x",
                Some("y") => "scroll_y",
                _ => "scroll",
            }
            .to_string(),
        );
    }

    node.semantics.scroll_view = Some(BuiScrollViewSemantics {
        binding_source: dom_node
            .attribute("data-scroll-binding")
            .or_else(|| dom_node.attribute("data-binding"))
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string),
        axis,
    });
}

fn apply_dropdown_semantics(node: &mut BuiNode, dom_node: roxmltree::Node<'_, '_>) {
    let tag = dom_node.tag_name().name();
    if tag == "select"
        || dom_node.attribute("role") == Some("combobox")
        || dom_node.attribute("data-dropdown").is_some()
        || dom_node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(is_dropdown_like_class))
    {
        let group = dom_node
            .attribute("data-dropdown")
            .or_else(|| dom_node.attribute("name"))
            .unwrap_or(&node.id)
            .to_string();
        node.semantics.dropdown_group_name = Some(group.clone());
        node.semantics.dropdown_binding_source = dom_node
            .attribute("data-binding")
            .or_else(|| dom_node.attribute("name"))
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
            .or(Some(group));
        return;
    }

    if tag != "option" && dom_node.attribute("role") != Some("option") {
        return;
    }

    let Some(parent) = dom_node.parent_element() else {
        return;
    };
    let parent_tag = parent.tag_name().name();
    let parent_is_dropdown = parent_tag == "select"
        || parent.attribute("role") == Some("combobox")
        || parent.attribute("role") == Some("listbox")
        || parent.attribute("data-dropdown").is_some()
        || parent
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(is_dropdown_like_class));
    if !parent_is_dropdown {
        return;
    }

    let group = parent
        .attribute("data-dropdown")
        .or_else(|| parent.attribute("name"))
        .or_else(|| parent.attribute("id"))
        .unwrap_or("dropdown")
        .to_string();
    let value = dom_node
        .attribute("value")
        .or_else(|| dom_node.attribute("data-value"))
        .or_else(|| dom_node.text())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&node.id)
        .to_string();
    let label = dom_node
        .text()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    node.semantics.dropdown_group_name = Some(group);
    node.semantics.dropdown_value = Some(value);
    node.semantics.dropdown_label = label;
}

fn normalize_scroll_axis(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "horizontal" | "x" => "x".to_string(),
        "vertical" | "y" => "y".to_string(),
        _ => "both".to_string(),
    }
}

fn infer_scroll_axis(overflow: Option<&str>) -> Option<String> {
    match overflow {
        Some("scroll_x") | Some("scroll hidden") | Some("scroll clip") => Some("x".to_string()),
        Some("scroll_y") | Some("hidden scroll") | Some("clip scroll") => Some("y".to_string()),
        Some(value) if overflow_has_scroll_axis(value) => Some("both".to_string()),
        _ => None,
    }
}

fn overflow_has_scroll_axis(value: &str) -> bool {
    value
        .split_whitespace()
        .any(|part| matches!(part, "scroll" | "scroll_x" | "scroll_y"))
}

fn numeric_attribute(dom_node: roxmltree::Node<'_, '_>, names: &[&str]) -> Option<f32> {
    names
        .iter()
        .find_map(|name| dom_node.attribute(*name)?.parse::<f32>().ok())
}

fn apply_slider_child_defaults(node: &mut BuiNode) {
    if node.kind != "slider" {
        return;
    }
    node.layout
        .styles
        .display
        .get_or_insert_with(|| "flex".to_string());
    node.layout
        .styles
        .align_items
        .get_or_insert_with(|| "center".to_string());
}

fn apply_text_input_content_defaults(node: &mut BuiNode, dom_node: roxmltree::Node<'_, '_>) {
    if node.kind != "text_input" || node.content.text.is_some() {
        return;
    }

    let tag = dom_node.tag_name().name();
    let content = if tag == "textarea" {
        dom_node.text().map(str::trim).unwrap_or_default()
    } else {
        dom_node.attribute("value").unwrap_or_default()
    };
    let placeholder = dom_node
        .attribute("placeholder")
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string);

    node.content.text = Some(BuiTextConfig {
        content: content.to_string(),
        placeholder,
        font_size: 16.0,
        font_color: "#ffffff".to_string(),
        font_path: Some("Hiragino Sans GB.ttc".to_string()),
        font_weight: None,
        line_height: None,
        letter_spacing: None,
        text_align: None,
        text_shadow: None,
        linebreak: None,
        visible_width: None,
        allow_newlines: Some(tag == "textarea"),
    });
}

fn is_button_like_class(class_name: &str) -> bool {
    class_name == "btn" || class_name.ends_with("-btn") || class_name.ends_with("-button")
}

fn is_progress_like_class(class_name: &str) -> bool {
    class_name == "progress" || class_name.ends_with("-progress") || class_name.ends_with("-meter")
}

fn is_progress_fill_like_class(class_name: &str) -> bool {
    class_name == "fill" || class_name.ends_with("-fill") || class_name.ends_with("-bar")
}

fn is_scroll_view_like_class(class_name: &str) -> bool {
    class_name == "scroll-view"
        || class_name == "scrollview"
        || class_name == "scroller"
        || class_name.ends_with("-scroll-view")
        || class_name.ends_with("-scroll")
        || class_name.ends_with("-scroller")
}

fn is_dropdown_like_class(class_name: &str) -> bool {
    class_name == "dropdown"
        || class_name == "select"
        || class_name == "combobox"
        || class_name.ends_with("-dropdown")
        || class_name.ends_with("-select")
        || class_name.ends_with("-combobox")
}

fn apply_attribute_state_aliases(node: &mut BuiNode, dom_node: roxmltree::Node<'_, '_>) {
    let has_selected_class = dom_node.attribute("class").is_some_and(|classes| {
        classes
            .split_whitespace()
            .any(|class| class == "active" || class == "selected" || class.ends_with("-active"))
    });
    let has_selected_attribute = dom_node.attribute("aria-selected") == Some("true")
        || dom_node.attribute("aria-current") == Some("page")
        || dom_node.attribute("selected").is_some();
    if has_selected_class || has_selected_attribute {
        push_unique_marker(node, "initial-state:selected");
        if let Some(hovered) = node.state_visuals.get("hovered").cloned() {
            node.state_visuals
                .entry("selected".to_string())
                .or_insert(hovered);
        } else {
            node.state_visuals
                .entry("selected".to_string())
                .or_insert_with(|| crate::core::model::BuiStateVisual {
                    styles: Default::default(),
                    visuals: Default::default(),
                    text_color: None,
                    image: None,
                });
        }
    }

    let has_checked_attribute = dom_node.attribute("checked").is_some()
        || dom_node.attribute("aria-checked") == Some("true");
    if has_checked_attribute {
        push_unique_marker(node, "initial-state:checked");
        node.state_visuals
            .entry("checked".to_string())
            .or_insert_with(|| crate::core::model::BuiStateVisual {
                styles: Default::default(),
                visuals: Default::default(),
                text_color: None,
                image: None,
            });
    }
}

fn push_unique_marker(node: &mut BuiNode, marker: &str) {
    if node.markers.iter().any(|existing| existing == marker) {
        return;
    }
    node.markers.push(marker.to_string());
}

fn apply_button_text_layout_defaults(node: &mut BuiNode) {
    if node.kind != "button" {
        return;
    }

    node.layout
        .styles
        .display
        .get_or_insert_with(|| "flex".to_string());
    node.layout
        .styles
        .align_items
        .get_or_insert_with(|| "center".to_string());
    node.layout
        .styles
        .justify_content
        .get_or_insert_with(|| "center".to_string());
}

fn apply_direct_text_alignment_to_container(node: &mut BuiNode) {
    let Some(text_align) = node
        .children
        .iter()
        .find_map(|child| child.content.text.as_ref()?.text_align.as_deref())
    else {
        return;
    };

    let justify_content = match text_align {
        "center" => "center",
        "right" | "end" => "flex_end",
        "left" | "start" => "flex_start",
        _ => return,
    };

    node.layout
        .styles
        .display
        .get_or_insert_with(|| "flex".to_string());
    node.layout
        .styles
        .justify_content
        .get_or_insert_with(|| justify_content.to_string());
}

fn normalize_implicit_grid_layout(node: &mut BuiNode) {
    if node.layout.styles.display.as_deref() != Some("grid")
        || node.layout.styles.grid_template_columns.is_some()
        || node.layout.styles.grid_template_rows.is_some()
    {
        return;
    }

    node.layout.styles.display = Some("flex".to_string());
    node.layout
        .styles
        .flex_direction
        .get_or_insert_with(|| "column".to_string());
}

fn move_background_padding_to_children(node: &mut BuiNode) {
    if node.content.image.is_none() || node.children.is_empty() || node.kind == "image" {
        return;
    }

    let styles = &mut node.layout.styles;
    let padding_left = styles.padding_left.take();
    let padding_right = styles.padding_right.take();
    let padding_top = styles.padding_top.take();
    let padding_bottom = styles.padding_bottom.take();

    let Some(first_child) = node.children.first_mut() else {
        return;
    };

    if first_child.layout.styles.margin_left.is_none() {
        first_child.layout.styles.margin_left = padding_left;
    }
    if first_child.layout.styles.margin_right.is_none() {
        first_child.layout.styles.margin_right = padding_right;
    }
    if first_child.layout.styles.margin_top.is_none() {
        first_child.layout.styles.margin_top = padding_top;
    }
    if first_child.layout.styles.margin_bottom.is_none() {
        first_child.layout.styles.margin_bottom = padding_bottom;
    }
}

fn is_non_visual_html_tag(tag: &str) -> bool {
    matches!(tag, "script" | "style" | "template" | "noscript")
}

fn append_line_break(parent: &mut BuiNode) {
    if let Some(text_config) = parent
        .children
        .last_mut()
        .and_then(|child| child.content.text.as_mut())
    {
        text_config.content.push('\n');
        text_config.allow_newlines = Some(true);
    }
}

fn append_text_after_line_break(parent: &mut BuiNode, text: &str) -> bool {
    let Some(text_config) = parent
        .children
        .last_mut()
        .and_then(|child| child.content.text.as_mut())
    else {
        return false;
    };

    if !text_config.content.ends_with('\n') {
        return false;
    }

    text_config.content.push_str(text);
    true
}

fn suppress_decorative_gradient_fallbacks(node: &mut BuiNode) {
    let has_class = |class_name: &str| {
        node.markers
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    if has_class("image-layer") {
        node.style.visuals.background_color = None;
    }

    if has_class("hero-glow") {
        node.style.visuals.background_color = None;
    }
}

fn generic_dom_id(
    dom_node: roxmltree::Node<'_, '_>,
    id_counts: &mut HashMap<String, usize>,
) -> String {
    let base = dom_node
        .attribute("id")
        .map(sanitize_id)
        .filter(|id| !id.is_empty())
        .or_else(|| {
            dom_node
                .attribute("class")
                .and_then(|classes| classes.split_whitespace().next())
                .map(sanitize_id)
                .filter(|id| !id.is_empty())
        })
        .unwrap_or_else(|| sanitize_id(dom_node.tag_name().name()));

    let count = id_counts.entry(base.clone()).or_default();
    *count += 1;

    if *count == 1 {
        base
    } else {
        format!("{base}_{}", *count)
    }
}
