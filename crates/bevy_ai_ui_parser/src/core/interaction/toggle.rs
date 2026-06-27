use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_ui::prelude::*;
use bevy_ui::Checked;

use crate::core::{
    interaction::{
        components::{BuiActions, BuiBindings, BuiToggle, PendingUiTargetCamera},
        types::{BuiActionTrigger, BuiActionTriggered, BuiBindingValue, BuiStateSet},
    },
    runtime::components::BuiId,
};

fn set_toggle_box_color(
    children: &Children,
    checked: bool,
    backgrounds: &mut Query<&mut BackgroundColor>,
) {
    let Some(first_child) = children.first() else {
        return;
    };

    let Ok(mut color) = backgrounds.get_mut(*first_child) else {
        return;
    };

    color.0 = if checked {
        Color::srgb(0.35, 0.75, 0.35)
    } else {
        Color::srgb(0.2, 0.2, 0.2)
    };
}

pub(crate) fn toggle_interaction_system(
    mut commands: Commands,
    toggles: Query<
        (
            Entity,
            &Interaction,
            &BuiId,
            Has<Checked>,
            Option<&BuiActions>,
            Option<&BuiBindings>,
        ),
        (Changed<Interaction>, With<BuiToggle>),
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (entity, interaction, id, checked, actions, bindings) in &toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let next_checked = !checked;
        if checked {
            commands.entity(entity).remove::<Checked>();
        } else {
            commands.entity(entity).insert(Checked);
        }

        state_writer.write(BuiStateSet {
            key: format!("{}.checked", id.0),
            value: BuiBindingValue::Bool(next_checked),
        });
        if let Some(bindings) = bindings {
            for binding in &bindings.0 {
                if binding.target == "checked" {
                    state_writer.write(BuiStateSet {
                        key: binding.source.clone(),
                        value: BuiBindingValue::Bool(next_checked),
                    });
                }
            }
        }

        let Some(actions) = actions else {
            continue;
        };
        for action in &actions.0 {
            if !matches!(
                action.event.as_str(),
                "value_changed" | "change" | "changed"
            ) {
                continue;
            }
            action_writer.write(BuiActionTriggered {
                entity,
                id: id.0.clone(),
                action: action.emit.clone(),
                trigger: BuiActionTrigger::ValueChanged,
            });
        }
    }
}

pub(crate) fn update_toggle_visual_system(
    toggles: Query<(&Children, Has<Checked>), With<BuiToggle>>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    for (children, checked) in &toggles {
        set_toggle_box_color(children, checked, &mut backgrounds);
    }
}

pub(crate) fn resolve_ui_target_camera_system(
    mut commands: Commands,
    pending_nodes: Query<(Entity, &PendingUiTargetCamera)>,
    named_entities: Query<(Entity, &Name)>,
) {
    for (entity, pending) in &pending_nodes {
        let Some((camera_entity, _)) = named_entities
            .iter()
            .find(|(_, name)| name.as_str() == pending.target_name)
        else {
            continue;
        };

        commands
            .entity(entity)
            .insert(UiTargetCamera(camera_entity))
            .remove::<PendingUiTargetCamera>();
    }
}
