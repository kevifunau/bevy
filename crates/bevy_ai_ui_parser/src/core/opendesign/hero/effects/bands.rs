use crate::core::{
    model::{BuiNode, BuiNodeType, bui_node},
};

pub(super) fn ensure_hero_panel_band(
    node: &mut BuiNode,
    id: &str,
    left: &str,
    right: &str,
    top: &str,
    bottom: &str,
    color: &str,
    z_index: &str,
) {
    if let Some(existing) = node.children.iter_mut().find(|child| child.id == id) {
        existing.styles.position_type = Some("absolute".to_string());
        existing.styles.left = Some(left.to_string());
        existing.styles.right = Some(right.to_string());
        existing.styles.top = Some(top.to_string());
        existing.styles.bottom = Some(bottom.to_string());
        existing.styles.z_index = Some(z_index.to_string());
        existing.visuals.background_color = Some(color.to_string());
        return;
    }

    let mut veil = bui_node(id, BuiNodeType::Node);
    veil.custom_tags.push("hero-info-panel:veil".to_string());
    veil.styles.position_type = Some("absolute".to_string());
    veil.styles.left = Some(left.to_string());
    veil.styles.right = Some(right.to_string());
    veil.styles.top = Some(top.to_string());
    veil.styles.bottom = Some(bottom.to_string());
    veil.styles.z_index = Some(z_index.to_string());
    veil.visuals.background_color = Some(color.to_string());
    node.children.push(veil);
}