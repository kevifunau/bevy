use bevy_ecs::prelude::*;
use bevy_ui::UiTransform;

use crate::core::{
    interaction::components::{
        BuiActions, BuiBaseVisualState, BuiBindings, BuiDisabled, BuiDropdownGroupDefinition,
        BuiDropdownItem, BuiListDefinition, BuiProgressFill, BuiProgressGroup,
        BuiTabGroupDefinition, BuiTabItem, BuiVisualState, BuiVisualStateDefinitions,
    },
    model::{BuiNode, BuiStateVisual},
    runtime::components::{BuiId, BuiLogicTags, BuiStageFit, BuiStageFitMode},
};

pub(crate) fn insert_identity_components(entity_commands: &mut EntityCommands, node: &BuiNode) {
    entity_commands.insert((Name::new(node.id.clone()), BuiId(node.id.clone())));

    if !node.markers.is_empty() {
        entity_commands.insert(BuiLogicTags(node.markers.clone()));
    }
    if let Some(fit) = stage_fit_from_node(node) {
        entity_commands.insert((fit, UiTransform::default()));
    }
    if !node.actions.is_empty() {
        entity_commands.insert(BuiActions(node.actions.clone()));
    }
    if !node.bindings.is_empty() {
        entity_commands.insert(BuiBindings(node.bindings.clone()));
    }
    if !node.state_visuals.is_empty() {
        entity_commands.insert(BuiBaseVisualState(BuiStateVisual {
            styles: node.layout.styles.clone(),
            visuals: node.style.visuals.clone(),
            text_color: node
                .content
                .text
                .as_ref()
                .map(|text| text.font_color.clone()),
            image: node.content.image.clone(),
        }));
        entity_commands.insert(BuiVisualStateDefinitions {
            states: node.state_visuals.clone(),
        });
    }
    if let Some(state) = node
        .markers
        .iter()
        .find_map(|marker| marker.strip_prefix("initial-state:"))
        .filter(|state| !state.trim().is_empty())
    {
        entity_commands.insert(BuiVisualState(state.to_string()));
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
    if let (Some(group), Some(source)) = (
        &node.semantics.dropdown_group_name,
        &node.semantics.dropdown_binding_source,
    ) {
        entity_commands.insert(BuiDropdownGroupDefinition {
            group: group.clone(),
            source: source.clone(),
        });
    }
    if let (Some(group), Some(value)) = (
        &node.semantics.dropdown_group_name,
        &node.semantics.dropdown_value,
    ) {
        entity_commands.insert(BuiDropdownItem {
            group: group.clone(),
            value: value.clone(),
        });
    }
}

pub(crate) fn stage_fit_from_node(node: &BuiNode) -> Option<BuiStageFit> {
    stage_fit_from_semantics(node).or_else(|| stage_fit_from_markers(&node.markers))
}

fn stage_fit_from_semantics(node: &BuiNode) -> Option<BuiStageFit> {
    let fit = node.semantics.stage_fit.as_ref()?;
    if fit.design_width <= 0.0 || fit.design_height <= 0.0 {
        return None;
    }
    Some(BuiStageFit {
        design_width: fit.design_width,
        design_height: fit.design_height,
        mode: stage_fit_mode(&fit.mode)?,
    })
}

fn stage_fit_from_markers(markers: &[String]) -> Option<BuiStageFit> {
    let value = markers
        .iter()
        .find_map(|marker| marker.strip_prefix("bui-stage-fit:"))?;
    let (width, height) = value.split_once('x')?;
    let design_width = width.parse::<f32>().ok()?;
    let design_height = height.parse::<f32>().ok()?;
    if design_width <= 0.0 || design_height <= 0.0 {
        return None;
    }

    Some(BuiStageFit {
        design_width,
        design_height,
        mode: BuiStageFitMode::ScaleDown,
    })
}

fn stage_fit_mode(value: &str) -> Option<BuiStageFitMode> {
    match value {
        "contain" => Some(BuiStageFitMode::Contain),
        "cover" => Some(BuiStageFitMode::Cover),
        "fill" => Some(BuiStageFitMode::Fill),
        "none" => Some(BuiStageFitMode::None),
        "scale-down" | "scale_down" => Some(BuiStageFitMode::ScaleDown),
        _ => None,
    }
}
