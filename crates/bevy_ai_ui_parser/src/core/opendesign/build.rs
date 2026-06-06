use std::collections::HashMap;

use crate::core::{
    model::{
        BuiNode,
        BuiNodeType,
        BuiStateVisual,
        BuiStyles,
        BuiTextConfig,
        BuiVisuals,
    },
    opendesign::stylesheet::{OpenDesignStylesheet, css_declarations},
    style::css_apply::{apply_opendesign_declaration, apply_opendesign_state_declaration},
};

pub(crate) fn ensure_state_visual<'a>(node: &'a mut BuiNode, state: &str) -> &'a mut BuiStateVisual {
    node.state_visuals
        .entry(state.to_string())
        .or_insert_with(|| BuiStateVisual {
            styles: BuiStyles::default(),
            visuals: BuiVisuals::default(),
            text_color: None,
        })
}

pub(crate) fn bui_node(id: &str, node_type: BuiNodeType) -> BuiNode {
    BuiNode {
        id: id.to_string(),
        node_type,
        custom_tags: Vec::new(),
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    }
}

pub(crate) fn text_node(
    id: &str,
    content: impl Into<String>,
    font_size: f32,
    font_color: &str,
    font_path: Option<&str>,
) -> BuiNode {
    let mut node = bui_node(id, BuiNodeType::Text);
    node.text_config = Some(BuiTextConfig {
        content: content.into(),
        placeholder: None,
        font_size,
        font_color: font_color.to_string(),
        font_path: font_path.map(str::to_string),
        font_weight: None,
        line_height: None,
        letter_spacing: None,
        text_align: None,
        text_shadow: None,
        linebreak: None,
        visible_width: None,
        allow_newlines: None,
    });
    node
}

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
