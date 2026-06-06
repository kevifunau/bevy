use serde::Deserialize;

use crate::core::model::BuiIrDocument;

pub(crate) fn parse_bui_ir_document(raw: &str) -> Result<BuiIrDocument, String> {
    serde_json::from_str(raw).map_err(|error| format!("Invalid BUI IR JSON: {error}"))
}

pub(crate) fn detect_bui_version(raw: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct VersionProbe {
        version: String,
    }

    let probe: VersionProbe =
        serde_json::from_str(raw).map_err(|error| format!("Invalid BUI JSON: {error}"))?;
    Ok(probe.version)
}
