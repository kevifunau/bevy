use bevy_ecs::prelude::*;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::types::{BuiBindingValue, BuiStateStore},
    legacy::{BuiProgressFill, BuiProgressGroup},
};

pub(crate) fn sync_bui_progress_groups_system(
    state_store: Res<BuiStateStore>,
    groups: Query<(&BuiProgressGroup, &Children)>,
    fills: Query<(), With<BuiProgressFill>>,
    mut nodes: Query<&mut Node>,
) {
    if !state_store.is_changed() {
        return;
    }

    for (group, children) in &groups {
        let Some(BuiBindingValue::Number(value)) = state_store.0.get(&group.source) else {
            continue;
        };

        let ratio = value.clamp(0.0, 1.0) * 100.0;

        for child in children.iter() {
            if fills.get(child).is_err() {
                continue;
            }

            let Ok(mut node) = nodes.get_mut(child) else {
                continue;
            };

            node.width = Val::Percent(ratio);
        }
    }
}
