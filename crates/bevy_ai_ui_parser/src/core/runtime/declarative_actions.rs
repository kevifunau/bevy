use std::collections::VecDeque;

use bevy_asset::AssetServer;
use bevy_camera::visibility::Visibility;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::components::BuiVisualState,
    interaction::types::{BuiActionTriggered, BuiBindingValue, BuiStateSet},
    model::{BuiInteractionModel, BuiInteractionStep},
    runtime::components::{BuiDocumentResource, BuiId},
};

#[derive(Resource, Debug, Default)]
pub(crate) struct BuiDelayedActionQueue {
    pending: Vec<BuiDelayedAction>,
}

#[derive(Debug, Clone)]
struct BuiDelayedAction {
    remaining_seconds: f32,
    steps: VecDeque<BuiInteractionStep>,
}

pub(crate) fn apply_declarative_action_system(
    mut actions: MessageReader<BuiActionTriggered>,
    document: Option<Res<BuiDocumentResource>>,
    asset_server: Res<AssetServer>,
    mut delayed_queue: ResMut<BuiDelayedActionQueue>,
    mut state_writer: MessageWriter<BuiStateSet>,
    mut texts: Query<(&BuiId, &mut Text)>,
    mut images: Query<(&BuiId, &mut ImageNode)>,
    mut visibilities: Query<(&BuiId, &mut Visibility)>,
    visual_states: Query<(Entity, &BuiId, Option<&BuiVisualState>)>,
    mut commands: Commands,
) {
    let Some(document) = document else {
        return;
    };
    let model = &document.0.interaction_model;
    if model.is_empty() {
        return;
    }

    for action in actions.read() {
        let Some(steps) = model.actions.get(&action.action) else {
            continue;
        };
        apply_steps(
            steps.iter().cloned().collect(),
            model,
            &asset_server,
            &mut delayed_queue,
            &mut state_writer,
            &mut texts,
            &mut images,
            &mut visibilities,
            &visual_states,
            &mut commands,
        );
    }
}

pub(crate) fn advance_delayed_declarative_actions_system(
    time: Res<Time>,
    document: Option<Res<BuiDocumentResource>>,
    asset_server: Res<AssetServer>,
    mut delayed_queue: ResMut<BuiDelayedActionQueue>,
    mut state_writer: MessageWriter<BuiStateSet>,
    mut texts: Query<(&BuiId, &mut Text)>,
    mut images: Query<(&BuiId, &mut ImageNode)>,
    mut visibilities: Query<(&BuiId, &mut Visibility)>,
    visual_states: Query<(Entity, &BuiId, Option<&BuiVisualState>)>,
    mut commands: Commands,
) {
    let Some(document) = document else {
        return;
    };
    if delayed_queue.pending.is_empty() {
        return;
    }

    let delta = time.delta_secs();
    let mut ready = Vec::new();
    let mut waiting = Vec::new();
    for mut action in delayed_queue.pending.drain(..) {
        action.remaining_seconds -= delta;
        if action.remaining_seconds <= 0.0 {
            ready.push(action.steps);
        } else {
            waiting.push(action);
        }
    }
    delayed_queue.pending = waiting;

    for steps in ready {
        apply_steps(
            steps,
            &document.0.interaction_model,
            &asset_server,
            &mut delayed_queue,
            &mut state_writer,
            &mut texts,
            &mut images,
            &mut visibilities,
            &visual_states,
            &mut commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_steps(
    mut steps: VecDeque<BuiInteractionStep>,
    model: &BuiInteractionModel,
    asset_server: &AssetServer,
    delayed_queue: &mut BuiDelayedActionQueue,
    state_writer: &mut MessageWriter<BuiStateSet>,
    texts: &mut Query<(&BuiId, &mut Text)>,
    images: &mut Query<(&BuiId, &mut ImageNode)>,
    visibilities: &mut Query<(&BuiId, &mut Visibility)>,
    visual_states: &Query<(Entity, &BuiId, Option<&BuiVisualState>)>,
    commands: &mut Commands,
) {
    while let Some(step) = steps.pop_front() {
        if let Some(delay_seconds) = delay_seconds(&step) {
            delayed_queue.pending.push(BuiDelayedAction {
                remaining_seconds: delay_seconds,
                steps,
            });
            return;
        }
        apply_step(
            &step,
            model,
            asset_server,
            delayed_queue,
            state_writer,
            texts,
            images,
            visibilities,
            visual_states,
            commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_step(
    step: &BuiInteractionStep,
    model: &BuiInteractionModel,
    asset_server: &AssetServer,
    delayed_queue: &mut BuiDelayedActionQueue,
    state_writer: &mut MessageWriter<BuiStateSet>,
    texts: &mut Query<(&BuiId, &mut Text)>,
    images: &mut Query<(&BuiId, &mut ImageNode)>,
    visibilities: &mut Query<(&BuiId, &mut Visibility)>,
    visual_states: &Query<(Entity, &BuiId, Option<&BuiVisualState>)>,
    commands: &mut Commands,
) {
    match step.op.as_str() {
        "set-text" | "set_text" => {
            let Some(value) = step.value.as_deref() else {
                return;
            };
            if let Some(binding) = step.binding.as_deref() {
                state_writer.write(BuiStateSet {
                    key: binding.to_string(),
                    value: BuiBindingValue::Text(value.to_string()),
                });
            }
            if let Some(node) = step.node.as_deref() {
                set_text_by_id(texts, node, value);
            }
        }
        "set-binding" | "set_binding" | "set-state" | "set_state" => {
            let Some(key) = step
                .binding
                .as_deref()
                .or(step.source.as_deref())
                .or(step.target.as_deref())
            else {
                return;
            };
            let Some(value) = step.value.as_deref() else {
                return;
            };
            state_writer.write(BuiStateSet {
                key: key.to_string(),
                value: parse_binding_value(step.value_type.as_deref(), value),
            });
        }
        "run-action" | "run_action" => {
            let Some(action) = step.target.as_deref().or(step.value.as_deref()) else {
                return;
            };
            let Some(steps) = model.actions.get(action) else {
                return;
            };
            apply_steps(
                steps.iter().cloned().collect(),
                model,
                asset_server,
                delayed_queue,
                state_writer,
                texts,
                images,
                visibilities,
                visual_states,
                commands,
            );
        }
        "set-image" | "set_image" => {
            let (Some(node), Some(image)) = (step.node.as_deref(), step.image.as_deref()) else {
                return;
            };
            set_image_by_id(asset_server, images, node, image);
        }
        "set-selected-image" | "set_selected_image" => {
            let (Some(group), Some(target), Some(idle), Some(selected)) = (
                step.group.as_deref(),
                step.target.as_deref(),
                step.idle_image.as_deref(),
                step.selected_image.as_deref(),
            ) else {
                return;
            };
            for node in group {
                let image = if node == target { selected } else { idle };
                set_image_by_id(asset_server, images, node, image);
            }
        }
        "set-selected" | "set_selected" => {
            let (Some(group), Some(target)) = (step.group.as_deref(), step.target.as_deref())
            else {
                return;
            };
            let state = step.state.as_deref().unwrap_or("selected");
            set_selected_visual_state(visual_states, commands, group, target, state);
        }
        "set-visual-state" | "set_visual_state" => {
            let Some(node) = step.node.as_deref().or(step.target.as_deref()) else {
                return;
            };
            let Some(state) = step.state.as_deref().or(step.value.as_deref()) else {
                return;
            };
            set_visual_state(visual_states, commands, node, Some(state));
        }
        "clear-visual-state" | "clear_visual_state" => {
            let Some(node) = step.node.as_deref().or(step.target.as_deref()) else {
                return;
            };
            set_visual_state(visual_states, commands, node, None);
        }
        "set-visible" | "set_visible" => {
            let Some(node) = step.node.as_deref().or(step.target.as_deref()) else {
                return;
            };
            let Some(value) = step.value.as_deref() else {
                return;
            };
            set_visibility_by_id(visibilities, node, parse_bool_like(value));
        }
        _ => {}
    }
}

fn delay_seconds(step: &BuiInteractionStep) -> Option<f32> {
    if !matches!(step.op.as_str(), "delay" | "wait") {
        return None;
    }
    step.seconds.filter(|seconds| *seconds > 0.0).or_else(|| {
        step.ms
            .map(|ms| ms as f32 / 1000.0)
            .filter(|seconds| *seconds > 0.0)
    })
}

fn parse_binding_value(value_type: Option<&str>, value: &str) -> BuiBindingValue {
    match value_type.unwrap_or("text") {
        "bool" | "boolean" => BuiBindingValue::Bool(parse_bool_like(value)),
        "number" | "float" => value
            .parse::<f32>()
            .map(BuiBindingValue::Number)
            .unwrap_or_else(|_| BuiBindingValue::Text(value.to_string())),
        "color" => BuiBindingValue::Color(value.to_string()),
        _ => BuiBindingValue::Text(value.to_string()),
    }
}

fn parse_bool_like(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "visible" | "show" | "shown"
    )
}

fn set_text_by_id(texts: &mut Query<(&BuiId, &mut Text)>, target_id: &str, value: &str) {
    for (id, mut text) in texts.iter_mut() {
        if id.0 == target_id {
            text.0 = value.to_string();
        }
    }
}

fn set_image_by_id(
    asset_server: &AssetServer,
    images: &mut Query<(&BuiId, &mut ImageNode)>,
    target_id: &str,
    image_path: &str,
) {
    for (id, mut image) in images.iter_mut() {
        if id.0 == target_id {
            image.image = asset_server.load(image_path.to_string());
        }
    }
}

fn set_selected_visual_state(
    visual_states: &Query<(Entity, &BuiId, Option<&BuiVisualState>)>,
    commands: &mut Commands,
    group: &[String],
    target_id: &str,
    state: &str,
) {
    for node in group {
        set_visual_state(
            visual_states,
            commands,
            node,
            (node == target_id).then_some(state),
        );
    }
}

fn set_visual_state(
    visual_states: &Query<(Entity, &BuiId, Option<&BuiVisualState>)>,
    commands: &mut Commands,
    target_id: &str,
    state: Option<&str>,
) {
    for (entity, id, current_state) in visual_states.iter() {
        if id.0 != target_id {
            continue;
        }
        match state {
            Some(state) if current_state.is_none_or(|current| current.0 != state) => {
                commands
                    .entity(entity)
                    .insert(BuiVisualState(state.to_string()));
            }
            None if current_state.is_some() => {
                commands.entity(entity).remove::<BuiVisualState>();
            }
            _ => {}
        }
    }
}

fn set_visibility_by_id(
    visibilities: &mut Query<(&BuiId, &mut Visibility)>,
    target_id: &str,
    visible: bool,
) {
    for (id, mut visibility) in visibilities.iter_mut() {
        if id.0 == target_id {
            *visibility = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
