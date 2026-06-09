use bevy_ecs::prelude::*;

use crate::core::model::BuiNode;

#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct BuiEditorState {
    pub(crate) mode: EditorMode,
    pub(crate) hovered_node_id: Option<String>,
    pub(crate) dragged_node_id: Option<String>,
    pub(crate) drag_origin_cursor: Option<(f32, f32)>,
    pub(crate) drag_origin_pos: Option<(f32, f32)>,
    pub(crate) drag_origin_style_left: Option<String>,
    pub(crate) drag_origin_style_top: Option<String>,
    pub(crate) drag_current_left: Option<f32>,
    pub(crate) drag_current_top: Option<f32>,
    pub(crate) drag_has_moved: bool,
    pub(crate) pending_edits: Vec<BuiEdit>,
    pub(crate) overlay_root_entity: Option<Entity>,
    pub(crate) close_icon_entity: Option<Entity>,
    pub(crate) dialog_entity: Option<Entity>,
    pub(crate) save_requested: bool,
    pub(crate) discard_requested: bool,
}

impl BuiEditorState {
    pub(crate) fn clear_session_state(&mut self) {
        self.hovered_node_id = None;
        self.dragged_node_id = None;
        self.drag_origin_cursor = None;
        self.drag_origin_pos = None;
        self.drag_origin_style_left = None;
        self.drag_origin_style_top = None;
        self.drag_current_left = None;
        self.drag_current_top = None;
        self.drag_has_moved = false;
        self.close_icon_entity = None;
        self.dialog_entity = None;
        self.overlay_root_entity = None;
        self.save_requested = false;
        self.discard_requested = false;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum EditorMode {
    #[default]
    Disabled,
    Active,
    AwaitingSaveDialog,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum BuiEdit {
    PositionChange {
        node_id: String,
        old_left: String,
        new_left: String,
        old_top: String,
        new_top: String,
    },
    NodeDeleted {
        node_id: String,
        parent_id: String,
        deleted_subtree: BuiNode,
    },
}

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BuiEditorBorder {
    pub(crate) node_id: String,
    pub(crate) target_entity: Entity,
}

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BuiEditorCloseIcon {
    pub(crate) node_id: String,
}

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BuiEditorOverlayRoot;

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BuiEditorSaveButton;

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BuiEditorDiscardButton;

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BuiEditorDialogPanel;
