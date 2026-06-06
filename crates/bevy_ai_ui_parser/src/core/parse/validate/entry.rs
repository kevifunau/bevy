use std::{
    collections::HashSet,
    fs,
    path::Path,
};

use crate::core::model::{BuiDocument, BuiNode, BuiNodeType};

use super::{
    node::validate_bui_node,
};
use crate::core::parse::{
    document::parse_bui_document,
    ir::parse_bui_ir_document,
};

pub(crate) const EXPECTED_VERSION: &str = "2.0";

pub(crate) fn validate_bui_json_str(json: &str) -> Result<(), String> {
    parse_bui_document(json).map(|_| ())
}

pub(crate) fn validate_bui_json_file(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read BUI JSON '{}': {error}", path.display()))?;

    validate_bui_json_str(&raw).map_err(|error| format!("{}: {error}", path.display()))
}

pub(crate) fn validate_bui_ir_json_str(json: &str) -> Result<(), String> {
    parse_bui_ir_document(json).and_then(|document| {
        let compat = document.into_compat_document()?;
        validate_bui_document(&compat)
    })
}

pub(crate) fn validate_bui_ir_json_file(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read BUI IR JSON '{}': {error}", path.display()))?;

    validate_bui_ir_json_str(&raw).map_err(|error| format!("{}: {error}", path.display()))
}

pub(crate) fn validate_bui_document(document: &BuiDocument) -> Result<(), String> {
    if document.version != EXPECTED_VERSION {
        return Err(format!(
            "Unsupported BUI version '{}'. This parser expects version {EXPECTED_VERSION}.",
            document.version
        ));
    }

    if document.scene_name.trim().is_empty() {
        return Err("BUI scene_name must not be empty.".to_string());
    }

    if !matches!(document.root.node_type, BuiNodeType::Node) {
        return Err("BUI root must be a Node.".to_string());
    }

    let mut ids = HashSet::new();
    validate_bui_node(&document.root, "root", &mut ids)
}

pub(super) fn reject_children(node: &BuiNode, path: &str) -> Result<(), String> {
    if !node.children.is_empty() {
        return Err(format!(
            "{path}: {:?} nodes cannot have children.",
            node.node_type
        ));
    }
    Ok(())
}

pub(super) fn reject_config(has_config: bool, path: &str, field: &str) -> Result<(), String> {
    if has_config {
        return Err(format!(
            "{path}: field '{field}' is not valid for this node type."
        ));
    }
    Ok(())
}
