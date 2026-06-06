use crate::core::model::BuiDocument;

pub(crate) fn parse_bui_document(raw: &str) -> Result<BuiDocument, String> {
    serde_json::from_str(raw).map_err(|error| format!("Invalid BUI JSON: {error}"))
}