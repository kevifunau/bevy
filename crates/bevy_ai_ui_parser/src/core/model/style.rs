use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiStyles {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) display: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) visibility: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) min_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) min_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) overflow_clip_margin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) margin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) margin_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) margin_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) margin_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) margin_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) padding_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) flex_direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) flex_wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) flex_grow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) flex_shrink: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) flex_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) row_gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) column_gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) justify_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) justify_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) align_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) align_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) align_self: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) justify_self: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ui_translation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ui_scale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ui_rotation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ui_opacity: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tab_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tab_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auto_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) relative_cursor_position: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ui_target_camera: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) position_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) fixed_node: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) z_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) global_z_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) grid_template_columns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) grid_template_rows: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) grid_column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) grid_row: Option<String>,
}

impl BuiStyles {
    pub(crate) fn is_empty(&self) -> bool {
        self.display.is_none()
            && self.visibility.is_none()
            && self.width.is_none()
            && self.height.is_none()
            && self.aspect_ratio.is_none()
            && self.min_width.is_none()
            && self.min_height.is_none()
            && self.max_width.is_none()
            && self.max_height.is_none()
            && self.left.is_none()
            && self.right.is_none()
            && self.top.is_none()
            && self.bottom.is_none()
            && self.overflow.is_none()
            && self.overflow_clip_margin.is_none()
            && self.margin.is_none()
            && self.margin_left.is_none()
            && self.margin_right.is_none()
            && self.margin_top.is_none()
            && self.margin_bottom.is_none()
            && self.padding.is_none()
            && self.padding_left.is_none()
            && self.padding_right.is_none()
            && self.padding_top.is_none()
            && self.padding_bottom.is_none()
            && self.flex_direction.is_none()
            && self.flex_wrap.is_none()
            && self.flex_grow.is_none()
            && self.flex_shrink.is_none()
            && self.flex_basis.is_none()
            && self.row_gap.is_none()
            && self.column_gap.is_none()
            && self.justify_content.is_none()
            && self.justify_items.is_none()
            && self.align_content.is_none()
            && self.align_items.is_none()
            && self.align_self.is_none()
            && self.justify_self.is_none()
            && self.ui_translation.is_none()
            && self.ui_scale.is_none()
            && self.ui_rotation.is_none()
            && self.ui_opacity.is_none()
            && self.tab_group.is_none()
            && self.tab_index.is_none()
            && self.auto_focus.is_none()
            && self.relative_cursor_position.is_none()
            && self.ui_target_camera.is_none()
            && self.position_type.is_none()
            && self.fixed_node.is_none()
            && self.z_index.is_none()
            && self.global_z_index.is_none()
            && self.grid_template_columns.is_none()
            && self.grid_template_rows.is_none()
            && self.grid_column.is_none()
            && self.grid_row.is_none()
    }
}
