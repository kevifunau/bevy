use std::{fs, path::Path};

use crate::core::model::BuiDocument;

pub(crate) fn save_ir(document: &BuiDocument, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(document)
        .map_err(|error| format!("Failed to serialize BuiDocument: {error}"))?;
    fs::write(path, json)
        .map_err(|error| format!("Failed to write IR JSON to '{}': {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::core::model::bui_node;

    use super::save_ir;

    #[test]
    fn save_ir_writes_pretty_printed_document() {
        let path = env::temp_dir().join(format!(
            "bevy_ai_ui_parser_save_ir_{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        ));
        let document = crate::core::model::BuiDocument {
            version: "3.0-ir".to_string(),
            scene_name: "EditorSaveTest".to_string(),
            imports: Vec::new(),
            state_model: crate::core::model::BuiStateModel::default(),
            resources: crate::core::model::BuiResources::default(),
            root: bui_node("root", "node"),
        };

        save_ir(&document, &path).expect("IR JSON should be saved");
        let written = fs::read_to_string(&path).expect("saved IR JSON should be readable");

        assert!(written.contains("\"version\": \"3.0-ir\""));
        assert!(written.contains("\"scene_name\": \"EditorSaveTest\""));

        let _ = fs::remove_file(path);
    }
}
