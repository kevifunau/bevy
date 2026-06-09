use bevy_ecs::prelude::*;
use bevy_input::prelude::*;
use bevy_picking::hover::HoverMap;

use crate::core::editor::state::{BuiEdit, BuiEditorCloseIcon, BuiEditorState, EditorMode};
use crate::core::runtime::components::{BuiDocumentResource, BuiIdMap};
use crate::core::support::tree::{collect_bui_subtree_ids, find_bui_parent_id, remove_bui_node};

pub(crate) fn editor_delete_system(
    mut editor_state: ResMut<BuiEditorState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    hover_map: Res<HoverMap>,
    close_icon_query: Query<(Entity, &BuiEditorCloseIcon), With<BuiEditorCloseIcon>>,
    mut id_map: ResMut<BuiIdMap>,
    mut doc_resource: ResMut<BuiDocumentResource>,
    mut commands: Commands,
) {
    if editor_state.mode != EditorMode::Active {
        return;
    }

    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let clicked_icon = close_icon_query.iter().find(|(entity, _)| {
        hover_map
            .iter()
            .any(|(_, hovered)| hovered.contains_key(entity))
    });

    let Some((icon_entity, icon_data)) = clicked_icon else {
        return;
    };

    let node_id = icon_data.node_id.clone();
    let deleted = delete_node_by_id(
        &node_id,
        &mut editor_state,
        &mut id_map,
        &mut doc_resource,
        &mut commands,
    );
    if !deleted {
        return;
    }
    let _ = icon_entity;
}

pub(crate) fn delete_node_by_id(
    node_id: &str,
    editor_state: &mut BuiEditorState,
    id_map: &mut BuiIdMap,
    doc_resource: &mut BuiDocumentResource,
    commands: &mut Commands,
) -> bool {
    if let Some(close_icon_entity) = editor_state.close_icon_entity.take() {
        commands.entity(close_icon_entity).despawn();
    }

    let Some(parent_id) = find_bui_parent_id(&doc_resource.0.root, node_id)
        .or_else(|| Some(doc_resource.0.root.id.clone()))
    else {
        return false;
    };

    let Some(deleted_subtree) = remove_bui_node(&mut doc_resource.0.root, node_id) else {
        return false;
    };

    editor_state.pending_edits.push(BuiEdit::NodeDeleted {
        node_id: node_id.to_string(),
        parent_id,
        deleted_subtree: deleted_subtree.clone(),
    });

    let mut ids_to_remove = Vec::new();
    collect_bui_subtree_ids(&deleted_subtree, &mut ids_to_remove);

    let root_entity = id_map.0.remove(node_id);
    for id in ids_to_remove {
        if id == node_id {
            continue;
        }
        id_map.0.remove(&id);
    }

    if let Some(entity) = root_entity {
        commands.entity(entity).despawn();
    }

    editor_state.hovered_node_id = None;
    true
}
