use crate::core::{
    model::BuiNode,
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_meters(root: &mut BuiNode) {
    for meter_id in ["meter", "meter_2"] {
        if let Some(node) = find_bui_node_mut(root, meter_id) {
            node.style.visuals.background_color = Some("#4A342796".to_string());
            if let Some(box_shadow) = node.style.visuals.box_shadow.as_mut() {
                box_shadow.color = Some("#F4DFC426".to_string());
            }

            for child in &mut node.children {
                match child.id.as_str() {
                    "meter_gradient_overlay" | "meter_2_gradient_overlay" => {
                        child.style.visuals.background_color = Some("#4A3326AE".to_string());
                    }
                    "meter_gradient_overlay_2" | "meter_2_gradient_overlay_2" => {
                        child.style.visuals.background_color = Some("#452F23A8".to_string());
                    }
                    "meter_gradient_overlay_3" | "meter_2_gradient_overlay_3" => {
                        child.style.visuals.background_color = Some("#402C219F".to_string());
                    }
                    _ => {}
                }
            }
        }
    }
}

pub(super) fn soften_xp_energy_fills(root: &mut BuiNode) {
    for meter_id in ["xpfill", "energyfill"] {
        if let Some(node) = find_bui_node_mut(root, meter_id) {
            node.style.visuals.background_color = Some(if meter_id == "xpfill" {
                "#9ED760".to_string()
            } else {
                "#F0C55A".to_string()
            });
            for child in &mut node.children {
                match child.id.as_str() {
                    "xpfill_gradient_overlay" => {
                        child.style.visuals.background_color = Some("#FFFFFF30".to_string());
                    }
                    "energyfill_gradient_overlay" => {
                        child.style.visuals.background_color = Some("#FFF9DD30".to_string());
                    }
                    "xpfill_gradient_overlay_2" => {
                        child.style.visuals.background_color = Some("#FFFFFF1E".to_string());
                    }
                    "energyfill_gradient_overlay_2" => {
                        child.style.visuals.background_color = Some("#FFF5CF1E".to_string());
                    }
                    "xpfill_gradient_overlay_3" => {
                        child.style.visuals.background_color = Some("#9ED760".to_string());
                    }
                    "xpfill_gradient_overlay_4" => {
                        child.style.visuals.background_color = Some("#A9DC73".to_string());
                    }
                    "xpfill_gradient_overlay_5" => {
                        child.style.visuals.background_color = Some("#B4E085".to_string());
                    }
                    "xpfill_gradient_overlay_6" => {
                        child.style.visuals.background_color = Some("#BFE499".to_string());
                    }
                    "xpfill_gradient_overlay_7" => {
                        child.style.visuals.background_color = Some("#C8E7AA".to_string());
                    }
                    "xpfill_gradient_overlay_8" => {
                        child.style.visuals.background_color = Some("#D1EABC".to_string());
                    }
                    "xpfill_gradient_overlay_9" => {
                        child.style.visuals.background_color = Some("#D8EDC9".to_string());
                    }
                    "energyfill_gradient_overlay_3" => {
                        child.style.visuals.background_color = Some("#F0C55A".to_string());
                    }
                    "energyfill_gradient_overlay_4" => {
                        child.style.visuals.background_color = Some("#F2CD6E".to_string());
                    }
                    "energyfill_gradient_overlay_5" => {
                        child.style.visuals.background_color = Some("#F4D37F".to_string());
                    }
                    "energyfill_gradient_overlay_6" => {
                        child.style.visuals.background_color = Some("#F6DA92".to_string());
                    }
                    "energyfill_gradient_overlay_7" => {
                        child.style.visuals.background_color = Some("#F7E1A5".to_string());
                    }
                    "energyfill_gradient_overlay_8" => {
                        child.style.visuals.background_color = Some("#F8E7B8".to_string());
                    }
                    "energyfill_gradient_overlay_9" => {
                        child.style.visuals.background_color = Some("#F9ECC7".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    for text_id in ["xptext_text_1", "energytext_text_1", "b_text_1", "b_2_text_1"] {
        if let Some(text) = find_bui_node_mut(root, text_id)
            && let Some(text_config) = text.content.text.as_mut()
        {
            text_config.font_color = "#F5EBD6".to_string();
            if let Some(shadow) = &mut text_config.text_shadow {
                shadow.color = Some("#2B180FA6".to_string());
            }
        }
    }
}