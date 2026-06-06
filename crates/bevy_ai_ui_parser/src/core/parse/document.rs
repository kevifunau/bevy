use crate::core::{
    model::BuiDocument,
};

use super::{
    ir::{detect_bui_version, parse_bui_ir_document},
    validate::validate_bui_document,
};

pub(crate) fn parse_bui_document(raw: &str) -> Result<BuiDocument, String> {
    let version = detect_bui_version(raw)?;
    let document = if version == "3.0-ir" {
        parse_bui_ir_document(raw)?.into_compat_document()?
    } else {
        serde_json::from_str(raw).map_err(|error| format!("Invalid BUI JSON: {error}"))?
    };

    validate_bui_document(&document)?;

    Ok(document)
}
