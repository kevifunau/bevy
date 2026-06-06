use std::{
    fs,
    path::Path,
};

use crate::core::{
    opendesign::html::opendesign_html_to_bui_document,
    parse::validate::{
        validate_bui_json_file as parse_validate_bui_json_file,
        validate_bui_json_str as parse_validate_bui_json_str,
    },
};

/// Validate a BUI JSON string against the parser contract without spawning UI.
pub fn validate_bui_json_str(json: &str) -> Result<(), String> {
    parse_validate_bui_json_str(json)
}

/// Validate a BUI JSON file against the parser contract without spawning UI.
pub fn validate_bui_json_file(path: impl AsRef<Path>) -> Result<(), String> {
    parse_validate_bui_json_file(path)
}

/// Compile an OpenDesign HTML artifact into formatted BUI JSON.
pub fn opendesign_html_to_bui_json_str(html: &str) -> Result<String, String> {
    let document = opendesign_html_to_bui_document(html)?;
    serde_json::to_string_pretty(&document)
        .map_err(|error| format!("Failed to serialize generated BUI JSON: {error}"))
}

/// Compile an OpenDesign HTML artifact file into formatted BUI JSON.
pub fn opendesign_html_file_to_bui_json(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "Failed to read OpenDesign HTML '{}': {error}",
            path.display()
        )
    })?;

    opendesign_html_to_bui_json_str(&raw).map_err(|error| format!("{}: {error}", path.display()))
}