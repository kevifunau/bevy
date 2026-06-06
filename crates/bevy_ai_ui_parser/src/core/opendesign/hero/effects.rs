use crate::core::{
    model::{BuiNode, BuiNodeType},
    opendesign::build::bui_node,
    style::{css_effects::scale_helper_child_opacity, css_values::scale_hex_alpha},
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_hero_game_ui_effect_fallbacks(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "crest") {
        node.visuals.background_color = None;
        node.visuals.border_color = Some("#51617018".to_string());
        node.styles.ui_opacity = Some(0.08);
        scale_helper_child_opacity(node, 0.12);
        for child in &mut node.children {
            match child.id.as_str() {
                "crest_gradient_overlay" => {
                    child.visuals.border_color = Some("#3952640C".to_string());
                }
                "crest_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#50697B05".to_string());
                }
                "crest_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#4A627404".to_string());
                }
                _ => {}
            }
        }
    }

    if let Some(node) = find_bui_node_mut(root, "image_layer_pseudo_before") {
        node.visuals.background_color = None;
        scale_helper_child_opacity(node, 0.12);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_before_gradient_overlay" => {
                    child.visuals.background_color = Some("#A8D3E40C".to_string());
                    child.styles.width = Some("28%".to_string());
                    child.styles.height = Some("28%".to_string());
                    child.styles.left = Some("4%".to_string());
                    child.styles.top = Some("-2%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#A6D0E009".to_string());
                    child.styles.width = Some("18%".to_string());
                    child.styles.height = Some("18%".to_string());
                    child.styles.left = Some("9%".to_string());
                    child.styles.top = Some("3%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#9CC9DA02".to_string());
                    child.styles.left = Some("6%".to_string());
                    child.styles.right = Some("58%".to_string());
                    child.styles.top = Some("0".to_string());
                    child.styles.bottom = Some("0".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#B7CBD400".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_5" => {
                    child.visuals.background_color = Some("#6A514402".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_6" => {
                    child.visuals.background_color = Some("#38271F06".to_string());
                }
                _ => {}
            }
        }
    }

    if let Some(node) = find_bui_node_mut(root, "image_layer_pseudo_after") {
        node.visuals.background_color = None;
        scale_helper_child_opacity(node, 0.1);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_after_gradient_overlay" => {
                    child.visuals.background_color = Some("#C39C5604".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#BE965004".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#B98F4703".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#160C1002".to_string());
                }
                _ => {}
            }
        }
    }

    if let Some(node) = find_bui_node_mut(root, "hero_glow") {
        scale_helper_child_opacity(node, 0.05);
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
                    child.visuals.background_color = Some("#D8C58F02".to_string());
                }
                "hero_glow_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#D6C08A03".to_string());
                }
                "hero_glow_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#C9AE7604".to_string());
                }
                _ => {}
            }
        }
    }

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

    if let Some(text) = find_bui_node_mut(root, "page_title_text_1")
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = "#D59B10".to_string();
        if let Some(shadow) = &mut text_config.text_shadow {
            shadow.color = Some("#915812".to_string());
        }
    }

    for id in ["small_text_1", "levelvalue_text_1"] {
        if let Some(text) = find_bui_node_mut(root, id)
            && let Some(text_config) = text.text_config.as_mut()
        {
            text_config.font_color = "#D2A11A".to_string();
            if let Some(shadow) = &mut text_config.text_shadow {
                shadow.color = Some("#5C3612B8".to_string());
            }
        }
    }

    if let Some(text) = find_bui_node_mut(root, "hero_name_text_1")
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = "#F5E8CB".to_string();
        if let Some(shadow) = &mut text_config.text_shadow {
            shadow.color = Some("#4C352780".to_string());
        }
    }

    if let Some(node) = find_bui_node_mut(root, "info_panel") {
        node.visuals.background_color = Some("#BE9D7A86".to_string());
        if let Some(box_shadow) = node.visuals.box_shadow.as_mut() {
            box_shadow.offset_x = Some("-10px".to_string());
            box_shadow.blur_radius = Some("42px".to_string());
            box_shadow.color = Some("#E7CFAB18".to_string());
        }

        for child in &mut node.children {
            match child.id.as_str() {
                "info_panel_gradient_overlay" => child.visuals.background_color = Some("#BD9B7234".to_string()),
                "info_panel_gradient_overlay_2" => child.visuals.background_color = Some("#A27F5D28".to_string()),
                "info_panel_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#F0DEB206".to_string());
                    child.styles.width = Some("56%".to_string());
                    child.styles.height = Some("56%".to_string());
                    child.styles.left = Some("73%".to_string());
                    child.styles.top = Some("18%".to_string());
                }
                "info_panel_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#F0DEB207".to_string());
                    child.styles.width = Some("43%".to_string());
                    child.styles.height = Some("43%".to_string());
                    child.styles.left = Some("79%".to_string());
                    child.styles.top = Some("24%".to_string());
                }
                "info_panel_gradient_overlay_5" => child.visuals.background_color = Some("#F0DEB20B".to_string()),
                "info_panel_gradient_overlay_6" => child.visuals.background_color = Some("#A9856420".to_string()),
                "info_panel_gradient_overlay_7" => child.visuals.background_color = Some("#9D7D5E22".to_string()),
                "info_panel_gradient_overlay_8" => child.visuals.background_color = Some("#916F5624".to_string()),
                "info_panel_gradient_overlay_9" => child.visuals.background_color = Some("#F0DEB204".to_string()),
                "info_panel_gradient_overlay_10" => child.visuals.background_color = Some("#F0DEB205".to_string()),
                "info_panel_left_cut_1" => child.visuals.background_color = Some("#261F270E".to_string()),
                "info_panel_left_cut_2" => child.visuals.background_color = Some("#58493E10".to_string()),
                "info_panel_left_cut_3" => child.visuals.background_color = Some("#AC8D680C".to_string()),
                "info_panel_left_mask_soft" => child.visuals.background_color = Some("#D9B67E08".to_string()),
                "info_panel_left_inner_glow" => child.visuals.background_color = Some("#F3D8A208".to_string()),
                "info_panel_mid_warmth" => child.visuals.background_color = Some("#E6C08A06".to_string()),
                "info_panel_right_sheen" => child.visuals.background_color = Some("#F0D4A203".to_string()),
                "info_panel_right_hotspot" => {
                    child.visuals.background_color = Some("#FFF0C804".to_string());
                    if let Some(box_shadow) = child.visuals.box_shadow.as_mut() {
                        box_shadow.blur_radius = Some("8px".to_string());
                        box_shadow.spread_radius = Some("1px".to_string());
                        box_shadow.color = Some("#FFF1D004".to_string());
                    }
                }
                "info_panel_lower_ember" => child.visuals.background_color = Some("#7C4F3B04".to_string()),
                _ => {}
            }
        }

        ensure_hero_panel_band(node, "info_panel_bottom_veil_1", "0", "0", "36%", "0", "#47362B0A", "13");
        ensure_hero_panel_band(node, "info_panel_bottom_veil_2", "0", "0", "48%", "0", "#47362B10", "14");
        ensure_hero_panel_band(node, "info_panel_bottom_veil_3", "0", "0", "60%", "0", "#47362B16", "15");
        ensure_hero_panel_band(node, "info_panel_bottom_veil_4", "0", "0", "72%", "0", "#47362B20", "16");
    }

    for (row_id, opacity) in [
        ("hero_stat_row_1", 1.0),
        ("hero_stat_row_2", 0.96),
        ("hero_stat_row_3", 0.86),
        ("hero_stat_row_4", 0.0),
        ("hero_stat_row_5", 0.0),
    ] {
        if let Some(row) = find_bui_node_mut(root, row_id) {
            row.styles.ui_opacity = Some(opacity);
            if opacity == 0.0 {
                row.styles.visibility = Some("hidden".to_string());
                row.styles.min_height = Some("0px".to_string());
            }
        }
    }

    if let Some(action_strip) = find_bui_node_mut(root, "action_strip") {
        action_strip.styles.ui_opacity = Some(0.0);
        action_strip.styles.visibility = Some("hidden".to_string());
    }

    for meter_id in ["meter_fill", "meter_fill_2"] {
        if let Some(node) = find_bui_node_mut(root, meter_id) {
            for child in &mut node.children {
                match child.id.as_str() {
                    "meter_fill_gradient_overlay" | "meter_fill_2_gradient_overlay" => {
                        child.visuals.background_color = Some("#D6FFF024".to_string());
                    }
                    "meter_fill_gradient_overlay_2" | "meter_fill_2_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#D6FFF01A".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    for skill_id in ["skill_button", "skill_button_2", "skill_button_3"] {
        if let Some(node) = find_bui_node_mut(root, skill_id) {
            node.visuals.background_color = Some("#5A4A43EE".to_string());
            node.visuals.border_color = Some("#D7C4A7A6".to_string());
            for child in &mut node.children {
                match child.id.as_str() {
                    "skill_button_gradient_overlay"
                    | "skill_button_2_gradient_overlay"
                    | "skill_button_3_gradient_overlay" => {
                        child.visuals.background_color = Some("#FFF6D20A".to_string());
                    }
                    "skill_button_gradient_overlay_2"
                    | "skill_button_2_gradient_overlay_2"
                    | "skill_button_3_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#FFF6D20D".to_string());
                    }
                    "skill_button_gradient_overlay_3"
                    | "skill_button_2_gradient_overlay_3"
                    | "skill_button_3_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#FFF6D210".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    for equip_id in ["equip_slot", "equip_slot_2", "equip_slot_3", "equip_slot_4", "equip_slot_5"] {
        if let Some(node) = find_bui_node_mut(root, equip_id) {
            if node.id == "equip_slot" {
                node.visuals.background_color = Some("#6B5744A0".to_string());
            } else {
                node.visuals.background_color = Some("#6C5B4C92".to_string());
            }

            for child in &mut node.children {
                match child.id.as_str() {
                    "equip_slot_gradient_overlay"
                    | "equip_slot_2_gradient_overlay"
                    | "equip_slot_3_gradient_overlay"
                    | "equip_slot_4_gradient_overlay"
                    | "equip_slot_5_gradient_overlay" => {
                        child.visuals.background_color = Some("#2B211E4A".to_string());
                    }
                    "equip_slot_gradient_overlay_2"
                    | "equip_slot_2_gradient_overlay_2"
                    | "equip_slot_3_gradient_overlay_2"
                    | "equip_slot_4_gradient_overlay_2"
                    | "equip_slot_5_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#FFF8DE08".to_string());
                    }
                    "equip_slot_gradient_overlay_3"
                    | "equip_slot_2_gradient_overlay_3"
                    | "equip_slot_3_gradient_overlay_3"
                    | "equip_slot_4_gradient_overlay_3"
                    | "equip_slot_5_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#FFF8DE0C".to_string());
                    }
                    "equip_slot_gradient_overlay_4"
                    | "equip_slot_2_gradient_overlay_4"
                    | "equip_slot_3_gradient_overlay_4"
                    | "equip_slot_4_gradient_overlay_4"
                    | "equip_slot_5_gradient_overlay_4" => {
                        child.visuals.background_color = Some("#FFF8DE10".to_string());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ensure_hero_panel_band(
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
