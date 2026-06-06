use crate::core::{
    model::{BuiBoxShadowConfig, BuiNode, BuiNodeType},
    opendesign::build::bui_node,
};

pub(super) fn inject_hero_info_panel_layers(info_panel: &mut BuiNode) {
    if info_panel
        .children
        .iter()
        .any(|child| child.id == "info_panel_mid_warmth")
    {
        return;
    }

    let mut left_cut_1 = bui_node("info_panel_left_cut_1", BuiNodeType::Node);
    left_cut_1.custom_tags.push("hero-info-panel:decor".to_string());
    left_cut_1.styles.position_type = Some("absolute".to_string());
    left_cut_1.styles.left = Some("0".to_string());
    left_cut_1.styles.top = Some("0".to_string());
    left_cut_1.styles.bottom = Some("0".to_string());
    left_cut_1.styles.width = Some("3%".to_string());
    left_cut_1.styles.z_index = Some("-1".to_string());
    left_cut_1.visuals.background_color = Some("#2D25301E".to_string());

    let mut left_cut_2 = bui_node("info_panel_left_cut_2", BuiNodeType::Node);
    left_cut_2.custom_tags.push("hero-info-panel:decor".to_string());
    left_cut_2.styles.position_type = Some("absolute".to_string());
    left_cut_2.styles.left = Some("3%".to_string());
    left_cut_2.styles.top = Some("0".to_string());
    left_cut_2.styles.bottom = Some("0".to_string());
    left_cut_2.styles.width = Some("4%".to_string());
    left_cut_2.styles.z_index = Some("-1".to_string());
    left_cut_2.visuals.background_color = Some("#5D4D4318".to_string());

    let mut left_cut_3 = bui_node("info_panel_left_cut_3", BuiNodeType::Node);
    left_cut_3.custom_tags.push("hero-info-panel:decor".to_string());
    left_cut_3.styles.position_type = Some("absolute".to_string());
    left_cut_3.styles.left = Some("7%".to_string());
    left_cut_3.styles.top = Some("0".to_string());
    left_cut_3.styles.bottom = Some("0".to_string());
    left_cut_3.styles.width = Some("5%".to_string());
    left_cut_3.styles.z_index = Some("-1".to_string());
    left_cut_3.visuals.background_color = Some("#A88A6512".to_string());

    let mut left_mask_soft = bui_node("info_panel_left_mask_soft", BuiNodeType::Node);
    left_mask_soft.custom_tags.push("hero-info-panel:decor".to_string());
    left_mask_soft.styles.position_type = Some("absolute".to_string());
    left_mask_soft.styles.left = Some("12%".to_string());
    left_mask_soft.styles.top = Some("0".to_string());
    left_mask_soft.styles.bottom = Some("0".to_string());
    left_mask_soft.styles.width = Some("8%".to_string());
    left_mask_soft.styles.z_index = Some("-1".to_string());
    left_mask_soft.visuals.background_color = Some("#D7B47B0A".to_string());

    let mut top_gloss = bui_node("info_panel_top_gloss", BuiNodeType::Node);
    top_gloss.custom_tags.push("hero-info-panel:decor".to_string());
    top_gloss.styles.position_type = Some("absolute".to_string());
    top_gloss.styles.right = Some("0".to_string());
    top_gloss.styles.top = Some("0".to_string());
    top_gloss.styles.width = Some("28%".to_string());
    top_gloss.styles.height = Some("12%".to_string());
    top_gloss.styles.z_index = Some("-1".to_string());
    top_gloss.visuals.background_color = Some("#FFF3D10E".to_string());
    top_gloss.visuals.border_radius = Some("999px".to_string());

    let mut left_inner_glow = bui_node("info_panel_left_inner_glow", BuiNodeType::Node);
    left_inner_glow.custom_tags.push("hero-info-panel:decor".to_string());
    left_inner_glow.styles.position_type = Some("absolute".to_string());
    left_inner_glow.styles.left = Some("15%".to_string());
    left_inner_glow.styles.top = Some("0".to_string());
    left_inner_glow.styles.bottom = Some("0".to_string());
    left_inner_glow.styles.width = Some("7%".to_string());
    left_inner_glow.styles.z_index = Some("-1".to_string());
    left_inner_glow.visuals.background_color = Some("#F5D8A40C".to_string());

    let mut mid_warmth = bui_node("info_panel_mid_warmth", BuiNodeType::Node);
    mid_warmth.custom_tags.push("hero-info-panel:decor".to_string());
    mid_warmth.styles.position_type = Some("absolute".to_string());
    mid_warmth.styles.left = Some("18%".to_string());
    mid_warmth.styles.right = Some("0".to_string());
    mid_warmth.styles.top = Some("8%".to_string());
    mid_warmth.styles.bottom = Some("0".to_string());
    mid_warmth.styles.z_index = Some("-1".to_string());
    mid_warmth.visuals.background_color = Some("#E5C18A0D".to_string());

    let mut right_hotspot = bui_node("info_panel_right_hotspot", BuiNodeType::Node);
    right_hotspot.custom_tags.push("hero-info-panel:decor".to_string());
    right_hotspot.styles.position_type = Some("absolute".to_string());
    right_hotspot.styles.right = Some("5%".to_string());
    right_hotspot.styles.top = Some("5%".to_string());
    right_hotspot.styles.width = Some("12%".to_string());
    right_hotspot.styles.height = Some("7%".to_string());
    right_hotspot.styles.z_index = Some("-1".to_string());
    right_hotspot.visuals.background_color = Some("#FFF0C808".to_string());
    right_hotspot.visuals.border_radius = Some("999px".to_string());
    right_hotspot.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("14px".to_string()),
        spread_radius: Some("4px".to_string()),
        color: Some("#FFF1D008".to_string()),
    });

    let mut right_sheen = bui_node("info_panel_right_sheen", BuiNodeType::Node);
    right_sheen.custom_tags.push("hero-info-panel:decor".to_string());
    right_sheen.styles.position_type = Some("absolute".to_string());
    right_sheen.styles.right = Some("0".to_string());
    right_sheen.styles.top = Some("0".to_string());
    right_sheen.styles.bottom = Some("0".to_string());
    right_sheen.styles.width = Some("10%".to_string());
    right_sheen.styles.z_index = Some("-1".to_string());
    right_sheen.visuals.background_color = Some("#F0D4A206".to_string());

    let mut lower_ember = bui_node("info_panel_lower_ember", BuiNodeType::Node);
    lower_ember.custom_tags.push("hero-info-panel:decor".to_string());
    lower_ember.styles.position_type = Some("absolute".to_string());
    lower_ember.styles.left = Some("18%".to_string());
    lower_ember.styles.right = Some("4%".to_string());
    lower_ember.styles.bottom = Some("0".to_string());
    lower_ember.styles.height = Some("18%".to_string());
    lower_ember.styles.z_index = Some("-1".to_string());
    lower_ember.visuals.background_color = Some("#7C4F3B08".to_string());

    let mut inner_band = bui_node("info_panel_inner_band", BuiNodeType::Node);
    inner_band.custom_tags.push("hero-info-panel:decor".to_string());
    inner_band.styles.position_type = Some("absolute".to_string());
    inner_band.styles.left = Some("24%".to_string());
    inner_band.styles.right = Some("14%".to_string());
    inner_band.styles.top = Some("30%".to_string());
    inner_band.styles.height = Some("1px".to_string());
    inner_band.styles.z_index = Some("-1".to_string());
    inner_band.visuals.background_color = Some("#FFF0D608".to_string());

    info_panel.children.insert(0, right_hotspot);
    info_panel.children.insert(0, right_sheen);
    info_panel.children.insert(0, lower_ember);
    info_panel.children.insert(0, inner_band);
    info_panel.children.insert(0, mid_warmth);
    info_panel.children.insert(0, top_gloss);
    info_panel.children.insert(0, left_mask_soft);
    info_panel.children.insert(0, left_inner_glow);
    info_panel.children.insert(0, left_cut_3);
    info_panel.children.insert(0, left_cut_2);
    info_panel.children.insert(0, left_cut_1);
}
