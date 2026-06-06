use bevy_ecs::prelude::*;

use crate::core::interaction::types::{BuiBindingUpdate, BuiStateSet, BuiStateStore};

pub(crate) fn apply_bui_state_updates_system(
    mut updates: MessageReader<BuiStateSet>,
    mut state_store: ResMut<BuiStateStore>,
    mut binding_writer: MessageWriter<BuiBindingUpdate>,
) {
    for update in updates.read() {
        let key = update.key.clone();
        let value = update.value.clone();

        let changed = state_store.0.get(&key) != Some(&value);
        if !changed {
            continue;
        }

        state_store.0.insert(key.clone(), value.clone());
        binding_writer.write(BuiBindingUpdate { source: key, value });
    }
}
