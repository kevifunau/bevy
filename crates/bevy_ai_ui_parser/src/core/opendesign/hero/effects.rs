use crate::core::{
    model::{BuiNode, BuiNodeType},
    opendesign::build::bui_node,
    style::{css_effects::scale_helper_child_opacity, css_values::scale_hex_alpha},
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_hero_game_ui_effect_fallbacks(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "crest") {
        node.visuals.background_color = None;
        node.visuals.border_color = Some("#51617010".to_string());
        node.styles.ui_opacity = Some(0.05);
        scale_helper_child_opacity(node, 0.06);
        for child in &mut node.children {
            match child.id.as_str() {
                "crest_gradient_overlay" => {
                    child.visuals.border_color = Some("#39526408".to_string());
                }
                "crest_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#50697B02".to_string());
                }
                "crest_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#4A627402".to_string());
                }
                _ => {}
            }
        }
    }

    if let Some(node) = find_bui_node_mut(root, "image_layer_pseudo_before") {
        node.visuals.background_color = None;
        scale_helper_child_opacity(node, 0.028);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_before_gradient_overlay" => {
                    child.visuals.background_color = Some("#C7EBFA0E".to_string());
                    child.styles.width = Some("16%".to_string());
                    child.styles.height = Some("16%".to_string());
                    child.styles.left = Some("9%".to_string());
                    child.styles.top = Some("4%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#C0E6F406".to_string());
                    child.styles.width = Some("40%".to_string());
                    child.styles.height = Some("36%".to_string());
                    child.styles.left = Some("-8%".to_string());
                    child.styles.top = Some("-6%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#BFE2F202".to_string());
                    child.styles.width = Some("92%".to_string());
                    child.styles.height = Some("82%".to_string());
                    child.styles.left = Some("-22%".to_string());
                    child.styles.top = Some("-18%".to_string());
                    child.styles.right = None;
                    child.styles.bottom = None;
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#BCD1DB00".to_string());
                    child.styles.left = Some("0%".to_string());
                    child.styles.right = Some("50%".to_string());
                    child.styles.top = Some("10%".to_string());
                    child.styles.bottom = Some("20%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_5" => {
                    child.visuals.background_color = Some("#8DA7B101".to_string());
                    child.styles.left = Some("18%".to_string());
                    child.styles.right = Some("42%".to_string());
                    child.styles.top = Some("28%".to_string());
                    child.styles.bottom = Some("40%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_6" => {
                    child.visuals.background_color = Some("#9BA38F01".to_string());
                    child.styles.left = Some("28%".to_string());
                    child.styles.right = Some("34%".to_string());
                    child.styles.top = Some("60%".to_string());
                    child.styles.bottom = Some("16%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_7" => {
                    child.visuals.background_color = Some("#55647401".to_string());
                    child.styles.left = Some("42%".to_string());
                    child.styles.right = Some("26%".to_string());
                    child.styles.top = Some("18%".to_string());
                    child.styles.bottom = Some("28%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_8" => {
                    child.visuals.background_color = Some("#2E3A4701".to_string());
                    child.styles.left = Some("54%".to_string());
                    child.styles.right = Some("22%".to_string());
                    child.styles.top = Some("16%".to_string());
                    child.styles.bottom = Some("32%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_9" => {
                    child.visuals.background_color = Some("#32314201".to_string());
                    child.styles.left = Some("66%".to_string());
                    child.styles.right = Some("20%".to_string());
                    child.styles.top = Some("18%".to_string());
                    child.styles.bottom = Some("30%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_10" => {
                    child.visuals.background_color = Some("#302E3E01".to_string());
                    child.styles.left = Some("70%".to_string());
                    child.styles.right = Some("16%".to_string());
                    child.styles.top = Some("16%".to_string());
                    child.styles.bottom = Some("34%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_11" => {
                    child.visuals.background_color = Some("#2D2B3B01".to_string());
                    child.styles.left = Some("74%".to_string());
                    child.styles.right = Some("14%".to_string());
                    child.styles.top = Some("14%".to_string());
                    child.styles.bottom = Some("38%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                "image_layer_pseudo_before_gradient_overlay_12" => {
                    child.visuals.background_color = Some("#2A283701".to_string());
                    child.styles.left = Some("78%".to_string());
                    child.styles.right = Some("12%".to_string());
                    child.styles.top = Some("12%".to_string());
                    child.styles.bottom = Some("42%".to_string());
                    child.visuals.border_radius = Some("50%".to_string());
                }
                _ => {}
            }
        }
    }

    if let Some(node) = find_bui_node_mut(root, "image_layer_pseudo_after") {
        node.visuals.background_color = None;
        scale_helper_child_opacity(node, 0.06);
        for child in &mut node.children {
            match child.id.as_str() {
                "image_layer_pseudo_after_gradient_overlay" => {
                    child.visuals.background_color = Some("#C39C5602".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#BE965002".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#B98F4702".to_string());
                }
                "image_layer_pseudo_after_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#160C1001".to_string());
                }
                _ => {}
            }
        }
    }

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

    if let Some(node) = find_bui_node_mut(root, "backbutton") {
        node.visuals.background_color = Some("#5A4D4342".to_string());
        node.visuals.border_color = Some("#B8944F72".to_string());
        node.visuals.border_width = Some("1px".to_string());
        if let Some(box_shadow) = node.visuals.box_shadow.as_mut() {
            box_shadow.blur_radius = Some("8px".to_string());
            box_shadow.color = Some("#14050614".to_string());
        }

        for child in &mut node.children {
            match child.id.as_str() {
                "backbutton_box_shadow_layer_1" => {
                    if let Some(box_shadow) = child.visuals.box_shadow.as_mut() {
                        box_shadow.color = Some("#C4BFBF14".to_string());
                    }
                }
                "backbutton_box_shadow_layer_2" => {
                    if let Some(box_shadow) = child.visuals.box_shadow.as_mut() {
                        box_shadow.blur_radius = Some("8px".to_string());
                        box_shadow.color = Some("#180E1314".to_string());
                    }
                }
                "backbutton_gradient_overlay" => {
                    child.visuals.background_color = Some("#FFF4D012".to_string());
                }
                "backbutton_gradient_overlay_2" => {
                    child.visuals.background_color = Some("#FFF1CB10".to_string());
                }
                "backbutton_gradient_overlay_3" => {
                    child.visuals.background_color = Some("#FFF0CA0C".to_string());
                }
                "backbutton_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#FFF0C806".to_string());
                }
                _ => {}
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
                    child.styles.width = Some("38%".to_string());
                    child.styles.height = Some("38%".to_string());
                    child.styles.left = Some("73%".to_string());
                    child.styles.top = Some("14%".to_string());
                }
                "info_panel_gradient_overlay_4" => {
                    child.visuals.background_color = Some("#F0DEB207".to_string());
                    child.styles.width = Some("43%".to_string());
                    child.styles.height = Some("43%".to_string());
                    child.styles.left = Some("79%".to_string());
                    child.styles.top = Some("22%".to_string());
                }
                "info_panel_gradient_overlay_5" => child.visuals.background_color = Some("#F0DEB20B".to_string()),
                "info_panel_gradient_overlay_6" => {
                    child.visuals.background_color = Some("#A9856420".to_string());
                    child.styles.width = Some("56%".to_string());
                    child.styles.height = Some("56%".to_string());
                    child.styles.left = Some("73%".to_string());
                    child.styles.top = Some("18%".to_string());
                }
                "info_panel_gradient_overlay_7" => {
                    child.visuals.background_color = Some("#9D7D5E22".to_string());
                    child.styles.width = Some("46%".to_string());
                    child.styles.height = Some("46%".to_string());
                    child.styles.left = Some("78%".to_string());
                    child.styles.top = Some("24%".to_string());
                }
                "info_panel_gradient_overlay_8" => child.visuals.background_color = Some("#916F5624".to_string()),
                "info_panel_gradient_overlay_9" => child.visuals.background_color = Some("#F0DEB204".to_string()),
                "info_panel_gradient_overlay_10" => child.visuals.background_color = Some("#F0DEB205".to_string()),
                "info_panel_left_cut_1" => child.visuals.background_color = Some("#261F270E".to_string()),
                "info_panel_left_cut_2" => child.visuals.background_color = Some("#58493E10".to_string()),
                "info_panel_left_cut_3" => child.visuals.background_color = Some("#AC8D680C".to_string()),
                "info_panel_left_mask_soft" => child.visuals.background_color = Some("#D9B67E08".to_string()),
                "info_panel_left_inner_glow" => child.visuals.background_color = Some("#F3D8A208".to_string()),
                "info_panel_top_gloss" => {
                    child.visuals.background_color = Some("#FFF3D10C".to_string());
                    child.styles.width = Some("24%".to_string());
                    child.styles.height = Some("10%".to_string());
                }
                "info_panel_mid_warmth" => {
                    child.visuals.background_color = Some("#E6C08A06".to_string());
                    child.styles.left = Some("18%".to_string());
                    child.styles.top = Some("11%".to_string());
                    child.styles.bottom = Some("6%".to_string());
                }
                "info_panel_right_sheen" => child.visuals.background_color = Some("#F0D4A203".to_string()),
                "info_panel_right_hotspot" => {
                    child.visuals.background_color = Some("#FFF0C804".to_string());
                    child.styles.width = Some("10%".to_string());
                    child.styles.height = Some("6%".to_string());
                    child.styles.right = Some("7%".to_string());
                    child.styles.top = Some("5%".to_string());
                    if let Some(box_shadow) = child.visuals.box_shadow.as_mut() {
                        box_shadow.blur_radius = Some("8px".to_string());
                        box_shadow.spread_radius = Some("1px".to_string());
                        box_shadow.color = Some("#FFF1D004".to_string());
                    }
                }
                "info_panel_lower_ember" => {
                    child.visuals.background_color = Some("#7C4F3B04".to_string());
                    child.styles.left = Some("16%".to_string());
                    child.styles.right = Some("6%".to_string());
                    child.styles.height = Some("24%".to_string());
                }
                "info_panel_inner_band" => {
                    child.visuals.background_color = Some("#EED8B00B".to_string());
                    child.styles.left = Some("20%".to_string());
                    child.styles.right = Some("18%".to_string());
                    child.styles.top = Some("32%".to_string());
                }
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
            row.visuals.background_color = Some(match row_id {
                "hero_stat_row_1" => "#6D5A6218".to_string(),
                "hero_stat_row_2" => "#6B594F16".to_string(),
                "hero_stat_row_3" => "#69574B14".to_string(),
                _ => "#6D5A6210".to_string(),
            });
            for child in &mut row.children {
                if child.id == format!("{row_id}_sheen") {
                    child.visuals.background_color = Some(match row_id {
                        "hero_stat_row_1" => "#D7BF980E".to_string(),
                        "hero_stat_row_2" => "#D1B28A0B".to_string(),
                        "hero_stat_row_3" => "#C9A77908".to_string(),
                        _ => "#C9A77904".to_string(),
                    });
                }

                match child.id.as_str() {
                    "hero_stat_label_1"
                    | "hero_stat_label_2"
                    | "hero_stat_label_3"
                    | "hero_stat_label_4"
                    | "hero_stat_label_5" => {
                        if let Some(text) = child.children.iter_mut().find(|grandchild| grandchild.id.starts_with("hero_stat_label_text_"))
                            && let Some(text_config) = text.text_config.as_mut()
                        {
                            text_config.font_color = "#F0E3CC".to_string();
                        }
                    }
                    "hero_stat_bonus_1"
                    | "hero_stat_bonus_2"
                    | "hero_stat_bonus_3"
                    | "hero_stat_bonus_4"
                    | "hero_stat_bonus_5" => {
                        if let Some(text) = child.children.iter_mut().find(|grandchild| grandchild.id.starts_with("hero_stat_bonus_text_"))
                            && let Some(text_config) = text.text_config.as_mut()
                        {
                            text_config.font_color = "#A8D256".to_string();
                        }
                    }
                    _ => {}
                }
            }
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

    for meter_id in ["meter", "meter_2"] {
        if let Some(node) = find_bui_node_mut(root, meter_id) {
            node.visuals.background_color = Some("#4A342796".to_string());
            if let Some(box_shadow) = node.visuals.box_shadow.as_mut() {
                box_shadow.color = Some("#F4DFC426".to_string());
            }

            for child in &mut node.children {
                match child.id.as_str() {
                    "meter_gradient_overlay" | "meter_2_gradient_overlay" => {
                        child.visuals.background_color = Some("#4A3326AE".to_string());
                    }
                    "meter_gradient_overlay_2" | "meter_2_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#452F23A8".to_string());
                    }
                    "meter_gradient_overlay_3" | "meter_2_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#402C219F".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    for meter_id in ["xpfill", "energyfill"] {
        if let Some(node) = find_bui_node_mut(root, meter_id) {
            node.visuals.background_color = Some(if meter_id == "xpfill" {
                "#9ED760".to_string()
            } else {
                "#F0C55A".to_string()
            });
            for child in &mut node.children {
                match child.id.as_str() {
                    "xpfill_gradient_overlay" => {
                        child.visuals.background_color = Some("#FFFFFF30".to_string());
                    }
                    "energyfill_gradient_overlay" => {
                        child.visuals.background_color = Some("#FFF9DD30".to_string());
                    }
                    "xpfill_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#FFFFFF1E".to_string());
                    }
                    "energyfill_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#FFF5CF1E".to_string());
                    }
                    "xpfill_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#9ED760".to_string());
                    }
                    "xpfill_gradient_overlay_4" => {
                        child.visuals.background_color = Some("#A9DC73".to_string());
                    }
                    "xpfill_gradient_overlay_5" => {
                        child.visuals.background_color = Some("#B4E085".to_string());
                    }
                    "xpfill_gradient_overlay_6" => {
                        child.visuals.background_color = Some("#BFE499".to_string());
                    }
                    "xpfill_gradient_overlay_7" => {
                        child.visuals.background_color = Some("#C8E7AA".to_string());
                    }
                    "xpfill_gradient_overlay_8" => {
                        child.visuals.background_color = Some("#D1EABC".to_string());
                    }
                    "xpfill_gradient_overlay_9" => {
                        child.visuals.background_color = Some("#D8EDC9".to_string());
                    }
                    "energyfill_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#F0C55A".to_string());
                    }
                    "energyfill_gradient_overlay_4" => {
                        child.visuals.background_color = Some("#F2CD6E".to_string());
                    }
                    "energyfill_gradient_overlay_5" => {
                        child.visuals.background_color = Some("#F4D37F".to_string());
                    }
                    "energyfill_gradient_overlay_6" => {
                        child.visuals.background_color = Some("#F6DA92".to_string());
                    }
                    "energyfill_gradient_overlay_7" => {
                        child.visuals.background_color = Some("#F7E1A5".to_string());
                    }
                    "energyfill_gradient_overlay_8" => {
                        child.visuals.background_color = Some("#F8E7B8".to_string());
                    }
                    "energyfill_gradient_overlay_9" => {
                        child.visuals.background_color = Some("#F9ECC7".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    for text_id in ["xptext_text_1", "energytext_text_1", "b_text_1", "b_2_text_1"] {
        if let Some(text) = find_bui_node_mut(root, text_id)
            && let Some(text_config) = text.text_config.as_mut()
        {
            text_config.font_color = "#F5EBD6".to_string();
            if let Some(shadow) = &mut text_config.text_shadow {
                shadow.color = Some("#2B180FA6".to_string());
            }
        }
    }

    for skill_id in ["skill_button", "skill_button_2", "skill_button_3"] {
        if let Some(node) = find_bui_node_mut(root, skill_id) {
            node.visuals.background_color = Some("#5A4A43D8".to_string());
            node.visuals.border_color = Some("#D7C4A796".to_string());
            for child in &mut node.children {
                match child.id.as_str() {
                    "skill_button_gradient_overlay"
                    | "skill_button_2_gradient_overlay"
                    | "skill_button_3_gradient_overlay" => {
                        child.visuals.background_color = Some("#FFF6D206".to_string());
                    }
                    "skill_button_gradient_overlay_2"
                    | "skill_button_2_gradient_overlay_2"
                    | "skill_button_3_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#FFF6D208".to_string());
                    }
                    "skill_button_gradient_overlay_3"
                    | "skill_button_2_gradient_overlay_3"
                    | "skill_button_3_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#FFF6D20A".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    for equip_id in ["equip_slot", "equip_slot_2", "equip_slot_3", "equip_slot_4", "equip_slot_5"] {
        if let Some(node) = find_bui_node_mut(root, equip_id) {
            if node.id == "equip_slot" {
                node.visuals.background_color = Some("#6B57448A".to_string());
            } else {
                node.visuals.background_color = Some("#6C5B4C7C".to_string());
            }

            for child in &mut node.children {
                match child.id.as_str() {
                    "equip_slot_gradient_overlay"
                    | "equip_slot_2_gradient_overlay"
                    | "equip_slot_3_gradient_overlay"
                    | "equip_slot_4_gradient_overlay"
                    | "equip_slot_5_gradient_overlay" => {
                        child.visuals.background_color = Some("#2B211E24".to_string());
                    }
                    "equip_slot_gradient_overlay_2"
                    | "equip_slot_2_gradient_overlay_2"
                    | "equip_slot_3_gradient_overlay_2"
                    | "equip_slot_4_gradient_overlay_2"
                    | "equip_slot_5_gradient_overlay_2" => {
                        child.visuals.background_color = Some("#FFF8DE05".to_string());
                    }
                    "equip_slot_gradient_overlay_3"
                    | "equip_slot_2_gradient_overlay_3"
                    | "equip_slot_3_gradient_overlay_3"
                    | "equip_slot_4_gradient_overlay_3"
                    | "equip_slot_5_gradient_overlay_3" => {
                        child.visuals.background_color = Some("#FFF8DE07".to_string());
                    }
                    "equip_slot_gradient_overlay_4"
                    | "equip_slot_2_gradient_overlay_4"
                    | "equip_slot_3_gradient_overlay_4"
                    | "equip_slot_4_gradient_overlay_4"
                    | "equip_slot_5_gradient_overlay_4" => {
                        child.visuals.background_color = Some("#FFF8DE09".to_string());
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
