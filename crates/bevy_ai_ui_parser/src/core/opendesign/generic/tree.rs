use std::collections::HashMap;

use crate::core::{
    model::{BuiActionBinding, BuiNode, bui_node, text_node},
    opendesign::{
        build::apply_opendesign_styles,
        stylesheet::OpenDesignStylesheet,
        svg::{is_svg_tag, svg_fallback_text_node},
    },
    support::ids::sanitize_id,
};

use super::text::{apply_inherited_text_styles, propagate_direct_text_state_visuals};

pub(crate) fn generic_append_children(
    parent: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
    id_counts: &mut HashMap<String, usize>,
) {
    let before_decls = stylesheet.matching_pseudo_declarations(dom_node, "before");
    if !before_decls.is_empty() {
        let mut pseudo_node = bui_node(&format!("{}_pseudo_before", parent.id), "node");
        pseudo_node.markers.push("pseudo:before".to_string());
        for (name, value) in &before_decls {
            let value = stylesheet.resolve_value(value);
            crate::core::style::css_apply::apply_opendesign_declaration(&mut pseudo_node, name, &value);
        }
        parent.children.push(pseudo_node);
    }

    let mut direct_text_index = 0;
    let mut svg_fallback_index = 0;

    for child in dom_node.children() {
        if child.is_element() {
            if is_svg_tag(child.tag_name().name()) {
                if child.tag_name().name() == "svg"
                    && let Some(mut svg_fallback) =
                        svg_fallback_text_node(parent, child, stylesheet, svg_fallback_index + 1)
                {
                    svg_fallback_index += 1;
                    apply_inherited_text_styles(stylesheet, &mut svg_fallback, child);
                    apply_opendesign_styles(stylesheet, &mut svg_fallback, child);
                    parent.children.push(svg_fallback);
                }
                continue;
            }
            let id = generic_dom_id(child, id_counts);
            let kind = generic_node_kind(child);
            let mut child_node = generic_element_node(&id, kind, stylesheet, child);
            generic_append_children(&mut child_node, child, stylesheet, id_counts);
            parent.children.push(child_node);
        } else if child.node_type() == roxmltree::NodeType::Comment {
            continue;
        } else if let Some(text) = child.text().map(str::trim).filter(|text| !text.is_empty()) {
            direct_text_index += 1;
            let mut text_child = text_node(
                &format!("{}_text_{}", parent.id, direct_text_index),
                text,
                16.0,
                "#3B2818",
                Some("Hiragino Sans GB.ttc"),
            );
            apply_inherited_text_styles(stylesheet, &mut text_child, dom_node);
            apply_opendesign_styles(stylesheet, &mut text_child, dom_node);
            parent.children.push(text_child);
        }
    }

    propagate_direct_text_state_visuals(parent);

    let after_decls = stylesheet.matching_pseudo_declarations(dom_node, "after");
    if !after_decls.is_empty() {
        let mut pseudo_node = bui_node(&format!("{}_pseudo_after", parent.id), "node");
        pseudo_node.markers.push("pseudo:after".to_string());
        for (name, value) in &after_decls {
            let value = stylesheet.resolve_value(value);
            crate::core::style::css_apply::apply_opendesign_declaration(&mut pseudo_node, name, &value);
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
        .attribute("aria-label")
        .filter(|value| !value.trim().is_empty())
    {
        node.markers.push(format!("aria-label:{value}"));
    }

    if let Some(action) = dom_node.attribute("data-action") {
        node.actions.push(BuiActionBinding {
            event: "press".to_string(),
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
        node.semantics.tab_value = Some(value.to_string());
    }
    apply_opendesign_styles(stylesheet, &mut node, dom_node);
    suppress_decorative_gradient_fallbacks(&mut node);
    node
}

fn generic_node_kind(dom_node: roxmltree::Node<'_, '_>) -> &'static str {
    let tag = dom_node.tag_name().name();
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

fn is_button_like_class(class_name: &str) -> bool {
    class_name == "btn" || class_name.ends_with("-btn") || class_name.ends_with("-button")
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
