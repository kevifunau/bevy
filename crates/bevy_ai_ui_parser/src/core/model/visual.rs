use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiVisuals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_shadow: Option<BuiBoxShadowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_shader: Option<String>,
}

impl BuiVisuals {
    pub fn is_empty(&self) -> bool {
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
pub struct BuiBoxShadowConfig {
    #[serde(default, skip_serializing_if = "is_false")]
    pub inset: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_x: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Component, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiTextConfig {
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    pub font_size: f32,
    pub font_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_shadow: Option<BuiTextShadowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linebreak: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_newlines: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiTextShadowConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiImageConfig {
    pub texture_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_repeat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atlas: Option<BuiTextureAtlasConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slicer: Option<BuiTextureSlicerConfig>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub flip_x: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub flip_y: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiTextureAtlasConfig {
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub rows: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_x: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_y: Option<u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiTextureSlicerConfig {
    pub border: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center_scale_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sides_scale_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch_value: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_corner_scale: Option<f32>,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct BuiBackgroundImageLayout {
    pub size: Option<String>,
    pub position: Option<String>,
    pub repeat: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}
