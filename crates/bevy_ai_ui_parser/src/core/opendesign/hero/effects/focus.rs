use crate::core::{
    model::BuiNode,
    style::css_effects::scale_helper_child_opacity,
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_hero_glow(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "hero_glow") {
        scale_helper_child_opacity(node, 0.04);
        for child in &mut node.children {
            if child.markers.iter().any(|tag| tag == "css-filter-blur")
                && let Some(box_shadow) = &mut child.style.visuals.box_shadow
            {
                box_shadow.color = Some("#CDA66110".to_string());
                box_shadow.blur_radius = Some("34px".to_string());
                box_shadow.spread_radius = Some("10px".to_string());
            }

            match child.id.as_str() {
                "hero_glow_gradient_overlay" => {
                    child.style.visuals.background_color = Some("#D7BD7408".to_string());
                    child.layout.styles.width = Some("88%".to_string());
                    child.layout.styles.height = Some("66%".to_string());
                    child.layout.styles.left = Some("8%".to_string());
                    child.layout.styles.top = Some("24%".to_string());
                }
                "hero_glow_gradient_overlay_2" => {
                    child.style.visuals.background_color = Some("#D5B26A0C".to_string());
                    child.layout.styles.width = Some("68%".to_string());
                    child.layout.styles.height = Some("52%".to_string());
                    child.layout.styles.left = Some("18%".to_string());
                    child.layout.styles.top = Some("30%".to_string());
                }
                "hero_glow_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#C59B6207".to_string());
                }
                _ => {}
            }
        }
    }
}

pub(super) fn soften_hero_cutout(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "hero_cutout") {
        node.children.retain(|child| {
            !child
                .markers
                .iter()
                .any(|tag| tag == "hero-cutout:fallback")
        });
        node.children.retain(|child| {
            child.id != "hero_cutout_filter_drop_shadow_1" && child.id != "hero_cutout_clip_bounds"
        });
        for child in &mut node.children {
            if child.id == "hero_cutout_filter_drop_shadow_2"
                && let Some(box_shadow) = child.style.visuals.box_shadow.as_mut()
            {
                box_shadow.color = Some("#170C1036".to_string());
                child.style.visuals.border_radius = Some("42%".to_string());
            }
        }
    }
}
