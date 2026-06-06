use std::collections::HashMap;

use crate::core::{
    model::{BuiActionBinding, BuiBinding, BuiNode, BuiNodeType, BuiStateVisual},
    style::css_parser::{
        parse_color, parse_rotation, parse_val2, parse_vec2, parse_visibility,
    },
};

use super::style::{validate_styles, validate_visuals};

pub(super) fn validate_actions(actions: &[BuiActionBinding]) -> Result<(), String> {
    for action in actions {
        parse_action_trigger(&action.event)?;
        if action.emit.trim().is_empty() {
            return Err("actions.emit must not be empty.".to_string());
        }
    }
    Ok(())
}

pub(super) fn validate_bindings(bindings: &[BuiBinding]) -> Result<(), String> {
    for binding in bindings {
        let target = binding.target.trim();
        let source = binding.source.trim();
        if target.is_empty() {
            return Err("bindings.target must not be empty.".to_string());
        }
        if source.is_empty() {
            return Err("bindings.source must not be empty.".to_string());
        }
    }
    Ok(())
}

pub(super) fn validate_state_visuals(states: &HashMap<String, BuiStateVisual>) -> Result<(), String> {
    for (name, state) in states {
        if name.trim().is_empty() {
            return Err("state_visuals keys must not be empty.".to_string());
        }
        validate_styles(&state.styles).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        if let Some(value) = &state.styles.visibility {
            parse_visibility(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        if let Some(value) = &state.styles.ui_translation {
            parse_val2(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        if let Some(value) = &state.styles.ui_scale {
            parse_vec2(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        if let Some(value) = &state.styles.ui_rotation {
            parse_rotation(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        validate_visuals(&state.visuals)?;
        if let Some(text_color) = &state.text_color {
            parse_color(text_color)?;
        }
    }
    Ok(())
}

pub(super) fn validate_tab_semantics(node: &BuiNode) -> Result<(), String> {
    if let Some(group) = &node.semantics.tab_group_name
        && group.trim().is_empty()
    {
        return Err("tab_group_name must not be empty when present.".to_string());
    }
    if let Some(source) = &node.semantics.tab_binding_source {
        if source.trim().is_empty() {
            return Err("tab_binding_source must not be empty when present.".to_string());
        }
        if node.semantics.tab_group_name.is_none() {
            return Err("tab_binding_source requires tab_group_name.".to_string());
        }
    }
    if let Some(value) = &node.semantics.tab_value {
        if value.trim().is_empty() {
            return Err("tab_value must not be empty when present.".to_string());
        }
        if node.semantics.tab_group_name.is_none() {
            return Err("tab_value requires tab_group_name.".to_string());
        }
    }
    Ok(())
}

pub(super) fn validate_progress_semantics(node: &BuiNode) -> Result<(), String> {
    if let Some(source) = &node.semantics.progress_binding_source
        && source.trim().is_empty()
    {
        return Err("progress_binding_source must not be empty when present.".to_string());
    }
    if node.semantics.progress_fill && !matches!(node.node_type(), BuiNodeType::Node) {
        return Err("progress_fill is only supported on Node nodes.".to_string());
    }
    Ok(())
}

pub(super) fn validate_list_semantics(node: &BuiNode) -> Result<(), String> {
    if let Some(source) = &node.semantics.list_binding_source {
        if source.trim().is_empty() {
            return Err("list_binding_source must not be empty when present.".to_string());
        }
        if !matches!(node.node_type(), BuiNodeType::Node) {
            return Err("list_binding_source is only supported on Node nodes.".to_string());
        }
        if node.children.is_empty() {
            return Err("list_binding_source requires at least one child template.".to_string());
        }
    }
    Ok(())
}

fn parse_action_trigger(value: &str) -> Result<(), String> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "press" | "pressed" | "hover_enter" | "hovered" | "hover_exit" | "unhovered" => Ok(()),
        _ => Err(format!(
            "Unsupported action event '{}'. Expected one of: press, hover_enter, hover_exit.",
            value
        )),
    }
}