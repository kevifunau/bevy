use bevy_ecs::prelude::*;
use bevy_log::{error, info};

use crate::core::editor::{
    persistence::save_ir,
    state::{BuiEditorState, EditorMode},
};
use crate::core::runtime::components::{BuiDocumentResource, BuiSourcePaths};

pub(crate) fn editor_save_system(
    mut editor_state: ResMut<BuiEditorState>,
    doc_resource: Res<BuiDocumentResource>,
    source_paths: Res<BuiSourcePaths>,
) {
    if editor_state.mode != EditorMode::AwaitingSaveDialog {
        return;
    }

    if !editor_state.save_requested {
        return;
    }

    editor_state.save_requested = false;

    let Some(path) = &source_paths.ir_json_path else {
        error!("No IR JSON path configured — cannot save.");
        return;
    };

    match save_ir(&doc_resource.0, path) {
        Ok(()) => {
            info!("Saved IR JSON to '{}'.", path.display());
            editor_state.pending_edits.clear();
            editor_state.mode = EditorMode::Disabled;
            editor_state.clear_session_state();
        }
        Err(error) => {
            error!("{error}");
        }
    }
}
