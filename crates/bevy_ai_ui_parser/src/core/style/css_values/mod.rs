mod background;
mod color;
mod text;

pub(crate) use background::{
    css_background_base_color, css_simple_color, split_css_layers,
};
pub(crate) use color::{
    append_hex_alpha, blend_hex_colors, css_adjust_filter_color, css_color,
    css_hex_rgba, css_multiply_blend_fallback_color, css_percentage_value, scale_hex_alpha,
    CssFilterColorAdjustment,
};
pub(crate) use text::{
    adjust_font_path_for_content, apply_css_white_space, css_font_family_to_path,
    css_font_weight,
};
