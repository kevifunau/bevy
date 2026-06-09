use bevy_ecs::prelude::*;
use bevy_input::prelude::*;
use bevy_log::info;

use crate::core::editor::state::{BuiEditorState, EditorMode};
use crate::core::runtime::components::BuiSourcePaths;

pub(crate) fn toggle_editor_mode_system(
    mut editor_state: ResMut<BuiEditorState>,
    keys: Res<ButtonInput<KeyCode>>,
    source_paths: Res<BuiSourcePaths>,
) {
    if !keys.just_pressed(KeyCode::F8) {
        return;
    }

    match editor_state.mode {
        EditorMode::Disabled => {
            if source_paths.ir_json_path.is_none() {
                return;
            }
            editor_state.mode = EditorMode::Active;
            info!("BUI editor enabled.");
        }
        EditorMode::Active => {
            if editor_state.pending_edits.is_empty() {
                editor_state.mode = EditorMode::Disabled;
                editor_state.clear_session_state();
                info!("BUI editor disabled without pending edits.");
            } else {
                editor_state.mode = EditorMode::AwaitingSaveDialog;
                editor_state.save_requested = false;
                editor_state.discard_requested = false;
                info!("BUI editor exit requested with pending edits; opening save dialog.");
            }
        }
        EditorMode::AwaitingSaveDialog => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::{Res, ResMut, World};
    use std::path::PathBuf;

    use bevy_ecs::system::SystemState;
    use bevy_input::keyboard::KeyCode;
    use bevy_input::ButtonInput;

    use crate::core::editor::state::{BuiEdit, BuiEditorState, EditorMode};
    use crate::core::model::bui_node;
    use crate::core::runtime::components::BuiSourcePaths;

    use super::toggle_editor_mode_system;

    #[test]
    fn f8_enters_editor_when_ir_path_exists() {
        let mut world = World::new();
        world.insert_resource(BuiEditorState::default());
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(BuiSourcePaths {
            ir_json_path: Some(PathBuf::from("/tmp/test.ir.json")),
            html_path: None,
        });

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);

        let mut system_state = SystemState::<(
            ResMut<BuiEditorState>,
            Res<ButtonInput<KeyCode>>,
            Res<BuiSourcePaths>,
        )>::new(&mut world);
        let Ok((editor_state, keys, source_paths)) = system_state.get_mut(&mut world) else {
            panic!("editor toggle system state should be available");
        };
        toggle_editor_mode_system(editor_state, keys, source_paths);

        assert_eq!(world.resource::<BuiEditorState>().mode, EditorMode::Active);
    }

    #[test]
    fn f8_without_edits_exits_editor() {
        let mut world = World::new();
        let editor_state = BuiEditorState {
            mode: EditorMode::Active,
            hovered_node_id: Some("node".to_string()),
            ..BuiEditorState::default()
        };
        world.insert_resource(editor_state);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(BuiSourcePaths {
            ir_json_path: Some(PathBuf::from("/tmp/test.ir.json")),
            html_path: None,
        });

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);

        let mut system_state = SystemState::<(
            ResMut<BuiEditorState>,
            Res<ButtonInput<KeyCode>>,
            Res<BuiSourcePaths>,
        )>::new(&mut world);
        let Ok((editor_state, keys, source_paths)) = system_state.get_mut(&mut world) else {
            panic!("editor toggle system state should be available");
        };
        toggle_editor_mode_system(editor_state, keys, source_paths);

        let editor_state = world.resource::<BuiEditorState>();
        assert_eq!(editor_state.mode, EditorMode::Disabled);
        assert_eq!(editor_state.hovered_node_id, None);
    }

    #[test]
    fn f8_with_pending_edits_opens_save_dialog_mode() {
        let mut world = World::new();
        let editor_state = BuiEditorState {
            mode: EditorMode::Active,
            pending_edits: vec![BuiEdit::NodeDeleted {
                node_id: "node".to_string(),
                parent_id: "root".to_string(),
                deleted_subtree: bui_node("node", "node"),
            }],
            ..BuiEditorState::default()
        };
        world.insert_resource(editor_state);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(BuiSourcePaths {
            ir_json_path: Some(PathBuf::from("/tmp/test.ir.json")),
            html_path: None,
        });

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);

        let mut system_state = SystemState::<(
            ResMut<BuiEditorState>,
            Res<ButtonInput<KeyCode>>,
            Res<BuiSourcePaths>,
        )>::new(&mut world);
        let Ok((editor_state, keys, source_paths)) = system_state.get_mut(&mut world) else {
            panic!("editor toggle system state should be available");
        };
        toggle_editor_mode_system(editor_state, keys, source_paths);

        assert_eq!(
            world.resource::<BuiEditorState>().mode,
            EditorMode::AwaitingSaveDialog
        );
    }
}
