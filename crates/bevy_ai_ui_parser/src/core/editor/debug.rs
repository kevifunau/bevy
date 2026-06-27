use bevy_ecs::prelude::*;

use crate::core::editor::state::{BuiEditorState, EditorMode};

pub(crate) fn force_debug_hover_node_system(
    mut editor_state: ResMut<BuiEditorState>,
    mut debug_hover_node_id: Local<Option<Option<String>>>,
) {
    if editor_state.mode != EditorMode::Active {
        return;
    }

    let configured_node_id = debug_hover_node_id
        .get_or_insert_with(read_debug_hover_node_id)
        .clone();

    let Some(node_id) = configured_node_id else {
        return;
    };

    if editor_state.dragged_node_id.is_none() {
        debug_trace(&format!("forcing hovered node '{node_id}'"));
        editor_state.hovered_node_id = Some(node_id);
    }
}

fn read_debug_hover_node_id() -> Option<String> {
    std::env::var("BUI_EDITOR_DEBUG_HOVER_NODE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
