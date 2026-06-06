use bevy_ecs::prelude::*;

/// Stable id copied from the BUI node's `id` field.
#[derive(Component, Debug, Clone)]
pub struct BuiId(pub String);

/// Logic tags copied from the BUI node's `custom_tags` field.
#[derive(Component, Debug, Clone)]
pub struct BuiLogicTags(pub Vec<String>);

/// Entity id of the spawned BUI root.
#[derive(Resource, Debug, Clone)]
pub struct BuiRootEntity(pub Entity);
