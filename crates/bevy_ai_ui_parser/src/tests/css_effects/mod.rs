mod blend_filter;
mod clip_mask_shadow;
mod metadata;
mod svg_fallback;

use super::shared::find_bui_node;
use crate::core::model::bui_node;
use crate::core::opendesign::{
    html::opendesign_html_to_bui_document,
    svg::{semantic_svg_fallback_spec, svg_fallback_icon},
};
use crate::core::style::css_effects::{
    MaskFadeDirection, apply_box_shadow_fallback, css_box_shadow_layers,
    css_simple_clip_polygon_contour, css_simple_mask_fade,
};
use crate::core::style::css_metadata::{
    CssPropertySupportLevel, css_effect_fallback_registry, css_property_info,
};
use crate::core::style::css_values::{
    CssFilterColorAdjustment, css_adjust_filter_color, css_hex_rgba,
    css_multiply_blend_fallback_color,
};