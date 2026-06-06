use bevy_ecs::prelude::*;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::types::{BuiBindingValue, BuiStateSet, BuiStateStore},
    interaction::components::{BuiDisabled, BuiTabGroupDefinition, BuiTabItem, BuiVisualState},
};

pub(crate) fn dispatch_bui_tab_selection_system(
    tab_groups: Query<&BuiTabGroupDefinition>,
    tab_items: Query<(&Interaction, &BuiTabItem, Has<BuiDisabled>), Changed<Interaction>>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (interaction, tab_item, disabled) in &tab_items {
        if disabled || *interaction != Interaction::Pressed {
            continue;
        }

        let Some(group) = tab_groups.iter().find(|group| group.group == tab_item.group) else {
            continue;
        };

        state_writer.write(BuiStateSet {
            key: group.source.clone(),
            value: BuiBindingValue::Text(tab_item.value.clone()),
        });
    }
}

pub(crate) fn sync_bui_tab_selected_state_system(
    tab_groups: Query<&BuiTabGroupDefinition>,
    tab_items: Query<(Entity, &BuiTabItem)>,
    state_store: Res<BuiStateStore>,
    mut commands: Commands,
) {
    if !state_store.is_changed() {
        return;
    }

    for (entity, tab_item) in &tab_items {
        let Some(group) = tab_groups.iter().find(|group| group.group == tab_item.group) else {
            continue;
        };

        let selected = matches!(
            state_store.0.get(&group.source),
            Some(BuiBindingValue::Text(value)) if value == &tab_item.value
        );

        if selected {
            commands
                .entity(entity)
                .insert(BuiVisualState("selected".to_string()));
        } else {
            commands.entity(entity).remove::<BuiVisualState>();
        }
    }
}
