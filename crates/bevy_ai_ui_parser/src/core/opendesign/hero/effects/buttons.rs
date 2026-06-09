use crate::core::{model::BuiNode, support::tree::find_bui_node_mut};

pub(super) fn soften_skill_buttons(root: &mut BuiNode) {
    for skill_id in ["skill_button", "skill_button_2", "skill_button_3"] {
        if let Some(node) = find_bui_node_mut(root, skill_id) {
            node.style.visuals.background_color = Some("#5A4A43D8".to_string());
            node.style.visuals.border_color = Some("#D7C4A796".to_string());
            for child in &mut node.children {
                match child.id.as_str() {
                    "skill_button_gradient_overlay"
                    | "skill_button_2_gradient_overlay"
                    | "skill_button_3_gradient_overlay" => {
                        child.style.visuals.background_color = Some("#FFF6D206".to_string());
                    }
                    "skill_button_gradient_overlay_2"
                    | "skill_button_2_gradient_overlay_2"
                    | "skill_button_3_gradient_overlay_2" => {
                        child.style.visuals.background_color = Some("#FFF6D208".to_string());
                    }
                    "skill_button_gradient_overlay_3"
                    | "skill_button_2_gradient_overlay_3"
                    | "skill_button_3_gradient_overlay_3" => {
                        child.style.visuals.background_color = Some("#FFF6D20A".to_string());
                    }
                    _ => {}
                }
            }
        }
    }
}

pub(super) fn soften_equip_slots(root: &mut BuiNode) {
    for equip_id in [
        "equip_slot",
        "equip_slot_2",
        "equip_slot_3",
        "equip_slot_4",
        "equip_slot_5",
    ] {
        if let Some(node) = find_bui_node_mut(root, equip_id) {
            if node.id == "equip_slot" {
                node.style.visuals.background_color = Some("#6B57448A".to_string());
            } else {
                node.style.visuals.background_color = Some("#6C5B4C7C".to_string());
            }

            for child in &mut node.children {
                match child.id.as_str() {
                    "equip_slot_gradient_overlay"
                    | "equip_slot_2_gradient_overlay"
                    | "equip_slot_3_gradient_overlay"
                    | "equip_slot_4_gradient_overlay"
                    | "equip_slot_5_gradient_overlay" => {
                        child.style.visuals.background_color = Some("#2B211E24".to_string());
                    }
                    "equip_slot_gradient_overlay_2"
                    | "equip_slot_2_gradient_overlay_2"
                    | "equip_slot_3_gradient_overlay_2"
                    | "equip_slot_4_gradient_overlay_2"
                    | "equip_slot_5_gradient_overlay_2" => {
                        child.style.visuals.background_color = Some("#FFF8DE05".to_string());
                    }
                    "equip_slot_gradient_overlay_3"
                    | "equip_slot_2_gradient_overlay_3"
                    | "equip_slot_3_gradient_overlay_3"
                    | "equip_slot_4_gradient_overlay_3"
                    | "equip_slot_5_gradient_overlay_3" => {
                        child.style.visuals.background_color = Some("#FFF8DE07".to_string());
                    }
                    "equip_slot_gradient_overlay_4"
                    | "equip_slot_2_gradient_overlay_4"
                    | "equip_slot_3_gradient_overlay_4"
                    | "equip_slot_4_gradient_overlay_4"
                    | "equip_slot_5_gradient_overlay_4" => {
                        child.style.visuals.background_color = Some("#FFF8DE09".to_string());
                    }
                    _ => {}
                }
            }
        }
    }
}
