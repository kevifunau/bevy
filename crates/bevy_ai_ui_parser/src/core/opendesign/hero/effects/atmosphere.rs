use crate::core::{
    model::BuiNode, style::css_effects::scale_helper_child_opacity,
    support::tree::find_bui_node_mut,
};

fn set_orb_overlay(
    node: &mut BuiNode,
    color: &str,
    width: &str,
    left: &str,
    top: &str,
    aspect_ratio: &str,
) {
    node.style.visuals.background_color = Some(color.to_string());
    node.layout.styles.width = Some(width.to_string());
    node.layout.styles.height = None;
    node.layout.styles.aspect_ratio = Some(aspect_ratio.to_string());
    node.layout.styles.left = Some(left.to_string());
    node.layout.styles.top = Some(top.to_string());
    node.layout.styles.right = None;
    node.layout.styles.bottom = None;
    node.style.visuals.border_radius = Some("50%".to_string());
}

fn set_ellipse_overlay(
    node: &mut BuiNode,
    color: &str,
    width: &str,
    height: &str,
    left: &str,
    top: &str,
) {
    node.style.visuals.background_color = Some(color.to_string());
    node.layout.styles.width = Some(width.to_string());
    node.layout.styles.height = Some(height.to_string());
    node.layout.styles.aspect_ratio = None;
    node.layout.styles.left = Some(left.to_string());
    node.layout.styles.top = Some(top.to_string());
    node.layout.styles.right = None;
    node.layout.styles.bottom = None;
    node.style.visuals.border_radius = Some("50%".to_string());
}

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
        scale_helper_child_opacity(node, 0.018);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_before_gradient_overlay" => {
                    set_orb_overlay(child, "#CDEFFF1C", "14%", "13%", "3%", "1");
                }
                "image_layer_pseudo_before_gradient_overlay_2" => {
                    set_orb_overlay(child, "#BEDFF114", "22%", "6%", "0%", "1");
                }
                "image_layer_pseudo_before_gradient_overlay_3" => {
                    set_orb_overlay(child, "#A8C7DB09", "26%", "4%", "-2%", "0.98");
                }
                "image_layer_pseudo_before_gradient_overlay_4" => {
                    set_ellipse_overlay(child, "#8FAABD02", "44%", "98%", "-22%", "-30%");
                }
                "image_layer_pseudo_before_gradient_overlay_5" => {
                    set_orb_overlay(child, "#71879503", "18%", "21%", "22%", "1.1");
                }
                "image_layer_pseudo_before_gradient_overlay_6" => {
                    set_ellipse_overlay(child, "#9A8F6704", "30%", "20%", "24%", "67%");
                }
                "image_layer_pseudo_before_gradient_overlay_7" => {
                    set_ellipse_overlay(child, "#5B697804", "20%", "18%", "50%", "24%");
                }
                "image_layer_pseudo_before_gradient_overlay_8" => {
                    set_ellipse_overlay(child, "#34414C03", "17%", "15%", "61%", "19%");
                }
                "image_layer_pseudo_before_gradient_overlay_9" => {
                    set_ellipse_overlay(child, "#37354A02", "15%", "13%", "68%", "22%");
                }
                "image_layer_pseudo_before_gradient_overlay_10" => {
                    set_ellipse_overlay(child, "#302E4102", "13%", "12%", "74%", "17%");
                }
                "image_layer_pseudo_before_gradient_overlay_11" => {
                    set_ellipse_overlay(child, "#2D2B3C01", "12%", "10%", "78%", "14%");
                }
                "image_layer_pseudo_before_gradient_overlay_12" => {
                    set_ellipse_overlay(child, "#2A273601", "10%", "9%", "82%", "12%");
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
