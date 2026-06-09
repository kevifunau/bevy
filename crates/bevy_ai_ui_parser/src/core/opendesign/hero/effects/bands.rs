use crate::core::model::{bui_node, BuiNode};

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
        existing.layout.styles.position_type = Some("absolute".to_string());
        existing.layout.styles.left = Some(left.to_string());
        existing.layout.styles.right = Some(right.to_string());
        existing.layout.styles.top = Some(top.to_string());
        existing.layout.styles.bottom = Some(bottom.to_string());
        existing.layout.styles.z_index = Some(z_index.to_string());
        existing.style.visuals.background_color = Some(color.to_string());
        return;
    }

    let mut veil = bui_node(id, "node");
    veil.markers.push("hero-info-panel:veil".to_string());
    veil.layout.styles.position_type = Some("absolute".to_string());
    veil.layout.styles.left = Some(left.to_string());
    veil.layout.styles.right = Some(right.to_string());
    veil.layout.styles.top = Some(top.to_string());
    veil.layout.styles.bottom = Some(bottom.to_string());
    veil.layout.styles.z_index = Some(z_index.to_string());
    veil.style.visuals.background_color = Some(color.to_string());
    node.children.push(veil);
}
