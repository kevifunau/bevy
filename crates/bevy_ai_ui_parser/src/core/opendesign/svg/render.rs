use crate::core::model::{bui_node, BuiImageConfig, BuiNode};

use super::extract::svg_viewbox_size;

pub(crate) fn is_svg_tag(tag: &str) -> bool {
    matches!(
        tag,
        "svg" | "path" | "circle" | "ellipse" | "rect" | "line" | "polyline" | "polygon" | "g"
    )
}

pub(crate) fn svg_image_node(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
    index: usize,
    png_path: &str,
) -> BuiNode {
    let (w, h) = svg_viewbox_size(svg_node);
    let mut node = bui_node(&format!("{}_svg_{}", parent.id, index), "image");
    node.content.image = Some(BuiImageConfig {
        texture_path: png_path.to_string(),
        image_mode: Some("stretch".to_string()),
        background_size: None,
        background_position: None,
        background_repeat: None,
        atlas: None,
        slicer: None,
        flip_x: false,
        flip_y: false,
    });
    node.layout.styles.width = Some(format!("{:.0}px", w));
    node.layout.styles.height = Some(format!("{:.0}px", h));
    node
}
