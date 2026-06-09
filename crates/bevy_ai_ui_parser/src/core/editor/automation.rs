use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::*;
use bevy_math::Rect;
use bevy_ui::prelude::*;
use bevy_ui::ui_transform::UiGlobalTransform;
use bevy_ui::ComputedNode;

use crate::core::editor::delete::delete_node_by_id;
use crate::core::editor::state::{BuiEdit, BuiEditorState, EditorMode};
use crate::core::model::BuiDocument;
use crate::core::runtime::components::{BuiDocumentResource, BuiIdMap};
use crate::core::support::tree::{find_bui_node_mut, find_bui_node_ref};

pub(crate) fn run_debug_automation_system(
    mut editor_state: ResMut<BuiEditorState>,
    mut id_map: ResMut<BuiIdMap>,
    mut doc_resource: ResMut<BuiDocumentResource>,
    mut target_node_query: Query<&mut Node>,
    target_layout_query: Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
    child_of_query: Query<&ChildOf>,
    mut commands: Commands,
    mut automation_state: Local<DebugAutomationState>,
) {
    let spec = automation_state
        .spec
        .get_or_insert_with(read_debug_automation_spec);
    let Some(spec) = spec.clone() else {
        return;
    };

    if !automation_state.operation_applied {
        if editor_state.mode != EditorMode::Active {
            return;
        }

        let applied = match &spec.operation {
            DebugAutomationOperation::Move { delta_x, delta_y } => apply_move_spec(
                &spec.node_id,
                *delta_x,
                *delta_y,
                &mut editor_state,
                &id_map,
                &mut doc_resource.0,
                &mut target_node_query,
                &target_layout_query,
                &child_of_query,
            ),
            DebugAutomationOperation::Delete => delete_node_by_id(
                &spec.node_id,
                &mut editor_state,
                &mut id_map,
                &mut doc_resource,
                &mut commands,
            ),
        };

        if !applied {
            return;
        }

        automation_state.operation_applied = true;
        return;
    }

    if automation_state.exit_applied {
        return;
    }

    match spec.exit_action {
        Some(DebugExitAction::Save) => {
            if editor_state.mode == EditorMode::Active {
                editor_state.mode = EditorMode::AwaitingSaveDialog;
                return;
            }
            if editor_state.mode == EditorMode::AwaitingSaveDialog {
                editor_state.save_requested = true;
                automation_state.exit_applied = true;
            }
        }
        Some(DebugExitAction::Discard) => {
            if editor_state.mode == EditorMode::Active {
                editor_state.mode = EditorMode::AwaitingSaveDialog;
                return;
            }
            if editor_state.mode == EditorMode::AwaitingSaveDialog {
                editor_state.discard_requested = true;
                automation_state.exit_applied = true;
            }
        }
        None => {
            automation_state.exit_applied = true;
        }
    }
}

#[derive(Default)]
pub(crate) struct DebugAutomationState {
    spec: Option<Option<DebugAutomationSpec>>,
    operation_applied: bool,
    exit_applied: bool,
}

#[derive(Clone)]
struct DebugAutomationSpec {
    node_id: String,
    operation: DebugAutomationOperation,
    exit_action: Option<DebugExitAction>,
}

#[derive(Clone)]
enum DebugAutomationOperation {
    Move { delta_x: f32, delta_y: f32 },
    Delete,
}

#[derive(Clone, Copy)]
enum DebugExitAction {
    Save,
    Discard,
}

fn read_debug_automation_spec() -> Option<DebugAutomationSpec> {
    read_move_spec().or_else(read_delete_spec)
}

fn read_move_spec() -> Option<DebugAutomationSpec> {
    let raw = std::env::var("BUI_EDITOR_DEBUG_MOVE_NODE").ok()?;
    let mut parts = raw.split(':').map(str::trim);
    let node_id = parts.next()?.to_string();
    let delta_x = parts.next()?.parse::<f32>().ok()?;
    let delta_y = parts.next()?.parse::<f32>().ok()?;
    let exit_action = parts.next().and_then(parse_exit_action);

    if node_id.is_empty() {
        return None;
    }

    Some(DebugAutomationSpec {
        node_id,
        operation: DebugAutomationOperation::Move { delta_x, delta_y },
        exit_action,
    })
}

fn read_delete_spec() -> Option<DebugAutomationSpec> {
    let raw = std::env::var("BUI_EDITOR_DEBUG_DELETE_NODE").ok()?;
    let mut parts = raw.split(':').map(str::trim);
    let node_id = parts.next()?.to_string();
    let exit_action = parts.next().and_then(parse_exit_action);

    if node_id.is_empty() {
        return None;
    }

    Some(DebugAutomationSpec {
        node_id,
        operation: DebugAutomationOperation::Delete,
        exit_action,
    })
}

fn parse_exit_action(value: &str) -> Option<DebugExitAction> {
    match value.to_ascii_lowercase().as_str() {
        "save" => Some(DebugExitAction::Save),
        "discard" => Some(DebugExitAction::Discard),
        _ => None,
    }
}

fn apply_move_spec(
    node_id: &str,
    delta_x: f32,
    delta_y: f32,
    editor_state: &mut BuiEditorState,
    id_map: &BuiIdMap,
    document: &mut BuiDocument,
    target_node_query: &mut Query<&mut Node>,
    target_layout_query: &Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
    child_of_query: &Query<&ChildOf>,
) -> bool {
    let Some(doc_node) = find_bui_node_ref(&document.root, node_id) else {
        return false;
    };

    let position_type = doc_node.layout.styles.position_type.as_deref();
    if position_type != Some("absolute") && position_type != Some("Absolute") {
        return false;
    }

    let old_left = doc_node.layout.styles.left.clone().unwrap_or_default();
    let old_top = doc_node.layout.styles.top.clone().unwrap_or_default();
    let (current_left, current_top) = id_map
        .0
        .get(node_id)
        .and_then(|entity| {
            current_absolute_offsets_px(*entity, target_layout_query, child_of_query)
        })
        .unwrap_or_else(|| {
            (
                parse_px_value(doc_node.layout.styles.left.as_deref()),
                parse_px_value(doc_node.layout.styles.top.as_deref()),
            )
        });

    let new_left_px = current_left + delta_x;
    let new_top_px = current_top + delta_y;
    let new_left = format!("{new_left_px}px");
    let new_top = format!("{new_top_px}px");

    let Some(doc_node) = find_bui_node_mut(&mut document.root, node_id) else {
        return false;
    };
    doc_node.layout.styles.left = Some(new_left.clone());
    doc_node.layout.styles.top = Some(new_top.clone());
    doc_node.layout.styles.right = None;
    doc_node.layout.styles.bottom = None;

    if let Some(entity) = id_map.0.get(node_id) {
        if let Ok(mut node) = target_node_query.get_mut(*entity) {
            node.left = Val::Px(new_left_px);
            node.top = Val::Px(new_top_px);
            node.right = Val::Auto;
            node.bottom = Val::Auto;
        }
    }

    editor_state.pending_edits.push(BuiEdit::PositionChange {
        node_id: node_id.to_string(),
        old_left,
        old_top,
        new_left,
        new_top,
    });
    true
}

fn current_absolute_offsets_px(
    entity: Entity,
    layout_query: &Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
    child_of_query: &Query<&ChildOf>,
) -> Option<(f32, f32)> {
    let child_rect = logical_rect(entity, layout_query)?;
    let parent_min = child_of_query
        .get(entity)
        .ok()
        .and_then(|child_of| logical_rect(child_of.parent(), layout_query))
        .map(|rect| rect.min)
        .unwrap_or_default();

    Some((
        child_rect.min.x - parent_min.x,
        child_rect.min.y - parent_min.y,
    ))
}

fn logical_rect(
    entity: Entity,
    layout_query: &Query<(&UiGlobalTransform, Option<&ComputedNode>)>,
) -> Option<Rect> {
    let (transform, computed) = layout_query.get(entity).ok()?;
    let computed = computed?;
    let size = computed.size();
    if size.x <= 0.0 || size.y <= 0.0 {
        return None;
    }

    let inverse_scale = computed.inverse_scale_factor();
    let logical_center = transform.translation * inverse_scale;
    let logical_size = size * inverse_scale;
    Some(Rect::from_center_size(logical_center, logical_size))
}

fn parse_px_value(value: Option<&str>) -> f32 {
    value
        .and_then(|v| v.strip_suffix("px"))
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0)
}
