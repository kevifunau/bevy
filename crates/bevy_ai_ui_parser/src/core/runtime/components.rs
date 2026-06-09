use std::collections::HashMap;
use std::path::PathBuf;

use bevy_ecs::prelude::*;

use crate::core::model::BuiDocument;

/// Stable id copied from the BUI node's `id` field.
#[derive(Component, Debug, Clone)]
pub struct BuiId(pub String);

/// Logic tags copied from the BUI node's `custom_tags` field.
#[derive(Component, Debug, Clone)]
pub struct BuiLogicTags(pub Vec<String>);

/// Entity id of the spawned BUI root.
#[derive(Resource, Debug, Clone)]
pub struct BuiRootEntity(pub Entity);

/// Persisted BuiDocument, kept alive after spawn for editor access.
#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct BuiDocumentResource(pub(crate) BuiDocument);

/// BuiId string → ECS Entity mapping, built during spawn.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct BuiIdMap(pub(crate) HashMap<String, Entity>);

/// Source file paths for write-back on editor save.
#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct BuiSourcePaths {
    pub(crate) ir_json_path: Option<PathBuf>,
    pub(crate) html_path: Option<PathBuf>,
}
