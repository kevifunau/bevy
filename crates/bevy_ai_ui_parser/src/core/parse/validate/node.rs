use std::collections::HashSet;

use crate::core::{
    model::{BuiNode, BuiNodeType},
    style::css_parser::{parse_integer, parse_visibility},
};

use super::{
    entry::{reject_children, reject_config},
    semantics::{
        validate_actions, validate_bindings, validate_list_semantics, validate_progress_semantics,
        validate_state_visuals, validate_tab_semantics,
    },
    style::{validate_image_config, validate_styles, validate_text_config, validate_visuals},
};

pub(super) fn validate_bui_node(
    node: &BuiNode,
    path: &str,
    ids: &mut HashSet<String>,
) -> Result<(), String> {
    if node.id.trim().is_empty() {
        return Err(format!("{path}: id must not be empty."));
    }

    if !ids.insert(node.id.clone()) {
        return Err(format!("{path}: duplicate id '{}'.", node.id));
    }

    validate_styles(&node.layout.styles).map_err(|error| format!("{path}: {error}"))?;
    validate_visuals(&node.style.visuals).map_err(|error| format!("{path}: {error}"))?;
    if let Some(value) = &node.layout.styles.visibility {
        parse_visibility(value).map_err(|error| format!("{path}: {error}"))?;
    }
    if let Some(value) = &node.layout.styles.z_index {
        parse_integer(value).map_err(|error| format!("{path}: {error}"))?;
    }
    if let Some(value) = &node.layout.styles.global_z_index {
        parse_integer(value).map_err(|error| format!("{path}: {error}"))?;
    }
    validate_visuals(&node.style.visuals).map_err(|error| format!("{path}: {error}"))?;
    validate_actions(&node.actions).map_err(|error| format!("{path}: {error}"))?;
    validate_bindings(&node.bindings).map_err(|error| format!("{path}: {error}"))?;
    validate_state_visuals(&node.state_visuals).map_err(|error| format!("{path}: {error}"))?;
    validate_tab_semantics(node).map_err(|error| format!("{path}: {error}"))?;
    validate_progress_semantics(node).map_err(|error| format!("{path}: {error}"))?;
    validate_list_semantics(node).map_err(|error| format!("{path}: {error}"))?;

    match node.node_type() {
        BuiNodeType::Node | BuiNodeType::Button | BuiNodeType::Toggle => {
            reject_config(node.content.text.is_some(), path, "text_config")?;
            if !matches!(node.node_type(), BuiNodeType::Node | BuiNodeType::Button) {
                reject_config(node.content.image.is_some(), path, "image_config")?;
            }
            if let Some(image_config) = &node.content.image {
                validate_image_config(image_config).map_err(|error| format!("{path}: {error}"))?;
            }
        }
        BuiNodeType::Text => {
            let text_config = node
                .content
                .text
                .as_ref()
                .ok_or_else(|| format!("{path}: Text requires text_config."))?;
            validate_text_config(text_config).map_err(|error| format!("{path}: {error}"))?;
            reject_config(
                text_config.placeholder.is_some(),
                path,
                "text_config.placeholder",
            )?;
            reject_config(node.content.image.is_some(), path, "image_config")?;
            reject_children(node, path)?;
        }
        BuiNodeType::TextInput => {
            let text_config = node
                .content
                .text
                .as_ref()
                .ok_or_else(|| format!("{path}: TextInput requires text_config."))?;
            validate_text_config(text_config).map_err(|error| format!("{path}: {error}"))?;
            reject_config(node.content.image.is_some(), path, "image_config")?;
            reject_children(node, path)?;
        }
        BuiNodeType::Image => {
            let image_config = node
                .content
                .image
                .as_ref()
                .ok_or_else(|| format!("{path}: Image requires image_config."))?;
            validate_image_config(image_config).map_err(|error| format!("{path}: {error}"))?;
            reject_config(node.content.text.is_some(), path, "text_config")?;
            reject_children(node, path)?;
        }
    }

    for (index, child) in node.children.iter().enumerate() {
        validate_bui_node(child, &format!("{path}.children[{index}]"), ids)?;
    }

    Ok(())
}
