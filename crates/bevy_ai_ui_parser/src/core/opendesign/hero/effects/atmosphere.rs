use crate::core::{
    model::BuiNode,
    style::css_effects::scale_helper_child_opacity,
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_crest(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "crest") {
        node.style.visuals.background_color = None;
        node.style.visuals.border_color = Some("#51617010".to_string());
        node.layout.styles.ui_opacity = Some(0.05);
        scale_helper_child_opacity(node, 0.06);
        for child in &mut node.children {
            match child.id.as_str() {
                "crest_gradient_overlay" => {
                    child.style.visuals.border_color = Some("#39526408".to_string());
                }
                "crest_gradient_overlay_2" => {
                    child.style.visuals.background_color = Some("#50697B02".to_string());
                }
                "crest_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#4A627402".to_string());
                }
                _ => {}
            }
        }
    }
}

pub(super) fn soften_image_layer_before(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "image_layer_pseudo_before") {
        node.style.visuals.background_color = None;
        scale_helper_child_opacity(node, 0.028);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_before_gradient_overlay" => {
                    child.style.visuals.background_color = Some("#C7EBFA0E".to_string());
                    child.layout.styles.width = Some("16%".to_string());
                    child.layout.styles.height = Some("16%".to_string());
                    child.layout.styles.left = Some("9%".to_string());
                    child.layout.styles.top = Some("4%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_2" => {
                    child.style.visuals.background_color = Some("#C0E6F406".to_string());
                    child.layout.styles.width = Some("40%".to_string());
                    child.layout.styles.height = Some("36%".to_string());
                    child.layout.styles.left = Some("-8%".to_string());
                    child.layout.styles.top = Some("-6%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#BFE2F202".to_string());
                    child.layout.styles.width = Some("92%".to_string());
                    child.layout.styles.height = Some("82%".to_string());
                    child.layout.styles.left = Some("-22%".to_string());
                    child.layout.styles.top = Some("-18%".to_string());
                    child.layout.styles.right = None;
                    child.layout.styles.bottom = None;
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_4" => {
                    child.style.visuals.background_color = Some("#BCD1DB00".to_string());
                    child.layout.styles.left = Some("0%".to_string());
                    child.layout.styles.right = Some("50%".to_string());
                    child.layout.styles.top = Some("10%".to_string());
                    child.layout.styles.bottom = Some("20%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_5" => {
                    child.style.visuals.background_color = Some("#8DA7B101".to_string());
                    child.layout.styles.left = Some("18%".to_string());
                    child.layout.styles.right = Some("42%".to_string());
                    child.layout.styles.top = Some("28%".to_string());
                    child.layout.styles.bottom = Some("40%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_6" => {
                    child.style.visuals.background_color = Some("#9BA38F01".to_string());
                    child.layout.styles.left = Some("28%".to_string());
                    child.layout.styles.right = Some("34%".to_string());
                    child.layout.styles.top = Some("60%".to_string());
                    child.layout.styles.bottom = Some("16%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_7" => {
                    child.style.visuals.background_color = Some("#55647401".to_string());
                    child.layout.styles.left = Some("42%".to_string());
                    child.layout.styles.right = Some("26%".to_string());
                    child.layout.styles.top = Some("18%".to_string());
                    child.layout.styles.bottom = Some("28%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_8" => {
                    child.style.visuals.background_color = Some("#2E3A4701".to_string());
                    child.layout.styles.left = Some("54%".to_string());
                    child.layout.styles.right = Some("22%".to_string());
                    child.layout.styles.top = Some("16%".to_string());
                    child.layout.styles.bottom = Some("32%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_9" => {
                    child.style.visuals.background_color = Some("#32314201".to_string());
                    child.layout.styles.left = Some("66%".to_string());
                    child.layout.styles.right = Some("20%".to_string());
                    child.layout.styles.top = Some("18%".to_string());
                    child.layout.styles.bottom = Some("30%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_10" => {
                    child.style.visuals.background_color = Some("#302E3E01".to_string());
                    child.layout.styles.left = Some("70%".to_string());
                    child.layout.styles.right = Some("16%".to_string());
                    child.layout.styles.top = Some("16%".to_string());
                    child.layout.styles.bottom = Some("34%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_11" => {
                    child.style.visuals.background_color = Some("#2D2B3B01".to_string());
                    child.layout.styles.left = Some("74%".to_string());
                    child.layout.styles.right = Some("14%".to_string());
                    child.layout.styles.top = Some("14%".to_string());
                    child.layout.styles.bottom = Some("38%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_12" => {
                    child.style.visuals.background_color = Some("#2A283701".to_string());
                    child.layout.styles.left = Some("78%".to_string());
                    child.layout.styles.right = Some("12%".to_string());
                    child.layout.styles.top = Some("12%".to_string());
                    child.layout.styles.bottom = Some("42%".to_string());
                    child.style.visuals.border_radius = Some("50%".to_string());
                }
                _ => {}
            }
        }
    }
}

pub(super) fn soften_image_layer_after(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "image_layer_pseudo_after") {
        node.style.visuals.background_color = None;
        scale_helper_child_opacity(node, 0.06);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_after_gradient_overlay" => {
                    child.style.visuals.background_color = Some("#C39C5602".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_2" => {
                    child.style.visuals.background_color = Some("#BE965002".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#B98F4702".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_4" => {
                    child.style.visuals.background_color = Some("#160C1001".to_string());
                }
                _ => {}
            }
        }
    }
}