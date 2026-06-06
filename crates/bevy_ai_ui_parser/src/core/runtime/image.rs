use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::{TextureAtlas, TextureAtlasLayout};
use bevy_math::{Rect, UVec2, Vec2};
use bevy_ui::{prelude::*, widget::ImageNodeSize};

use crate::core::{
    model::{BuiBackgroundImageLayout, BuiImageConfig},
    style::css_parser::parse_node_image_mode,
};

pub(crate) fn build_image_node(
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    image_config: &BuiImageConfig,
) -> Result<ImageNode, String> {
    let image_mode = image_config
        .image_mode
        .as_deref()
        .map(|value| parse_node_image_mode(value, image_config.slicer.as_ref()))
        .transpose()?
        .unwrap_or_default();

    let image = asset_server.load(&image_config.texture_path);
    let mut image_node = if let Some(atlas) = &image_config.atlas {
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(atlas.tile_width, atlas.tile_height),
            atlas.columns,
            atlas.rows,
            atlas
                .padding_x
                .zip(atlas.padding_y)
                .map(|(x, y)| UVec2::new(x, y)),
            None,
        );
        let layout = texture_atlases.add(layout);
        ImageNode::from_atlas_image(
            image,
            TextureAtlas {
                layout,
                index: atlas.index,
            },
        )
    } else {
        ImageNode::new(image)
    };

    image_node.image_mode = image_mode;
    image_node.flip_x = image_config.flip_x;
    image_node.flip_y = image_config.flip_y;
    Ok(image_node)
}

pub(crate) fn background_image_layout(
    image_config: &BuiImageConfig,
) -> Option<BuiBackgroundImageLayout> {
    if image_config.background_size.is_none() && image_config.background_position.is_none() {
        return None;
    }

    Some(BuiBackgroundImageLayout {
        size: image_config.background_size.clone(),
        position: image_config.background_position.clone(),
    })
}

pub(crate) fn sync_background_image_layout_system(
    mut query: Query<(
        &ComputedNode,
        &BuiBackgroundImageLayout,
        &mut ImageNode,
        &ImageNodeSize,
    )>,
) {
    for (computed_node, layout, mut image_node, image_size) in &mut query {
        let texture_size = image_size.size().as_vec2();
        let node_size = computed_node.size() * computed_node.inverse_scale_factor();

        if texture_size.cmple(Vec2::ZERO).any() || node_size.cmple(Vec2::ZERO).any() {
            continue;
        }

        image_node.rect = compute_background_rect(texture_size, node_size, layout);
    }
}

fn compute_background_rect(
    texture_size: Vec2,
    node_size: Vec2,
    layout: &BuiBackgroundImageLayout,
) -> Option<Rect> {
    let size = layout.size.as_deref().unwrap_or("auto");
    let position = layout.position.as_deref().unwrap_or("center");

    if size.eq_ignore_ascii_case("cover") {
        let scale = (node_size.x / texture_size.x)
            .max(node_size.y / texture_size.y)
            .max(0.0);
        let crop_size = if scale > 0.0 {
            node_size / scale
        } else {
            texture_size
        };
        let origin = background_crop_origin(texture_size, crop_size, position);
        let max = origin + crop_size;
        return Some(Rect { min: origin, max });
    }

    if let Some((width_scale, height_scale)) = css_background_scale(size, texture_size) {
        let crop_size = Vec2::new(
            (node_size.x / width_scale).min(texture_size.x),
            (node_size.y / height_scale).min(texture_size.y),
        );
        let origin = background_crop_origin(texture_size, crop_size, position);
        let max = origin + crop_size;
        return Some(Rect { min: origin, max });
    }

    None
}

fn css_background_scale(value: &str, texture_size: Vec2) -> Option<(f32, f32)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [single] => {
            if single.eq_ignore_ascii_case("contain") || single.eq_ignore_ascii_case("auto") {
                return None;
            }
            let scale = css_background_scale_component(single, texture_size.x)?;
            Some((scale, scale))
        }
        [x, y] => {
            let scale_x = css_background_scale_component(x, texture_size.x)?;
            let scale_y = css_background_scale_component(y, texture_size.y)?;
            Some((scale_x, scale_y))
        }
        _ => None,
    }
}

fn css_background_scale_component(value: &str, texture_axis: f32) -> Option<f32> {
    let value = value.trim();
    if let Some(percent) = value
        .strip_suffix('%')
        .and_then(|part| part.parse::<f32>().ok())
    {
        return Some((percent / 100.0).max(0.0001));
    }
    if let Some(px) = value
        .strip_suffix("px")
        .and_then(|part| part.parse::<f32>().ok())
        && texture_axis > 0.0
    {
        return Some((px / texture_axis).max(0.0001));
    }
    None
}

fn background_crop_origin(texture_size: Vec2, crop_size: Vec2, position: &str) -> Vec2 {
    let (x_ratio, y_ratio) = css_background_position(position);
    Vec2::new(
        ((texture_size.x - crop_size.x).max(0.0) * x_ratio).clamp(0.0, texture_size.x),
        ((texture_size.y - crop_size.y).max(0.0) * y_ratio).clamp(0.0, texture_size.y),
    )
}

fn css_background_position(value: &str) -> (f32, f32) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [] => (0.5, 0.5),
        [single] => {
            let ratio = css_background_position_component(single).unwrap_or(0.5);
            let (x, y) = match (*single).to_ascii_lowercase().as_str() {
                "left" | "right" => (ratio, 0.5),
                "top" | "bottom" => (0.5, ratio),
                _ => (ratio, 0.5),
            };
            (x, y)
        }
        [x, y, ..] => (
            css_background_position_component(x).unwrap_or(0.5),
            css_background_position_component(y).unwrap_or(0.5),
        ),
    }
}

fn css_background_position_component(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "left" | "top" => Some(0.0),
        "center" => Some(0.5),
        "right" | "bottom" => Some(1.0),
        _ => value
            .strip_suffix('%')
            .and_then(|part| part.parse::<f32>().ok())
            .map(|percent| (percent / 100.0).clamp(0.0, 1.0)),
    }
}
