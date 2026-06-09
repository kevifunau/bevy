use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_log::{error, info};

use crate::core::editor::state::{BuiEditorState, EditorMode};
use crate::core::interaction::types::{BuiBindingValue, BuiStateStore};
use crate::core::runtime::components::{BuiDocumentResource, BuiIdMap, BuiRootEntity};
use crate::core::runtime::plugin::{load_bui_document, AiUiSource};
use crate::core::runtime::spawn::spawn_bui_tree;

pub(crate) fn editor_discard_system(
    mut editor_state: ResMut<BuiEditorState>,
    source: Res<AiUiSource>,
    root_entity: Res<BuiRootEntity>,
    mut id_map: ResMut<BuiIdMap>,
    mut doc_resource: ResMut<BuiDocumentResource>,
    mut state_store: ResMut<BuiStateStore>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    if editor_state.mode != EditorMode::AwaitingSaveDialog {
        return;
    }

    if !editor_state.discard_requested {
        return;
    }

    editor_state.discard_requested = false;
    commands.entity(root_entity.0).despawn();

    match load_bui_document(&source.0) {
        Ok(document) => {
            info!("Reloaded BUI scene '{}' from disk.", document.scene_name);

            state_store.0.clear();
            for (key, value) in &document.state_model.values {
                state_store
                    .0
                    .insert(key.clone(), BuiBindingValue::Text(value.clone()));
            }

            match spawn_bui_tree(
                &mut commands,
                &asset_server,
                &mut texture_atlases,
                &document,
            ) {
                Ok((root, new_id_map)) => {
                    commands.insert_resource(BuiRootEntity(root));
                    *doc_resource = BuiDocumentResource(document);
                    *id_map = BuiIdMap(new_id_map);
                    editor_state.pending_edits.clear();
                    editor_state.mode = EditorMode::Disabled;
                    editor_state.clear_session_state();
                }
                Err(error) => {
                    error!("{error}");
                }
            }
        }
        Err(error) => {
            error!("Failed to reload BUI document: {error}");
        }
    }
}
