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

/// Fixed-design stage that should be visually fitted into its parent viewport.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct BuiStageFit {
    pub(crate) design_width: f32,
    pub(crate) design_height: f32,
    pub(crate) mode: BuiStageFitMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiStageFitMode {
    Contain,
    Cover,
    Fill,
    None,
    ScaleDown,
}

/// Entity id of the spawned BUI root.
#[derive(Resource, Debug, Clone)]
pub struct BuiRootEntity(pub Entity);

/// Persisted BuiDocument, kept alive after spawn for editor access.
#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub struct BuiDocumentResource(pub BuiDocument);

/// BuiId string → ECS Entity mapping, built during spawn.
#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BuiIdMap(pub std::collections::HashMap<String, Entity>);

/// Source file paths for write-back on editor save.
#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BuiSourcePaths {
    pub ir_json_path: Option<PathBuf>,
    pub html_path: Option<PathBuf>,
}
