use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiVisuals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) border_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) border_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) box_shadow: Option<BuiBoxShadowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) material_shader: Option<String>,
}

impl BuiVisuals {
    pub(crate) fn is_empty(&self) -> bool {
        self.background_color.is_none()
            && self.border_color.is_none()
            && self.border_width.is_none()
            && self.border_radius.is_none()
            && self.box_shadow.is_none()
            && self.material_shader.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiBoxShadowConfig {
    #[serde(default, skip_serializing_if = "is_false")]
    pub(crate) inset: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) offset_x: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) offset_y: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) blur_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) spread_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) color: Option<String>,
}

#[derive(Component, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiTextConfig {
    pub(crate) content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) placeholder: Option<String>,
    pub(crate) font_size: f32,
    pub(crate) font_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) font_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) font_weight: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) letter_spacing: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text_shadow: Option<BuiTextShadowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) linebreak: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) visible_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) allow_newlines: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiTextShadowConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) offset_x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) offset_y: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiImageConfig {
    pub(crate) texture_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background_position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) atlas: Option<BuiTextureAtlasConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) slicer: Option<BuiTextureSlicerConfig>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub(crate) flip_x: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub(crate) flip_y: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiTextureAtlasConfig {
    pub(crate) tile_width: u32,
    pub(crate) tile_height: u32,
    pub(crate) columns: u32,
    pub(crate) rows: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding_x: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding_y: Option<u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiTextureSlicerConfig {
    pub(crate) border: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) center_scale_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) sides_scale_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) stretch_value: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_corner_scale: Option<f32>,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiBackgroundImageLayout {
    pub(crate) size: Option<String>,
    pub(crate) position: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}
