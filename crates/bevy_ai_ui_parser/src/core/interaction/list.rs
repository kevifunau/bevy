use std::collections::HashMap;

use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;

use crate::core::{
    interaction::types::{BuiBindingValue, BuiStateStore},
    legacy::BuiListDefinition,
    model::{BuiNode, BuiTextConfig},
    runtime::spawn::spawn_bui_node,
};

fn replace_template_tokens(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (key, value) in values {
        let token = format!("{{{{{key}}}}}");
        result = result.replace(&token, value);
    }

    result
}

fn instantiate_list_item_template_text(template: &BuiNode, index: usize, item: &str) -> BuiNode {
    let mut node = template.clone();
    node.id = format!("{}__item_{index}", node.id);

    if let Some(text_config) = &mut node.text_config {
        text_config.content = text_config.content.replace("{{item}}", item);
    }

    node.children = node
        .children
        .iter()
        .map(|child| instantiate_list_item_template_text(child, index, item))
        .collect();

    node
}

fn instantiate_list_item_template_object(
    template: &BuiNode,
    index: usize,
    item: &HashMap<String, String>,
) -> BuiNode {
    let mut node = template.clone();
    node.id = format!("{}__item_{index}", node.id);

    if let Some(BuiTextConfig { content, .. }) = &mut node.text_config {
        *content = replace_template_tokens(content, item);
    }

    node.children = node
        .children
        .iter()
        .map(|child| instantiate_list_item_template_object(child, index, item))
        .collect();

    node
}

pub(crate) fn sync_bui_list_groups_system(
    state_store: Res<BuiStateStore>,
    list_groups: Query<(Entity, &BuiListDefinition, Option<&Children>)>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    if !state_store.is_changed() {
        return;
    }

    for (entity, list, children) in &list_groups {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn_related::<Children>();
                commands.entity(child).despawn();
            }
        }

        match state_store.0.get(&list.source) {
            Some(BuiBindingValue::StringList(items)) => {
                for (index, item) in items.iter().enumerate() {
                    let template =
                        instantiate_list_item_template_text(&list.item_template, index, item);
                    let Ok(child_entity) = spawn_bui_node(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlases,
                        &template,
                    ) else {
                        continue;
                    };
                    commands.entity(entity).add_child(child_entity);
                }
            }
            Some(BuiBindingValue::ObjectList(items)) => {
                for (index, item) in items.iter().enumerate() {
                    let template =
                        instantiate_list_item_template_object(&list.item_template, index, item);
                    let Ok(child_entity) = spawn_bui_node(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlases,
                        &template,
                    ) else {
                        continue;
                    };
                    commands.entity(entity).add_child(child_entity);
                }
            }
            _ => {}
        }
    }
}
