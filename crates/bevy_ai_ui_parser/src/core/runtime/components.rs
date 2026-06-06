use bevy_ecs::prelude::*;

use crate::core::model::{BuiActionBinding, BuiBinding};

/// Stable id copied from the BUI node's `id` field.
#[derive(Component, Debug, Clone)]
pub struct BuiId(pub String);

/// Logic tags copied from the BUI node's `custom_tags` field.
#[derive(Component, Debug, Clone)]
pub struct BuiLogicTags(pub Vec<String>);

/// Declarative action bindings copied from the BUI node's `actions` field.
#[derive(Component, Debug, Clone)]
pub struct BuiActions(pub Vec<BuiActionBinding>);

/// Declarative data bindings copied from the BUI node's `bindings` field.
#[derive(Component, Debug, Clone)]
pub struct BuiBindings(pub Vec<BuiBinding>);

/// Entity id of the spawned BUI root.
#[derive(Resource, Debug, Clone)]
pub struct BuiRootEntity(pub Entity);
