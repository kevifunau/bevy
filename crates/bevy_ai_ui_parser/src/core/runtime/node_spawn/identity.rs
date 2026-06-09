use bevy_ecs::prelude::*;

use crate::core::{
    interaction::components::{
        BuiActions, BuiBindings, BuiDisabled, BuiListDefinition, BuiProgressFill, BuiProgressGroup,
        BuiTabGroupDefinition, BuiTabItem, BuiVisualStateDefinitions,
    },
    model::BuiNode,
    runtime::components::{BuiId, BuiLogicTags},
};

pub(crate) fn insert_identity_components(entity_commands: &mut EntityCommands, node: &BuiNode) {
    entity_commands.insert((Name::new(node.id.clone()), BuiId(node.id.clone())));

    if !node.markers.is_empty() {
        entity_commands.insert(BuiLogicTags(node.markers.clone()));
    }
    if !node.actions.is_empty() {
        entity_commands.insert(BuiActions(node.actions.clone()));
    }
    if !node.bindings.is_empty() {
        entity_commands.insert(BuiBindings(node.bindings.clone()));
    }
    if !node.state_visuals.is_empty() {
        entity_commands.insert(BuiVisualStateDefinitions {
            states: node.state_visuals.clone(),
        });
    }
    if node.markers.iter().any(|tag| tag == "State_Disabled") {
        entity_commands.insert(BuiDisabled);
    }
    if let (Some(group), Some(source)) = (
        &node.semantics.tab_group_name,
        &node.semantics.tab_binding_source,
    ) {
        entity_commands.insert(BuiTabGroupDefinition {
            group: group.clone(),
            source: source.clone(),
        });
    }
    if let (Some(group), Some(value)) = (&node.semantics.tab_group_name, &node.semantics.tab_value)
    {
        entity_commands.insert(BuiTabItem {
            group: group.clone(),
            value: value.clone(),
        });
    }
    if let Some(source) = &node.semantics.progress_binding_source {
        entity_commands.insert(BuiProgressGroup {
            source: source.clone(),
        });
    }
    if node.semantics.progress_fill {
        entity_commands.insert(BuiProgressFill);
    }
    if let Some(source) = &node.semantics.list_binding_source
        && let Some(template) = node.children.first()
    {
        entity_commands.insert(BuiListDefinition {
            source: source.clone(),
            item_template: template.clone(),
        });
    }
}
