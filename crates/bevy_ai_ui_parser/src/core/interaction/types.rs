use std::collections::HashMap;

use bevy_ecs::prelude::*;

/// Runtime message emitted when a BUI action is triggered by interaction.
#[allow(dead_code)]
#[derive(Message, Debug, Clone)]
pub struct BuiActionTriggered {
    /// The entity whose interaction triggered the action.
    pub entity: Entity,
    /// Stable BUI id copied from the source node.
    pub id: String,
    /// The emitted action name declared in JSON.
    pub action: String,
    /// The interaction trigger that fired this action.
    pub trigger: BuiActionTrigger,
}

/// Runtime message emitted by game systems to update declarative BUI bindings.
#[derive(Message, Debug, Clone)]
pub struct BuiBindingUpdate {
    /// The binding source key, for example `hero.power_value`.
    pub source: String,
    /// The value to push into matching bindings.
    pub value: BuiBindingValue,
}

/// Supported runtime values for declarative BUI bindings.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BuiBindingValue {
    /// String-like content, currently applied to `text.content`.
    Text(String),
    /// String list content, currently applied to semantic list nodes.
    StringList(Vec<String>),
    /// Object list content, currently applied to semantic list nodes with keyed placeholders.
    ObjectList(Vec<HashMap<String, String>>),
    /// Numeric-like content, currently applied to size-like targets.
    Number(f32),
    /// Boolean-like content, currently applied to `visibility`.
    Bool(bool),
    /// Color-like content, currently applied to visual or text color targets.
    Color(String),
}

/// Scene-level runtime state store used to resolve declarative BUI bindings.
#[derive(Resource, Debug, Default)]
pub struct BuiStateStore(pub HashMap<String, BuiBindingValue>);

/// Runtime message emitted by game systems to update the scene-level BUI state store.
#[derive(Message, Debug, Clone)]
pub struct BuiStateSet {
    /// The state key to update, for example `hero.power_value`.
    pub key: String,
    /// The new value to store under `key`.
    pub value: BuiBindingValue,
}

/// Supported declarative interaction triggers for BUI actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiActionTrigger {
    /// Trigger when the node is pressed.
    Press,
    /// Trigger when the node enters the hovered state.
    HoverEnter,
    /// Trigger when the node exits the hovered state.
    HoverExit,
}
