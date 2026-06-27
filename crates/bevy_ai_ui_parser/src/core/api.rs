use std::{fs, path::Path};

use crate::core::{
    opendesign::{
        html::opendesign_html_to_bui_document_with_manifest,
        manifest::{discover_manifest_path, load_manifest_file},
    },
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
    let document = opendesign_html_to_bui_document_with_manifest(html, None, None)?;
    serde_json::to_string_pretty(&document)
        .map_err(|error| format!("Failed to serialize generated BUI JSON: {error}"))
}

/// Compile an OpenDesign HTML artifact plus asset manifest JSON into formatted BUI JSON.
pub fn opendesign_html_to_bui_json_str_with_manifest(
    html: &str,
    manifest_json: &str,
) -> Result<String, String> {
    let manifest = serde_json::from_str(manifest_json)
        .map_err(|error| format!("Failed to parse OpenDesign asset manifest JSON: {error}"))?;
    let document = opendesign_html_to_bui_document_with_manifest(html, Some(&manifest), None)?;
    serde_json::to_string_pretty(&document)
        .map_err(|error| format!("Failed to serialize generated BUI JSON: {error}"))
}

/// Compile an OpenDesign HTML artifact file into formatted BUI JSON.
pub fn opendesign_html_file_to_bui_json(path: impl AsRef<Path>) -> Result<String, String> {
    opendesign_html_file_to_bui_json_with_manifest(path, None::<&Path>)
}

/// Compile an OpenDesign HTML artifact file into formatted BUI JSON with an optional asset manifest file.
pub fn opendesign_html_file_to_bui_json_with_manifest(
    path: impl AsRef<Path>,
    manifest_path: Option<impl AsRef<Path>>,
) -> Result<String, String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "Failed to read OpenDesign HTML '{}': {error}",
            path.display()
        )
    })?;
    let manifest_path = manifest_path
        .map(|value| value.as_ref().to_path_buf())
        .or_else(|| discover_manifest_path(path));
    let manifest = manifest_path
        .as_deref()
        .map(load_manifest_file)
        .transpose()?;
    let document =
        opendesign_html_to_bui_document_with_manifest(&raw, manifest.as_ref(), path.parent())?;
    serde_json::to_string_pretty(&document)
        .map_err(|error| format!("Failed to serialize generated BUI JSON: {error}"))
}
