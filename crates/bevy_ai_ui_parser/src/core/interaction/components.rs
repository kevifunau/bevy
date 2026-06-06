use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::core::model::{BuiActionBinding, BuiBinding, BuiNode, BuiStateVisual};

/// Declarative action bindings copied from the BUI node's `actions` field.
#[derive(Component, Debug, Clone)]
pub struct BuiActions(pub Vec<BuiActionBinding>);

/// Declarative data bindings copied from the BUI node's `bindings` field.
#[derive(Component, Debug, Clone)]
pub struct BuiBindings(pub Vec<BuiBinding>);

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiTabGroupDefinition {
    pub(crate) group: String,
    pub(crate) source: String,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiTabItem {
    pub(crate) group: String,
    pub(crate) value: String,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiProgressGroup {
    pub(crate) source: String,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct BuiProgressFill;

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiListDefinition {
    pub(crate) source: String,
    pub(crate) item_template: BuiNode,
}

/// Marker for JSON nodes parsed as toggle widgets.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuiToggle;

/// Marker for JSON nodes parsed as text input widgets.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuiTextInput;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct BuiTextInputProxy {
    pub(crate) target: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct BuiTextInputMirror {
    pub(crate) target: Entity,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct PendingUiTargetCamera {
    pub(crate) target_name: String,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiVisualStateDefinitions {
    pub(crate) states: HashMap<String, BuiStateVisual>,
}

/// Explicit visual state name applied to a BUI node at runtime.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct BuiVisualState(pub String);

/// Marks a BUI node as disabled for declarative state resolution and action dispatch.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuiDisabled;