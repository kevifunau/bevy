use crate::core::{
    model::BuiNode,
    support::tree::find_bui_node_mut,
};

use super::bands::ensure_hero_panel_band;

pub(super) fn soften_info_panel(root: &mut BuiNode) {
    if let Some(node) = find_bui_node_mut(root, "info_panel") {
        node.style.visuals.background_color = Some("#BE9D7A86".to_string());
        if let Some(box_shadow) = node.style.visuals.box_shadow.as_mut() {
            box_shadow.offset_x = Some("-10px".to_string());
            box_shadow.blur_radius = Some("42px".to_string());
            box_shadow.color = Some("#E7CFAB18".to_string());
        }

        for child in &mut node.children {
            match child.id.as_str() {
                "info_panel_gradient_overlay" => child.style.visuals.background_color = Some("#BD9B7234".to_string()),
                "info_panel_gradient_overlay_2" => child.style.visuals.background_color = Some("#A27F5D28".to_string()),
                "info_panel_gradient_overlay_3" => {
                    child.style.visuals.background_color = Some("#F0DEB206".to_string());
                    child.layout.styles.width = Some("38%".to_string());
                    child.layout.styles.height = Some("38%".to_string());
                    child.layout.styles.left = Some("73%".to_string());
                    child.layout.styles.top = Some("14%".to_string());
                }
                "info_panel_gradient_overlay_4" => {
                    child.style.visuals.background_color = Some("#F0DEB207".to_string());
                    child.layout.styles.width = Some("43%".to_string());
                    child.layout.styles.height = Some("43%".to_string());
                    child.layout.styles.left = Some("79%".to_string());
                    child.layout.styles.top = Some("22%".to_string());
                }
                "info_panel_gradient_overlay_5" => child.style.visuals.background_color = Some("#F0DEB20B".to_string()),
                "info_panel_gradient_overlay_6" => {
                    child.style.visuals.background_color = Some("#A9856420".to_string());
                    child.layout.styles.width = Some("56%".to_string());
                    child.layout.styles.height = Some("56%".to_string());
                    child.layout.styles.left = Some("73%".to_string());
                    child.layout.styles.top = Some("18%".to_string());
                }
                "info_panel_gradient_overlay_7" => {
                    child.style.visuals.background_color = Some("#9D7D5E22".to_string());
                    child.layout.styles.width = Some("46%".to_string());
                    child.layout.styles.height = Some("46%".to_string());
                    child.layout.styles.left = Some("78%".to_string());
                    child.layout.styles.top = Some("24%".to_string());
                }
                "info_panel_gradient_overlay_8" => child.style.visuals.background_color = Some("#916F5624".to_string()),
                "info_panel_gradient_overlay_9" => child.style.visuals.background_color = Some("#F0DEB204".to_string()),
                "info_panel_gradient_overlay_10" => child.style.visuals.background_color = Some("#F0DEB205".to_string()),
                "info_panel_left_cut_1" => child.style.visuals.background_color = Some("#261F270E".to_string()),
                "info_panel_left_cut_2" => child.style.visuals.background_color = Some("#58493E10".to_string()),
                "info_panel_left_cut_3" => child.style.visuals.background_color = Some("#AC8D680C".to_string()),
                "info_panel_left_mask_soft" => child.style.visuals.background_color = Some("#D9B67E08".to_string()),
                "info_panel_left_inner_glow" => child.style.visuals.background_color = Some("#F3D8A208".to_string()),
                "info_panel_top_gloss" => {
                    child.style.visuals.background_color = Some("#FFF3D10C".to_string());
                    child.layout.styles.width = Some("24%".to_string());
                    child.layout.styles.height = Some("10%".to_string());
                }
                "info_panel_mid_warmth" => {
                    child.style.visuals.background_color = Some("#E6C08A06".to_string());
                    child.layout.styles.left = Some("18%".to_string());
                    child.layout.styles.top = Some("11%".to_string());
                    child.layout.styles.bottom = Some("6%".to_string());
                }
                "info_panel_right_sheen" => child.style.visuals.background_color = Some("#F0D4A203".to_string()),
                "info_panel_right_hotspot" => {
                    child.style.visuals.background_color = Some("#FFF0C804".to_string());
                    child.layout.styles.width = Some("10%".to_string());
                    child.layout.styles.height = Some("6%".to_string());
                    child.layout.styles.right = Some("7%".to_string());
                    child.layout.styles.top = Some("5%".to_string());
                    if let Some(box_shadow) = child.style.visuals.box_shadow.as_mut() {
                        box_shadow.blur_radius = Some("8px".to_string());
                        box_shadow.spread_radius = Some("1px".to_string());
                        box_shadow.color = Some("#FFF1D004".to_string());
                    }
                }
                "info_panel_lower_ember" => {
                    child.style.visuals.background_color = Some("#7C4F3B04".to_string());
                    child.layout.styles.left = Some("16%".to_string());
                    child.layout.styles.right = Some("6%".to_string());
                    child.layout.styles.height = Some("24%".to_string());
                }
                "info_panel_inner_band" => {
                    child.style.visuals.background_color = Some("#EED8B00B".to_string());
                    child.layout.styles.left = Some("20%".to_string());
                    child.layout.styles.right = Some("18%".to_string());
                    child.layout.styles.top = Some("32%".to_string());
                }
                _ => {}
            }
        }

        ensure_hero_panel_band(node, "info_panel_bottom_veil_1", "0", "0", "36%", "0", "#47362B0A", "13");
        ensure_hero_panel_band(node, "info_panel_bottom_veil_2", "0", "0", "48%", "0", "#47362B10", "14");
        ensure_hero_panel_band(node, "info_panel_bottom_veil_3", "0", "0", "60%", "0", "#47362B16", "15");
        ensure_hero_panel_band(node, "info_panel_bottom_veil_4", "0", "0", "72%", "0", "#47362B20", "16");
    }
}