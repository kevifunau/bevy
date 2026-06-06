use crate::core::{
    model::{
        BuiBoxShadowConfig, BuiImageConfig, BuiStyles, BuiTextConfig, BuiTextureAtlasConfig,
        BuiTextureSlicerConfig, BuiVisuals,
    },
    style::css_parser::{
        parse_align_content, parse_align_items, parse_align_self, parse_border_radius, parse_color,
        parse_display, parse_flex_direction, parse_flex_wrap, parse_grid_placement,
        parse_grid_tracks, parse_integer, parse_justify_content, parse_justify_items,
        parse_justify_self, parse_linebreak, parse_node_image_mode, parse_number, parse_overflow,
        parse_overflow_clip_margin, parse_position_type, parse_tab_group, parse_text_line_height,
        parse_ui_rect, parse_val,
    },
};

pub(super) fn validate_visuals(visuals: &BuiVisuals) -> Result<(), String> {
    if let Some(color) = &visuals.background_color {
        parse_color(color)?;
    }
    if let Some(color) = &visuals.border_color {
        parse_color(color)?;
    }
    if let Some(border_width) = &visuals.border_width {
        parse_ui_rect(border_width)?;
    }
    if let Some(border_radius) = &visuals.border_radius {
        parse_border_radius(border_radius)?;
    }
    if let Some(box_shadow) = &visuals.box_shadow {
        validate_box_shadow(box_shadow)?;
    }
    if let Some(shader) = &visuals.material_shader
        && shader.trim().is_empty()
    {
        return Err("visuals.material_shader must not be empty.".to_string());
    }
    Ok(())
}

fn validate_box_shadow(box_shadow: &BuiBoxShadowConfig) -> Result<(), String> {
    if let Some(value) = &box_shadow.offset_x {
        parse_val(value)?;
    }
    if let Some(value) = &box_shadow.offset_y {
        parse_val(value)?;
    }
    if let Some(value) = &box_shadow.blur_radius {
        parse_val(value)?;
    }
    if let Some(value) = &box_shadow.spread_radius {
        parse_val(value)?;
    }
    if let Some(color) = &box_shadow.color {
        parse_color(color)?;
    }
    Ok(())
}

pub(super) fn validate_text_config(text_config: &BuiTextConfig) -> Result<(), String> {
    if text_config.font_size <= 0.0 {
        return Err("text_config.font_size must be greater than 0.".to_string());
    }
    parse_color(&text_config.font_color)?;
    if let Some(font_path) = &text_config.font_path
        && font_path.trim().is_empty()
    {
        return Err("text_config.font_path must not be empty when present.".to_string());
    }
    if let Some(placeholder) = &text_config.placeholder
        && placeholder.trim().is_empty()
    {
        return Err("text_config.placeholder must not be empty when present.".to_string());
    }
    if let Some(text_shadow) = &text_config.text_shadow
        && let Some(color) = &text_shadow.color
    {
        parse_color(color)?;
    }
    if let Some(font_weight) = text_config.font_weight
        && !(1..=1000).contains(&font_weight)
    {
        return Err("text_config.font_weight must be between 1 and 1000.".to_string());
    }
    if let Some(line_height) = &text_config.line_height {
        parse_text_line_height(line_height)?;
    }
    if let Some(linebreak) = &text_config.linebreak {
        parse_linebreak(linebreak)?;
    }
    if let Some(visible_width) = text_config.visible_width
        && visible_width <= 0.0
    {
        return Err("text_config.visible_width must be greater than 0 when present.".to_string());
    }
    Ok(())
}

pub(super) fn validate_styles(styles: &BuiStyles) -> Result<(), String> {
    if let Some(target_camera) = &styles.ui_target_camera
        && target_camera.trim().is_empty()
    {
        return Err("styles.ui_target_camera must not be empty when present.".to_string());
    }
    if let Some(tab_group) = &styles.tab_group {
        parse_tab_group(tab_group)?;
    }
    if let Some(tab_index) = &styles.tab_index {
        parse_integer(tab_index)?;
    }
    if let Some(value) = &styles.display {
        parse_display(value)?;
    }
    if let Some(value) = &styles.width {
        parse_val(value)?;
    }
    if let Some(value) = &styles.height {
        parse_val(value)?;
    }
    if let Some(value) = &styles.min_width {
        parse_val(value)?;
    }
    if let Some(value) = &styles.min_height {
        parse_val(value)?;
    }
    if let Some(value) = &styles.max_width {
        parse_val(value)?;
    }
    if let Some(value) = &styles.max_height {
        parse_val(value)?;
    }
    if let Some(value) = &styles.left {
        parse_val(value)?;
    }
    if let Some(value) = &styles.right {
        parse_val(value)?;
    }
    if let Some(value) = &styles.top {
        parse_val(value)?;
    }
    if let Some(value) = &styles.bottom {
        parse_val(value)?;
    }
    if let Some(value) = &styles.margin {
        parse_ui_rect(value)?;
    }
    if let Some(value) = &styles.padding {
        parse_ui_rect(value)?;
    }
    if let Some(value) = &styles.overflow {
        parse_overflow(value)?;
    }
    if let Some(value) = &styles.overflow_clip_margin {
        parse_overflow_clip_margin(value)?;
    }
    if let Some(value) = &styles.flex_direction {
        parse_flex_direction(value)?;
    }
    if let Some(value) = &styles.flex_wrap {
        parse_flex_wrap(value)?;
    }
    if let Some(value) = &styles.position_type {
        parse_position_type(value)?;
    }
    if let Some(value) = &styles.grid_template_columns {
        parse_grid_tracks(value)?;
    }
    if let Some(value) = &styles.grid_template_rows {
        parse_grid_tracks(value)?;
    }
    if let Some(value) = &styles.grid_column {
        parse_grid_placement(value)?;
    }
    if let Some(value) = &styles.grid_row {
        parse_grid_placement(value)?;
    }
    if let Some(value) = &styles.aspect_ratio {
        parse_number(value)?;
    }
    if let Some(value) = &styles.justify_content {
        parse_justify_content(value)?;
    }
    if let Some(value) = &styles.justify_items {
        parse_justify_items(value)?;
    }
    if let Some(value) = &styles.align_content {
        parse_align_content(value)?;
    }
    if let Some(value) = &styles.align_items {
        parse_align_items(value)?;
    }
    if let Some(value) = &styles.align_self {
        parse_align_self(value)?;
    }
    if let Some(value) = &styles.justify_self {
        parse_justify_self(value)?;
    }
    if let Some(value) = &styles.flex_grow {
        parse_number(value)?;
    }
    if let Some(value) = &styles.flex_shrink {
        parse_number(value)?;
    }
    if let Some(value) = &styles.flex_basis {
        parse_val(value)?;
    }
    if let Some(value) = &styles.row_gap {
        parse_val(value)?;
    }
    if let Some(value) = &styles.column_gap {
        parse_val(value)?;
    }
    if let Some(value) = &styles.margin_left {
        parse_val(value)?;
    }
    if let Some(value) = &styles.margin_right {
        parse_val(value)?;
    }
    if let Some(value) = &styles.margin_top {
        parse_val(value)?;
    }
    if let Some(value) = &styles.margin_bottom {
        parse_val(value)?;
    }
    if let Some(value) = &styles.padding_left {
        parse_val(value)?;
    }
    if let Some(value) = &styles.padding_right {
        parse_val(value)?;
    }
    if let Some(value) = &styles.padding_top {
        parse_val(value)?;
    }
    if let Some(value) = &styles.padding_bottom {
        parse_val(value)?;
    }
    Ok(())
}

pub(super) fn validate_image_config(image_config: &BuiImageConfig) -> Result<(), String> {
    if image_config.texture_path.trim().is_empty() {
        return Err("image_config.texture_path must not be empty.".to_string());
    }
    if let Some(image_mode) = &image_config.image_mode {
        parse_node_image_mode(image_mode, image_config.slicer.as_ref())?;
    }
    if image_config.slicer.is_some()
        && !matches!(image_config.image_mode.as_deref(), Some("sliced"))
    {
        return Err("image_config.slicer requires image_mode 'sliced'.".to_string());
    }
    if let Some(atlas) = &image_config.atlas {
        validate_texture_atlas_config(atlas)?;
    }
    if let Some(slicer) = &image_config.slicer {
        validate_texture_slicer_config(slicer)?;
    }
    Ok(())
}

fn validate_texture_atlas_config(atlas: &BuiTextureAtlasConfig) -> Result<(), String> {
    if atlas.tile_width == 0 || atlas.tile_height == 0 {
        return Err("image_config.atlas tile size must be greater than 0.".to_string());
    }
    if atlas.columns == 0 || atlas.rows == 0 {
        return Err("image_config.atlas grid size must be greater than 0.".to_string());
    }
    let cell_count = atlas.columns as usize * atlas.rows as usize;
    if atlas.index >= cell_count {
        return Err(format!(
            "image_config.atlas.index '{}' is out of range for a {}x{} atlas.",
            atlas.index, atlas.columns, atlas.rows
        ));
    }
    Ok(())
}

fn validate_texture_slicer_config(slicer: &BuiTextureSlicerConfig) -> Result<(), String> {
    if slicer.border < 0.0 {
        return Err("image_config.slicer.border must be non-negative.".to_string());
    }
    if slicer.max_corner_scale.is_some_and(|value| value < 0.0) {
        return Err("image_config.slicer.max_corner_scale must be non-negative.".to_string());
    }
    if slicer.stretch_value.is_some_and(|value| value < 0.0) {
        return Err("image_config.slicer.stretch_value must be non-negative.".to_string());
    }
    Ok(())
}
