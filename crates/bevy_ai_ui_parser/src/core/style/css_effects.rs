mod blend;
mod border;
mod filter;
mod mask;
mod shadow;

pub(crate) use blend::{apply_mix_blend_mode_fallback, scale_helper_child_opacity};
pub(crate) use border::{
    apply_css_border, apply_css_edge_border, apply_css_edge_border_color,
    apply_css_edge_border_width,
};
pub(crate) use filter::{
    apply_filter_blur_fallback, apply_filter_color_adjustment, apply_state_filter_color_adjustment,
    apply_state_opacity_fallback, css_filter_blur_radius, css_filter_color_adjustment,
    css_filter_drop_shadows, css_filter_shadow_length,
};
pub(crate) use mask::{apply_clip_path_fallback, apply_mask_image_fallback};
pub(crate) use shadow::{
    apply_box_shadow_fallback, css_box_shadow, css_text_shadow, node_has_shadow_casting_paint,
    push_box_shadow_layer,
};

#[cfg(test)]
pub(crate) use shadow::css_box_shadow_layers;

#[cfg(test)]
pub(crate) use mask::{css_simple_clip_polygon_contour, css_simple_mask_fade, MaskFadeDirection};
