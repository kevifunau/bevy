use std::collections::HashMap;

use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_ui::{prelude::*, Checkable, Checked, FocusPolicy, OverflowAxis, ScrollPosition};
use bevy_ui_widgets::{
    Slider, SliderOrientation, SliderRange, SliderStep, SliderValue, TrackClick,
};

use crate::core::{
    interaction::components::{BuiFocusOrder, BuiScrollView, BuiTextInputProxy, BuiToggle},
    model::{BuiDocument, BuiNode, BuiNodeType},
    runtime::{
        image::{background_image_layout, build_image_node},
        node_spawn::{
            build_node, insert_identity_components, insert_style_components,
            insert_visual_components, stage_fit_from_node,
        },
        text::{spawn_text_input_mirror, spawn_text_input_node, spawn_text_node},
    },
};

pub fn spawn_bui_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    document: &BuiDocument,
) -> Result<(Entity, HashMap<String, Entity>), String> {
    let mut id_map = HashMap::new();
    let root = spawn_bui_node_inner(
        commands,
        asset_server,
        texture_atlases,
        &document.root,
        &mut id_map,
    )?;
    Ok((root, id_map))
}

fn spawn_bui_node_inner(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    node: &BuiNode,
    id_map: &mut HashMap<String, Entity>,
) -> Result<Entity, String> {
    let entity = commands.spawn_empty().id();
    let focus_order = id_map.len() as u32;
    id_map.insert(node.id.clone(), entity);
    commands.entity(entity).insert(BuiFocusOrder(focus_order));

    {
        let mut entity_commands = commands.entity(entity);
        insert_identity_components(&mut entity_commands, node);
        insert_visual_components(&mut entity_commands, node)?;
        insert_style_components(&mut entity_commands, node)?;
    }

    let mut base_node = build_node(&node.layout.styles, &node.style.visuals)?;
    if let Some(fit) = stage_fit_from_node(node) {
        base_node.flex_shrink = 0.0;
        base_node.width = Val::Px(fit.design_width);
        base_node.height = Val::Px(fit.design_height);
    }
    let is_scrollable = node_is_scrollable(node, &base_node);

    match node.node_type() {
        BuiNodeType::Node => {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert((base_node, FocusPolicy::Pass));
            insert_optional_background_image(
                &mut entity_commands,
                asset_server,
                texture_atlases,
                node,
            )?;
        }
        BuiNodeType::Button => {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert((Button, base_node));
            insert_optional_background_image(
                &mut entity_commands,
                asset_server,
                texture_atlases,
                node,
            )?;
        }
        BuiNodeType::Toggle => {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert((Button, Checkable, BuiToggle, base_node));
            if node.markers.iter().any(|tag| tag == "State_Checked") {
                entity_commands.insert(Checked);
            }
        }
        BuiNodeType::Slider => {
            let mut entity_commands = commands.entity(entity);
            let (value, range, step, orientation) = slider_components(node);
            entity_commands.insert((
                Slider {
                    track_click: TrackClick::Drag,
                    orientation,
                },
                value,
                range,
                step,
                base_node,
                FocusPolicy::Block,
            ));
            insert_optional_background_image(
                &mut entity_commands,
                asset_server,
                texture_atlases,
                node,
            )?;
        }
        BuiNodeType::Text => {
            let mut entity_commands = commands.entity(entity);
            spawn_text_node(&mut entity_commands, asset_server, node, base_node)?;
        }
        BuiNodeType::TextInput => {
            let mirror_spec = {
                let mut entity_commands = commands.entity(entity);
                spawn_text_input_node(&mut entity_commands, asset_server, node, base_node)?
            };
            spawn_text_input_mirror(commands, entity, mirror_spec)?;
        }
        BuiNodeType::Image => {
            let image_config = node
                .content
                .image
                .as_ref()
                .ok_or_else(|| format!("Image node '{}' is missing image_config.", node.id))?;
            let image_node = build_image_node(asset_server, texture_atlases, image_config)?;
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert((base_node, image_node, FocusPolicy::Pass));
            if let Some(layout) = background_image_layout(image_config) {
                entity_commands.insert(layout);
            }
        }
    }

    if is_scrollable {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(ScrollPosition::default());
        if let Some(scroll_view) = &node.semantics.scroll_view {
            entity_commands.insert(BuiScrollView {
                binding_source: scroll_view.binding_source.clone(),
            });
        }
    }

    if !matches!(node.node_type(), BuiNodeType::TextInput)
        && node.semantics.list_binding_source.is_none()
    {
        let mut first_text_input_child = None;

        for child in &node.children {
            let child_entity =
                spawn_bui_node_inner(commands, asset_server, texture_atlases, child, id_map)?;
            if first_text_input_child.is_none()
                && matches!(child.node_type(), BuiNodeType::TextInput)
            {
                first_text_input_child = Some(child_entity);
            }
            commands.entity(entity).add_child(child_entity);
        }

        if let Some(target) = first_text_input_child {
            commands.entity(entity).insert((
                Interaction::default(),
                FocusPolicy::Block,
                BuiTextInputProxy { target },
            ));
        }
    }

    Ok(entity)
}

fn slider_components(node: &BuiNode) -> (SliderValue, SliderRange, SliderStep, SliderOrientation) {
    let Some(slider) = &node.semantics.slider else {
        return (
            SliderValue::default(),
            SliderRange::default(),
            SliderStep::default(),
            SliderOrientation::Auto,
        );
    };

    let orientation = match slider.orientation.as_deref() {
        Some("horizontal") => SliderOrientation::Horizontal,
        Some("vertical") => SliderOrientation::Vertical,
        _ => SliderOrientation::Auto,
    };

    (
        SliderValue(slider.value),
        SliderRange::new(slider.min, slider.max),
        SliderStep(slider.step.unwrap_or(1.0)),
        orientation,
    )
}

fn node_is_scrollable(node: &BuiNode, base_node: &Node) -> bool {
    node.semantics.scroll_view.is_some()
        || base_node.overflow.x == OverflowAxis::Scroll
        || base_node.overflow.y == OverflowAxis::Scroll
}

/// Public spawn function used by list.rs (no id_map collection).
pub(crate) fn spawn_bui_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    node: &BuiNode,
) -> Result<Entity, String> {
    spawn_bui_node_inner(
        commands,
        asset_server,
        texture_atlases,
        node,
        &mut HashMap::new(),
    )
}

fn insert_optional_background_image(
    entity_commands: &mut EntityCommands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    node: &BuiNode,
) -> Result<(), String> {
    let Some(image_config) = &node.content.image else {
        return Ok(());
    };

    entity_commands.insert(build_image_node(
        asset_server,
        texture_atlases,
        image_config,
    )?);
    if let Some(layout) = background_image_layout(image_config) {
        entity_commands.insert(layout);
    }
    Ok(())
}

pub(crate) use crate::core::runtime::image::sync_background_image_layout_system;
