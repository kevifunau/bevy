use std::collections::HashMap;

use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_ui::prelude::*;

use crate::core::{
    interaction::components::BuiListDefinition,
    interaction::types::{BuiBindingValue, BuiStateStore},
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

    if let Some(text_config) = &mut node.content.text {
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

    if let Some(BuiTextConfig { content, .. }) = &mut node.content.text {
        *content = replace_template_tokens(content, item);
    }

    node.children = node
        .children
        .iter()
        .map(|child| instantiate_list_item_template_object(child, index, item))
        .collect();

    node
}

fn update_text_content_in_subtree(
    entity: Entity,
    new_content: &str,
    children_query: &Query<&Children>,
    text_query: &mut Query<&mut Text>,
) {
    if let Ok(mut text) = text_query.get_mut(entity) {
        text.0 = new_content.to_string();
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_text_content_in_subtree(child, new_content, children_query, text_query);
        }
    }
}

pub(crate) fn sync_bui_list_groups_system(
    state_store: Res<BuiStateStore>,
    list_groups: Query<(Entity, &BuiListDefinition, Option<&Children>)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    if !state_store.is_changed() {
        return;
    }

    for (entity, list, existing_children) in &list_groups {
        let current_count = existing_children.map(|c| c.len()).unwrap_or(0);

        match state_store.0.get(&list.source) {
            Some(BuiBindingValue::StringList(items)) => {
                let target_count = items.len();

                if current_count > target_count {
                    if let Some(children) = existing_children {
                        for child in children.iter().skip(target_count) {
                            commands.entity(child).despawn_related::<Children>();
                            commands.entity(child).despawn();
                        }
                    }
                }

                for (index, item) in items.iter().enumerate() {
                    if index < current_count {
                        if let Some(children) = existing_children {
                            if let Some(child_entity) = children.get(index) {
                                let template = instantiate_list_item_template_text(
                                    &list.item_template,
                                    index,
                                    item,
                                );
                                if let Some(text_config) = &template.content.text {
                                    update_text_content_in_subtree(
                                        *child_entity,
                                        &text_config.content,
                                        &children_query,
                                        &mut text_query,
                                    );
                                }
                            }
                        }
                    } else {
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
            }
            Some(BuiBindingValue::ObjectList(items)) => {
                let target_count = items.len();

                if current_count > target_count {
                    if let Some(children) = existing_children {
                        for child in children.iter().skip(target_count) {
                            commands.entity(child).despawn_related::<Children>();
                            commands.entity(child).despawn();
                        }
                    }
                }

                for (index, item) in items.iter().enumerate() {
                    if index < current_count {
                        if let Some(children) = existing_children {
                            if let Some(child_entity) = children.get(index) {
                                let template = instantiate_list_item_template_object(
                                    &list.item_template,
                                    index,
                                    item,
                                );
                                if let Some(BuiTextConfig { content, .. }) = &template.content.text
                                {
                                    update_text_content_in_subtree(
                                        *child_entity,
                                        content,
                                        &children_query,
                                        &mut text_query,
                                    );
                                }
                            }
                        }
                    } else {
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
            }
            _ => {
                if existing_children.is_some() && current_count > 0 {
                    if let Some(children) = existing_children {
                        for child in children.iter() {
                            commands.entity(child).despawn_related::<Children>();
                            commands.entity(child).despawn();
                        }
                    }
                }
            }
        }
    }
}
