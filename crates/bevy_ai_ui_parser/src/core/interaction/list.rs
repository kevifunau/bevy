use std::collections::HashMap;

use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_ui::prelude::*;
use serde_json::Value;

use crate::core::{
    interaction::components::BuiListDefinition,
    interaction::types::{BuiBindingValue, BuiStateStore},
    model::BuiNode,
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

fn replace_optional_template_tokens(value: &mut Option<String>, values: &HashMap<String, String>) {
    if let Some(value) = value {
        *value = replace_template_tokens(value, values);
    }
}

fn replace_node_template_tokens(node: &mut BuiNode, values: &HashMap<String, String>) {
    node.id = replace_template_tokens(&node.id, values);

    for marker in &mut node.markers {
        *marker = replace_template_tokens(marker, values);
    }
    for class in &mut node.classes {
        *class = replace_template_tokens(class, values);
    }
    for action in &mut node.actions {
        action.event = replace_template_tokens(&action.event, values);
        action.emit = replace_template_tokens(&action.emit, values);
    }
    for binding in &mut node.bindings {
        binding.target = replace_template_tokens(&binding.target, values);
        binding.source = replace_template_tokens(&binding.source, values);
    }

    if let Some(text_config) = &mut node.content.text {
        text_config.content = replace_template_tokens(&text_config.content, values);
    }
    if let Some(image_config) = &mut node.content.image {
        image_config.texture_path = replace_template_tokens(&image_config.texture_path, values);
        replace_optional_template_tokens(&mut image_config.image_mode, values);
        replace_optional_template_tokens(&mut image_config.background_size, values);
        replace_optional_template_tokens(&mut image_config.background_position, values);
        replace_optional_template_tokens(&mut image_config.background_repeat, values);
    }
}

fn instantiate_list_item_template_text(template: &BuiNode, index: usize, item: &str) -> BuiNode {
    let mut node = template.clone();
    let values = HashMap::from([
        ("index".to_string(), index.to_string()),
        ("item".to_string(), item.to_string()),
    ]);
    replace_node_template_tokens(&mut node, &values);

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
    let mut values = item.clone();
    values
        .entry("index".to_string())
        .or_insert(index.to_string());
    replace_node_template_tokens(&mut node, &values);

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

#[derive(Debug, Clone)]
pub(crate) struct BuiJsonListData {
    pub(crate) items: Vec<HashMap<String, String>>,
    pub(crate) mode: String,
    pub(crate) page_size: Option<usize>,
    pub(crate) page_source: Option<String>,
}

#[derive(Resource, Debug, Default, Clone)]
pub(crate) struct BuiJsonListStore {
    pub(crate) lists: HashMap<String, BuiJsonListData>,
}

pub(crate) fn seed_json_list_state(store: &BuiJsonListStore, state_store: &mut BuiStateStore) {
    for (source, data) in &store.lists {
        let value = json_list_value_for_state(data, state_store);
        state_store
            .0
            .insert(source.clone(), BuiBindingValue::ObjectList(value));
    }
}

pub(crate) fn sync_json_list_state_system(
    json_lists: Res<BuiJsonListStore>,
    mut state_store: ResMut<BuiStateStore>,
) {
    if json_lists.lists.is_empty() || !state_store.is_changed() {
        return;
    }

    let mut updates = Vec::new();
    for (source, data) in &json_lists.lists {
        if data.mode == "page" && data.page_source.is_some() {
            updates.push((
                source.clone(),
                json_list_value_for_state(data, &state_store),
            ));
        }
    }

    for (source, value) in updates {
        let next = BuiBindingValue::ObjectList(value);
        if state_store.0.get(&source) == Some(&next) {
            continue;
        }
        state_store.0.insert(source, next);
    }
}

pub(crate) fn json_array_to_object_list(
    value: &Value,
) -> Result<Vec<HashMap<String, String>>, String> {
    let Value::Array(items) = value else {
        return Err("JSON list source must contain an array.".to_string());
    };

    items
        .iter()
        .map(|item| {
            let Value::Object(object) = item else {
                return Err("JSON list source array items must be objects.".to_string());
            };
            let mut values = HashMap::new();
            for (key, value) in object {
                values.insert(key.clone(), json_scalar_to_template_value(value));
            }
            enrich_server_item_values(&mut values);
            Ok(values)
        })
        .collect()
}

fn json_scalar_to_template_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

fn enrich_server_item_values(values: &mut HashMap<String, String>) {
    if let (Some(id), Some(name)) = (values.get("id"), values.get("name")) {
        values.insert("label".to_string(), format!("{id}区  {name}"));
    }
    if let Some(state) = values.get("state").cloned() {
        let image = match state.as_str() {
            "1" => "status_smooth.png",
            "2" => "status_busy.png",
            "3" => "status_hot.png",
            "4" => "status_maintenance.png",
            _ => "transparent.png",
        };
        values.insert("stateImage".to_string(), image.to_string());
        values.insert(
            "stateClass".to_string(),
            if state == "0" {
                "state-0".to_string()
            } else {
                format!("state-{state}")
            },
        );
    }
    if let Some(is_new) = values.get("isNew").cloned() {
        values.insert(
            "newClass".to_string(),
            if is_new == "true" {
                "show".to_string()
            } else {
                String::new()
            },
        );
    }
}

fn json_list_value_for_state(
    data: &BuiJsonListData,
    state_store: &BuiStateStore,
) -> Vec<HashMap<String, String>> {
    match data.mode.as_str() {
        "regions" => region_list_items(&data.items, data.page_size.unwrap_or(5)),
        "page" => {
            let page_size = data.page_size.unwrap_or(data.items.len().max(1));
            let page = data
                .page_source
                .as_deref()
                .and_then(|source| state_store.0.get(source))
                .and_then(binding_value_to_index)
                .unwrap_or(0);
            let start = page.saturating_mul(page_size);
            data.items
                .iter()
                .skip(start)
                .take(page_size)
                .cloned()
                .collect()
        }
        _ => data.items.clone(),
    }
}

fn binding_value_to_index(value: &BuiBindingValue) -> Option<usize> {
    match value {
        BuiBindingValue::Number(value) => Some(value.max(0.0) as usize),
        BuiBindingValue::Text(value) => value.parse().ok(),
        _ => None,
    }
}

fn region_list_items(
    items: &[HashMap<String, String>],
    page_size: usize,
) -> Vec<HashMap<String, String>> {
    let page_size = page_size.max(1);
    let mut regions = Vec::new();
    for (index, chunk) in items.chunks(page_size).enumerate() {
        let begin = index * page_size + 1;
        let end = begin + chunk.len().saturating_sub(1);
        regions.push(HashMap::from([
            ("index".to_string(), index.to_string()),
            ("begin".to_string(), begin.to_string()),
            ("end".to_string(), end.to_string()),
            ("label".to_string(), format!("{begin} - {end}区")),
        ]));
    }
    regions
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
                if let Some(children) = existing_children {
                    for child in children.iter() {
                        commands.entity(child).despawn_related::<Children>();
                        commands.entity(child).despawn();
                    }
                }

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
