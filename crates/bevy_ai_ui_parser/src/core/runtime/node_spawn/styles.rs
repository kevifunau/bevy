use bevy_ecs::prelude::*;
use bevy_input_focus::{tab_navigation::TabIndex, AutoFocus};
use bevy_ui::{
    prelude::*,
    RelativeCursorPosition,
};

use crate::core::{
    interaction::components::PendingUiTargetCamera,
    model::{BuiNode, BuiStyles, BuiVisuals},
    style::css_parser::{
        parse_align_content, parse_align_items, parse_align_self, parse_border_radius,
        parse_display, parse_flex_direction, parse_flex_wrap, parse_grid_placement,
        parse_grid_tracks, parse_integer, parse_justify_content, parse_justify_items,
        parse_justify_self, parse_number, parse_overflow, parse_overflow_clip_margin,
        parse_position_type, parse_tab_group, parse_ui_rect, parse_val, parse_visibility,
    },
};

use super::helpers::{has_ui_transform_styles, insert_ui_transform, set_val};

pub(crate) fn insert_style_components(
    entity_commands: &mut EntityCommands,
    node: &BuiNode,
) -> Result<(), String> {
    if let Some(value) = &node.styles.visibility {
        entity_commands.insert(parse_visibility(value)?);
    }
    insert_ui_transform(entity_commands, &node.styles)?;
    if !has_ui_transform_styles(&node.styles)
        && node
            .state_visuals
            .values()
            .any(|state| has_ui_transform_styles(&state.styles))
    {
        entity_commands.insert(UiTransform::default());
    }
    if node.styles.relative_cursor_position.unwrap_or(false) {
        entity_commands.insert(RelativeCursorPosition::default());
    }
    if let Some(target_name) = &node.styles.ui_target_camera {
        entity_commands.insert(PendingUiTargetCamera {
            target_name: target_name.clone(),
        });
    }
    if let Some(value) = &node.styles.tab_group {
        entity_commands.insert(parse_tab_group(value)?);
    }
    if let Some(value) = &node.styles.tab_index {
        entity_commands.insert(TabIndex(parse_integer(value)?));
    }
    if node.styles.auto_focus.unwrap_or(false) {
        entity_commands.insert(AutoFocus);
    }
    if node.styles.fixed_node.unwrap_or(false) {
        entity_commands.insert(FixedNode);
    }
    if let Some(value) = &node.styles.z_index {
        entity_commands.insert(ZIndex(parse_integer(value)?));
    }
    if let Some(value) = &node.styles.global_z_index {
        entity_commands.insert(GlobalZIndex(parse_integer(value)?));
    }

    Ok(())
}

pub(crate) fn build_node(styles: &BuiStyles, visuals: &BuiVisuals) -> Result<Node, String> {
    let mut node = Node::default();

    if let Some(value) = &styles.display {
        node.display = parse_display(value)?;
    }

    set_val(&mut node.width, &styles.width)?;
    set_val(&mut node.height, &styles.height)?;
    if let Some(value) = &styles.aspect_ratio {
        node.aspect_ratio = Some(parse_number(value)?);
    }
    set_val(&mut node.min_width, &styles.min_width)?;
    set_val(&mut node.min_height, &styles.min_height)?;
    set_val(&mut node.max_width, &styles.max_width)?;
    set_val(&mut node.max_height, &styles.max_height)?;
    set_val(&mut node.left, &styles.left)?;
    set_val(&mut node.right, &styles.right)?;
    set_val(&mut node.top, &styles.top)?;
    set_val(&mut node.bottom, &styles.bottom)?;
    if let Some(value) = &styles.overflow {
        node.overflow = parse_overflow(value)?;
    }
    if let Some(value) = &styles.overflow_clip_margin {
        node.overflow_clip_margin = parse_overflow_clip_margin(value)?;
    }

    if let Some(margin) = &styles.margin {
        node.margin = parse_ui_rect(margin)?;
    }
    set_val(&mut node.margin.left, &styles.margin_left)?;
    set_val(&mut node.margin.right, &styles.margin_right)?;
    set_val(&mut node.margin.top, &styles.margin_top)?;
    set_val(&mut node.margin.bottom, &styles.margin_bottom)?;

    if let Some(padding) = &styles.padding {
        node.padding = parse_ui_rect(padding)?;
    }
    set_val(&mut node.padding.left, &styles.padding_left)?;
    set_val(&mut node.padding.right, &styles.padding_right)?;
    set_val(&mut node.padding.top, &styles.padding_top)?;
    set_val(&mut node.padding.bottom, &styles.padding_bottom)?;

    if let Some(border_width) = &visuals.border_width {
        node.border = parse_ui_rect(border_width)?;
    }
    if let Some(border_radius) = &visuals.border_radius {
        node.border_radius = parse_border_radius(border_radius)?;
    }

    if let Some(value) = &styles.flex_direction {
        node.flex_direction = parse_flex_direction(value)?;
    }
    if let Some(value) = &styles.flex_wrap {
        node.flex_wrap = parse_flex_wrap(value)?;
    }
    if let Some(value) = &styles.flex_grow {
        node.flex_grow = parse_number(value)?;
    }
    if let Some(value) = &styles.flex_shrink {
        node.flex_shrink = parse_number(value)?;
    }
    if let Some(value) = &styles.flex_basis {
        node.flex_basis = parse_val(value)?;
    }
    set_val(&mut node.row_gap, &styles.row_gap)?;
    set_val(&mut node.column_gap, &styles.column_gap)?;
    if let Some(value) = &styles.justify_content {
        node.justify_content = parse_justify_content(value)?;
    }
    if let Some(value) = &styles.justify_items {
        node.justify_items = parse_justify_items(value)?;
    }
    if let Some(value) = &styles.align_content {
        node.align_content = parse_align_content(value)?;
    }
    if let Some(value) = &styles.align_items {
        node.align_items = parse_align_items(value)?;
    }
    if let Some(value) = &styles.align_self {
        node.align_self = parse_align_self(value)?;
    }
    if let Some(value) = &styles.justify_self {
        node.justify_self = parse_justify_self(value)?;
    }
    if let Some(value) = &styles.position_type {
        node.position_type = parse_position_type(value)?;
    }
    if let Some(value) = &styles.grid_template_columns {
        node.grid_template_columns = parse_grid_tracks(value)?;
    }
    if let Some(value) = &styles.grid_template_rows {
        node.grid_template_rows = parse_grid_tracks(value)?;
    }
    if let Some(value) = &styles.grid_column {
        node.grid_column = parse_grid_placement(value)?;
    }
    if let Some(value) = &styles.grid_row {
        node.grid_row = parse_grid_placement(value)?;
    }

    Ok(node)
}
