use crate::core::{
    model::BuiNode,
    style::{css_effects::scale_helper_child_opacity, css_values::scale_hex_alpha},
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_hero_glow(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "hero_glow") {
        scale_helper_child_opacity(node, 0.025);
        for child in &mut node.children {
            if child.custom_tags.iter().any(|tag| tag == "css-filter-blur")
                && let Some(box_shadow) = &mut child.visuals.box_shadow
                && let Some(color) = &mut box_shadow.color
                && let Some(scaled) = scale_hex_alpha(color, 0.18)
            {
                *color = scaled;
            }

            match child.id.as_str() {
                "hero_glow_gradient_overlay" => {
                    child.visuals.background_color = Some("#D8C58F01".to_string());
                }
                "hero_glow_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#D6C08A02".to_string());
                }
                "hero_glow_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#C9AE7602".to_string());
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
                .custom_tags
                .iter()
                .any(|tag| tag == "hero-cutout:fallback")
        });
        node.children.retain(|child| {
            child.id != "hero_cutout_filter_drop_shadow_1" && child.id != "hero_cutout_clip_bounds"
        });
        for child in &mut node.children {
            if child.id == "hero_cutout_filter_drop_shadow_2"
                && let Some(box_shadow) = child.visuals.box_shadow.as_mut()
            {
                box_shadow.color = Some("#170C1036".to_string());
                child.visuals.border_radius = Some("42%".to_string());
            }
        }
    }
}