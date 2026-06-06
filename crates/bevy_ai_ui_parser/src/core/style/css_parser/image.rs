use bevy_ui::prelude::*;

use crate::core::model::BuiTextureSlicerConfig;

use super::normalize_token;

fn parse_slice_scale_mode(
    value: &str,
    stretch_value: Option<f32>,
) -> Result<SliceScaleMode, String> {
    match normalize_token(value).as_str() {
        "stretch" => Ok(SliceScaleMode::Stretch),
        "tile" => Ok(SliceScaleMode::Tile {
            stretch_value: stretch_value.unwrap_or(1.0),
        }),
        _ => Err(format!("Invalid image_config.slicer scale mode '{value}'.")),
    }
}

pub(crate) fn parse_node_image_mode(
    value: &str,
    slicer: Option<&BuiTextureSlicerConfig>,
) -> Result<NodeImageMode, String> {
    match normalize_token(value).as_str() {
        "auto" => Ok(NodeImageMode::Auto),
        "stretch" => Ok(NodeImageMode::Stretch),
        "sliced" => {
            let Some(slicer) = slicer else {
                return Err("image_mode 'sliced' requires image_config.slicer.".to_string());
            };

            Ok(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(slicer.border),
                center_scale_mode: parse_slice_scale_mode(
                    slicer.center_scale_mode.as_deref().unwrap_or("stretch"),
                    slicer.stretch_value,
                )?,
                sides_scale_mode: parse_slice_scale_mode(
                    slicer.sides_scale_mode.as_deref().unwrap_or("stretch"),
                    slicer.stretch_value,
                )?,
                max_corner_scale: slicer.max_corner_scale.unwrap_or(1.0),
            }))
        }
        _ => Err(format!(
            "Invalid image_config.image_mode '{value}'. Supported values are 'auto', 'stretch', and 'sliced'."
        )),
    }
}
