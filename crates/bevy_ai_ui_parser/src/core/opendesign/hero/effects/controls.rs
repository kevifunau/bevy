use crate::core::{
    model::BuiNode,
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_backbutton(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "backbutton") {
        node.style.visuals.background_color = Some("#5A4D4342".to_string());
        node.style.visuals.border_color = Some("#B8944F72".to_string());
        node.style.visuals.border_width = Some("1px".to_string());
        if let Some(box_shadow) = node.style.visuals.box_shadow.as_mut() {
            box_shadow.blur_radius = Some("8px".to_string());
            box_shadow.color = Some("#14050614".to_string());
        }

        for child in &mut node.children {
            match child.id.as_str() {
                "backbutton_box_shadow_layer_1" => {
                    if let Some(box_shadow) = child.style.visuals.box_shadow.as_mut() {
                        box_shadow.color = Some("#C4BFBF14".to_string());
                    }
                }
                "backbutton_box_shadow_layer_2" => {
                    if let Some(box_shadow) = child.style.visuals.box_shadow.as_mut() {
                        box_shadow.blur_radius = Some("8px".to_string());
                        box_shadow.color = Some("#180E1314".to_string());
                    }
                }
                "backbutton_gradient_overlay" => {
                    child.style.visuals.background_color = Some("#FFF4D012".to_string());
                }
                "backbutton_gradient_overlay_2" => {
                    child.style.visuals.background_color = Some("#FFF1CB10".to_string());
                }
                "backbutton_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#FFF0CA0C".to_string());
                }
                "backbutton_gradient_overlay_4" => {
                    child.style.visuals.background_color = Some("#FFF0C806".to_string());
                }
                _ => {}
            }
        }
    }
}

pub(super) fn soften_title_text(root: &mut BuiNode) {
    if let Some(text) = find_bui_node_mut(root, "page_title_text_1")
        && let Some(text_config) = text.content.text.as_mut()
    {
        text_config.font_color = "#D59B10".to_string();
        if let Some(shadow) = &mut text_config.text_shadow {
            shadow.color = Some("#915812".to_string());
        }
    }

    for id in ["small_text_1", "levelvalue_text_1"] {
        if let Some(text) = find_bui_node_mut(root, id)
            && let Some(text_config) = text.content.text.as_mut()
        {
            text_config.font_color = "#D2A11A".to_string();
            if let Some(shadow) = &mut text_config.text_shadow {
                shadow.color = Some("#5C3612B8".to_string());
            }
        }
    }

    if let Some(text) = find_bui_node_mut(root, "hero_name_text_1")
        && let Some(text_config) = text.content.text.as_mut()
    {
        text_config.font_color = "#F5E8CB".to_string();
        if let Some(shadow) = &mut text_config.text_shadow {
            shadow.color = Some("#4C352780".to_string());
        }
    }
}