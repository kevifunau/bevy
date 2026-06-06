use crate::core::{
    model::{BuiNode, BuiNodeType, ensure_state_visual},
    opendesign::{
        stylesheet::{css_declarations, OpenDesignStylesheet},
    },
    style::css_apply::{apply_opendesign_declaration, apply_opendesign_state_declaration},
};

pub(crate) fn propagate_direct_text_state_visuals(node: &mut BuiNode) {
    let Some(text_index) = node
        .children
        .iter()
        .position(|child| matches!(child.node_type, BuiNodeType::Text))
    else {
        return;
    };

    if node.state_visuals.is_empty() {
        return;
    }

    let text_child = &mut node.children[text_index];
    for (state_name, state_visual) in &node.state_visuals {
        let has_textual_state =
            state_visual.text_color.is_some() || state_visual.styles.visibility.is_some();
        if !has_textual_state {
            continue;
        }

        let text_state = ensure_state_visual(text_child, state_name);
        if text_state.text_color.is_none() {
            text_state.text_color = state_visual.text_color.clone();
        }
        if text_state.styles.visibility.is_none() {
            text_state.styles.visibility = state_visual.styles.visibility.clone();
        }
    }
}

pub(crate) fn apply_inherited_text_styles(
    stylesheet: &OpenDesignStylesheet,
    bui_node: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
) {
    if !matches!(bui_node.node_type, BuiNodeType::Text) {
        return;
    }

    let mut ancestors = dom_node
        .ancestors()
        .filter(|node| node.is_element())
        .collect::<Vec<_>>();
    ancestors.reverse();

    for ancestor in ancestors {
        let custom_properties = stylesheet.custom_properties_for_node(ancestor);
        for (name, value) in stylesheet.matching_declarations(ancestor) {
            if !is_inheritable_text_property(name) {
                continue;
            }
            let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
            apply_opendesign_declaration(bui_node, name, &value);
        }

        for (state, (name, value)) in stylesheet.matching_state_declarations(ancestor) {
            if !is_inheritable_text_property(name) {
                continue;
            }
            let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
            apply_opendesign_state_declaration(bui_node, state, name, &value);
        }

        if let Some(inline_style) = ancestor.attribute("style") {
            for (name, value) in css_declarations(inline_style) {
                if !is_inheritable_text_property(&name) {
                    continue;
                }
                let value = stylesheet.resolve_value_with_variables(&value, &custom_properties);
                apply_opendesign_declaration(bui_node, &name, &value);
            }
        }
    }
}

fn is_inheritable_text_property(name: &str) -> bool {
    matches!(
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
}
