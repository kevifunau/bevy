use bevy_ecs::prelude::*;
use bevy_log::info;

use crate::core::{
    editor::state::{BuiEditorState, EditorMode},
    runtime::components::BuiSourcePaths,
};

pub(crate) fn maybe_enable_editor_on_first_frame_system(
    mut editor_state: ResMut<BuiEditorState>,
    source_paths: Res<BuiSourcePaths>,
    mut checked: Local<bool>,
) {
    if *checked {
        return;
    }
    *checked = true;

    let auto_enable = std::env::var("BUI_EDITOR_AUTO_ENABLE")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);

    if !auto_enable
        || source_paths.ir_json_path.is_none()
        || editor_state.mode != EditorMode::Disabled
    {
        return;
    }

    editor_state.mode = EditorMode::Active;
    debug_trace("auto-enabled editor on first frame");
    info!("BUI editor auto-enabled on first frame.");
}

fn debug_trace(message: &str) {
    let enabled = std::env::var("BUI_EDITOR_DEBUG_TRACE")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);

    if enabled {
        eprintln!("[bui-editor-debug] {message}");
    }
}
