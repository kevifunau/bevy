use crate::core::{
    model::BuiNode,
    opendesign::stylesheet::{OpenDesignStylesheet, css_declarations},
    style::css_apply::{apply_opendesign_declaration, apply_opendesign_state_declaration},
};

pub(crate) fn apply_opendesign_styles(
    stylesheet: &OpenDesignStylesheet,
    bui_node: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
) {
    let custom_properties = stylesheet.custom_properties_for_node(dom_node);

    for (name, value) in stylesheet.matching_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        apply_opendesign_declaration(bui_node, name, &value);
    }

    for (state, (name, value)) in stylesheet.matching_state_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        apply_opendesign_state_declaration(bui_node, state, name, &value);
    }

    if let Some(inline_style) = dom_node.attribute("style") {
        for (name, value) in css_declarations(inline_style) {
            let value = stylesheet.resolve_value_with_variables(&value, &custom_properties);
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }
}