use crate::core::{model::BuiNode, support::tree::find_bui_node_mut};

use super::bands::ensure_hero_panel_band;

pub(super) fn soften_info_panel(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "info_panel") {
        node.style.visuals.background_color = Some("#B7936F72".to_string());
        if let Some(box_shadow) = node.style.visuals.box_shadow.as_mut() {
            box_shadow.offset_x = Some("-18px".to_string());
            box_shadow.blur_radius = Some("56px".to_string());
            box_shadow.color = Some("#E6CAA62A".to_string());
        }

        for child in &mut node.children {
            match child.id.as_str() {
                "info_panel_gradient_overlay" => {
                    child.style.visuals.background_color = Some("#8C6C513C".to_string())
                }
                "info_panel_gradient_overlay_2" => {
                    child.style.visuals.background_color = Some("#A47E5B20".to_string())
                }
                "info_panel_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#F0DEB20A".to_string());
                    child.layout.styles.width = Some("34%".to_string());
                    child.layout.styles.height = Some("34%".to_string());
                    child.layout.styles.left = Some("75%".to_string());
                    child.layout.styles.top = Some("12%".to_string());
                }
                "info_panel_gradient_overlay_4" => {
                    child.style.visuals.background_color = Some("#F0DEB20A".to_string());
                    child.layout.styles.width = Some("38%".to_string());
                    child.layout.styles.height = Some("38%".to_string());
                    child.layout.styles.left = Some("82%".to_string());
                    child.layout.styles.top = Some("20%".to_string());
                }
                "info_panel_gradient_overlay_5" => {
                    child.style.visuals.background_color = Some("#F0DEB212".to_string())
                }
                "info_panel_gradient_overlay_6" => {
                    child.style.visuals.background_color = Some("#9C765218".to_string());
                    child.layout.styles.width = Some("48%".to_string());
                    child.layout.styles.height = Some("48%".to_string());
                    child.layout.styles.left = Some("76%".to_string());
                    child.layout.styles.top = Some("22%".to_string());
                }
                "info_panel_gradient_overlay_7" => {
                    child.style.visuals.background_color = Some("#8E6B5120".to_string());
                    child.layout.styles.width = Some("42%".to_string());
                    child.layout.styles.height = Some("42%".to_string());
                    child.layout.styles.left = Some("80%".to_string());
                    child.layout.styles.top = Some("26%".to_string());
                }
                "info_panel_gradient_overlay_8" => {
                    child.style.visuals.background_color = Some("#865D4624".to_string())
                }
                "info_panel_gradient_overlay_9" => {
                    child.style.visuals.background_color = Some("#EFD9AA08".to_string())
                }
                "info_panel_gradient_overlay_10" => {
                    child.style.visuals.background_color = Some("#E8CF9C09".to_string())
                }
                "info_panel_left_cut_1" => {
                    child.style.visuals.background_color = Some("#2A212815".to_string())
                }
                "info_panel_left_cut_2" => {
                    child.style.visuals.background_color = Some("#5B473C18".to_string())
                }
                "info_panel_left_cut_3" => {
                    child.style.visuals.background_color = Some("#A4836112".to_string())
                }
                "info_panel_left_mask_soft" => {
                    child.style.visuals.background_color = Some("#D7B27A0C".to_string())
                }
                "info_panel_left_inner_glow" => {
                    child.style.visuals.background_color = Some("#E9CB9708".to_string())
                }
                "info_panel_top_gloss" => {
                    child.style.visuals.background_color = Some("#FFF0CC08".to_string());
                    child.layout.styles.width = Some("18%".to_string());
                    child.layout.styles.height = Some("8%".to_string());
                }
                "info_panel_mid_warmth" => {
                    child.style.visuals.background_color = Some("#E0B98308".to_string());
                    child.layout.styles.left = Some("16%".to_string());
                    child.layout.styles.top = Some("10%".to_string());
                    child.layout.styles.bottom = Some("4%".to_string());
                }
                "info_panel_right_sheen" => {
                    child.style.visuals.background_color = Some("#F0D4A106".to_string())
                }
                "info_panel_right_hotspot" => {
                    child.style.visuals.background_color = Some("#FFF0CA09".to_string());
                    child.layout.styles.width = Some("8%".to_string());
                    child.layout.styles.height = Some("5%".to_string());
                    child.layout.styles.right = Some("8%".to_string());
                    child.layout.styles.top = Some("4%".to_string());
                    if let Some(box_shadow) = child.style.visuals.box_shadow.as_mut() {
                        box_shadow.blur_radius = Some("10px".to_string());
                        box_shadow.spread_radius = Some("2px".to_string());
                        box_shadow.color = Some("#FFF1D008".to_string());
                    }
                }
                "info_panel_lower_ember" => {
                    child.style.visuals.background_color = Some("#7B4B3707".to_string());
                    child.layout.styles.left = Some("14%".to_string());
                    child.layout.styles.right = Some("4%".to_string());
                    child.layout.styles.height = Some("22%".to_string());
                }
                "info_panel_inner_band" => {
                    child.style.visuals.background_color = Some("#EED7AF08".to_string());
                    child.layout.styles.left = Some("18%".to_string());
                    child.layout.styles.right = Some("14%".to_string());
                    child.layout.styles.top = Some("30%".to_string());
                }
                _ => {}
            }
        }

        ensure_hero_panel_band(
            node,
            "info_panel_bottom_veil_1",
            "0",
            "0",
            "36%",
            "0",
            "#47362B08",
            "13",
        );
        ensure_hero_panel_band(
            node,
            "info_panel_bottom_veil_2",
            "0",
            "0",
            "48%",
            "0",
            "#47362B0D",
            "14",
        );
        ensure_hero_panel_band(
            node,
            "info_panel_bottom_veil_3",
            "0",
            "0",
            "60%",
            "0",
            "#47362B12",
            "15",
        );
        ensure_hero_panel_band(
            node,
            "info_panel_bottom_veil_4",
            "0",
            "0",
            "72%",
            "0",
            "#47362B18",
            "16",
        );
    }
}
