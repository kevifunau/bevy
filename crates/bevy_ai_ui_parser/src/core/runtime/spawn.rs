use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_ui::{prelude::*, Checkable, Checked, FocusPolicy};

use crate::core::{
    interaction::components::{BuiTextInputProxy, BuiToggle},
    model::{BuiDocument, BuiNode, BuiNodeType},
    runtime::{
        image::{background_image_layout, build_image_node},
        node_spawn::{
            build_node, insert_identity_components, insert_style_components,
            insert_visual_components,
        },
        text::{spawn_text_input_mirror, spawn_text_input_node, spawn_text_node},
    },
};

pub(crate) fn spawn_bui_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    document: &BuiDocument,
) -> Result<Entity, String> {
    spawn_bui_node(commands, asset_server, texture_atlases, &document.root)
}

pub(crate) fn spawn_bui_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    node: &BuiNode,
) -> Result<Entity, String> {
    let entity = commands.spawn_empty().id();

    {
        let mut entity_commands = commands.entity(entity);
        insert_identity_components(&mut entity_commands, node);
        insert_visual_components(&mut entity_commands, node)?;
        insert_style_components(&mut entity_commands, node)?;
    }

    let base_node = build_node(&node.styles, &node.visuals)?;

    match node.node_type {
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
            if node.custom_tags.iter().any(|tag| tag == "State_Checked") {
                entity_commands.insert(Checked);
            }
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
                .image_config
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

    if !matches!(node.node_type, BuiNodeType::TextInput) && node.list_binding_source.is_none() {
        let mut first_text_input_child = None;

        for child in &node.children {
            let child_entity = spawn_bui_node(commands, asset_server, texture_atlases, child)?;
            if first_text_input_child.is_none() && matches!(child.node_type, BuiNodeType::TextInput)
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

fn insert_optional_background_image(
    entity_commands: &mut EntityCommands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    node: &BuiNode,
) -> Result<(), String> {
    let Some(image_config) = &node.image_config else {
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
