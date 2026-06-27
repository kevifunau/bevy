use bevy_ecs::prelude::*;

use crate::core::interaction::types::{BuiBindingUpdate, BuiStateStore};

pub(crate) fn emit_initial_bui_binding_updates_system(
    state_store: Res<BuiStateStore>,
    mut binding_writer: MessageWriter<BuiBindingUpdate>,
    mut seeded: Local<bool>,
) {
    if *seeded {
        return;
    }

    *seeded = true;

    if state_store.0.is_empty() {
        return;
    }

    for (key, value) in &state_store.0 {
        binding_writer.write(BuiBindingUpdate {
            source: key.clone(),
            value: value.clone(),
        });
    }
}
