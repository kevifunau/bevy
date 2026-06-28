use crate::core::{
    model::BuiNode,
    opendesign::stylesheet::{css_declarations, OpenDesignStylesheet},
    style::css_apply::{apply_opendesign_declaration, apply_opendesign_state_declaration},
};

pub(crate) fn apply_opendesign_styles(
    stylesheet: &OpenDesignStylesheet,
    bui_node: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
) {
    let custom_properties = stylesheet.custom_properties_for_node(dom_node);
    let mut normal_declarations = Vec::new();

    for (name, value) in stylesheet.matching_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        normal_declarations.push((name.clone(), value.clone()));
        apply_opendesign_declaration(bui_node, name, &value);
    }

    for (state, (name, value)) in stylesheet.matching_state_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        apply_opendesign_state_declaration(bui_node, state, name, &value);
    }

    if let Some(inline_style) = dom_node.attribute("style") {
        for (name, value) in css_declarations(inline_style) {
            let value = stylesheet.resolve_value_with_variables(&value, &custom_properties);
            normal_declarations.push((name.clone(), value.clone()));
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }

    for (name, value) in normal_declarations {
        if matches!(name.as_str(), "background-size" | "background-position") {
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }
}

/// Like `apply_opendesign_styles` but skips universal-selector (`*`) declarations.
/// Used for text nodes, where universal defaults are already applied by
/// `apply_inherited_text_styles` (Pass 1). Applying them again here would
/// override inherited class values from ancestor nodes (e.g. a parent's
/// `.line { line-height: 64px }` would be clobbered by `* { line-height: 1.2 }`
/// matched on the direct parent element).
pub(crate) fn apply_opendesign_styles_for_text(
    stylesheet: &OpenDesignStylesheet,
    bui_node: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
) {
    let custom_properties = stylesheet.custom_properties_for_node(dom_node);
    let mut normal_declarations = Vec::new();

    let (_universal, specific) = stylesheet.matching_declarations_split_by_universality(dom_node);
    for (name, value) in specific {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        normal_declarations.push((name.clone(), value.clone()));
        apply_opendesign_declaration(bui_node, name, &value);
    }

    for (state, (name, value)) in stylesheet.matching_state_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        apply_opendesign_state_declaration(bui_node, state, name, &value);
    }

    if let Some(inline_style) = dom_node.attribute("style") {
        for (name, value) in css_declarations(inline_style) {
            let value = stylesheet.resolve_value_with_variables(&value, &custom_properties);
            normal_declarations.push((name.clone(), value.clone()));
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }

    for (name, value) in normal_declarations {
        if matches!(name.as_str(), "background-size" | "background-position") {
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }
}
