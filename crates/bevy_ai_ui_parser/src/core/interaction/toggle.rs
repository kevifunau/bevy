use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_ui::prelude::*;
use bevy_ui::Checked;

use crate::core::legacy::{BuiToggle, PendingUiTargetCamera};

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
    toggles: Query<(Entity, &Interaction, Has<Checked>), (Changed<Interaction>, With<BuiToggle>)>,
) {
    for (entity, interaction, checked) in &toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if checked {
            commands.entity(entity).remove::<Checked>();
        } else {
            commands.entity(entity).insert(Checked);
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
