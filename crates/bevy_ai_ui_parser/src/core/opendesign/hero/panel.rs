use crate::core::model::{bui_node, BuiBoxShadowConfig, BuiNode};

pub(super) fn inject_hero_info_panel_layers(info_panel: &mut BuiNode) {
    if info_panel
        .children
        .iter()
        .any(|child| child.id == "info_panel_mid_warmth")
    {
        return;
    }

    let mut left_cut_1 = bui_node("info_panel_left_cut_1", "node");
    left_cut_1.markers.push("hero-info-panel:decor".to_string());
    left_cut_1.layout.styles.position_type = Some("absolute".to_string());
    left_cut_1.layout.styles.left = Some("0".to_string());
    left_cut_1.layout.styles.top = Some("0".to_string());
    left_cut_1.layout.styles.bottom = Some("0".to_string());
    left_cut_1.layout.styles.width = Some("3%".to_string());
    left_cut_1.layout.styles.z_index = Some("-1".to_string());
    left_cut_1.style.visuals.background_color = Some("#2D25301E".to_string());

    let mut left_cut_2 = bui_node("info_panel_left_cut_2", "node");
    left_cut_2.markers.push("hero-info-panel:decor".to_string());
    left_cut_2.layout.styles.position_type = Some("absolute".to_string());
    left_cut_2.layout.styles.left = Some("3%".to_string());
    left_cut_2.layout.styles.top = Some("0".to_string());
    left_cut_2.layout.styles.bottom = Some("0".to_string());
    left_cut_2.layout.styles.width = Some("4%".to_string());
    left_cut_2.layout.styles.z_index = Some("-1".to_string());
    left_cut_2.style.visuals.background_color = Some("#5D4D4318".to_string());

    let mut left_cut_3 = bui_node("info_panel_left_cut_3", "node");
    left_cut_3.markers.push("hero-info-panel:decor".to_string());
    left_cut_3.layout.styles.position_type = Some("absolute".to_string());
    left_cut_3.layout.styles.left = Some("7%".to_string());
    left_cut_3.layout.styles.top = Some("0".to_string());
    left_cut_3.layout.styles.bottom = Some("0".to_string());
    left_cut_3.layout.styles.width = Some("5%".to_string());
    left_cut_3.layout.styles.z_index = Some("-1".to_string());
    left_cut_3.style.visuals.background_color = Some("#A88A6512".to_string());

    let mut left_mask_soft = bui_node("info_panel_left_mask_soft", "node");
    left_mask_soft
        .markers
        .push("hero-info-panel:decor".to_string());
    left_mask_soft.layout.styles.position_type = Some("absolute".to_string());
    left_mask_soft.layout.styles.left = Some("12%".to_string());
    left_mask_soft.layout.styles.top = Some("0".to_string());
    left_mask_soft.layout.styles.bottom = Some("0".to_string());
    left_mask_soft.layout.styles.width = Some("8%".to_string());
    left_mask_soft.layout.styles.z_index = Some("-1".to_string());
    left_mask_soft.style.visuals.background_color = Some("#D7B47B0A".to_string());

    let mut top_gloss = bui_node("info_panel_top_gloss", "node");
    top_gloss.markers.push("hero-info-panel:decor".to_string());
    top_gloss.layout.styles.position_type = Some("absolute".to_string());
    top_gloss.layout.styles.right = Some("0".to_string());
    top_gloss.layout.styles.top = Some("0".to_string());
    top_gloss.layout.styles.width = Some("28%".to_string());
    top_gloss.layout.styles.height = Some("12%".to_string());
    top_gloss.layout.styles.z_index = Some("-1".to_string());
    top_gloss.style.visuals.background_color = Some("#FFF3D10E".to_string());
    top_gloss.style.visuals.border_radius = Some("999px".to_string());

    let mut left_inner_glow = bui_node("info_panel_left_inner_glow", "node");
    left_inner_glow
        .markers
        .push("hero-info-panel:decor".to_string());
    left_inner_glow.layout.styles.position_type = Some("absolute".to_string());
    left_inner_glow.layout.styles.left = Some("15%".to_string());
    left_inner_glow.layout.styles.top = Some("0".to_string());
    left_inner_glow.layout.styles.bottom = Some("0".to_string());
    left_inner_glow.layout.styles.width = Some("7%".to_string());
    left_inner_glow.layout.styles.z_index = Some("-1".to_string());
    left_inner_glow.style.visuals.background_color = Some("#F5D8A40C".to_string());

    let mut mid_warmth = bui_node("info_panel_mid_warmth", "node");
    mid_warmth.markers.push("hero-info-panel:decor".to_string());
    mid_warmth.layout.styles.position_type = Some("absolute".to_string());
    mid_warmth.layout.styles.left = Some("18%".to_string());
    mid_warmth.layout.styles.right = Some("0".to_string());
    mid_warmth.layout.styles.top = Some("8%".to_string());
    mid_warmth.layout.styles.bottom = Some("0".to_string());
    mid_warmth.layout.styles.z_index = Some("-1".to_string());
    mid_warmth.style.visuals.background_color = Some("#E5C18A0D".to_string());

    let mut right_hotspot = bui_node("info_panel_right_hotspot", "node");
    right_hotspot
        .markers
        .push("hero-info-panel:decor".to_string());
    right_hotspot.layout.styles.position_type = Some("absolute".to_string());
    right_hotspot.layout.styles.right = Some("5%".to_string());
    right_hotspot.layout.styles.top = Some("5%".to_string());
    right_hotspot.layout.styles.width = Some("12%".to_string());
    right_hotspot.layout.styles.height = Some("7%".to_string());
    right_hotspot.layout.styles.z_index = Some("-1".to_string());
    right_hotspot.style.visuals.background_color = Some("#FFF0C808".to_string());
    right_hotspot.style.visuals.border_radius = Some("999px".to_string());
    right_hotspot.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("14px".to_string()),
        spread_radius: Some("4px".to_string()),
        color: Some("#FFF1D008".to_string()),
    });

    let mut right_sheen = bui_node("info_panel_right_sheen", "node");
    right_sheen
        .markers
        .push("hero-info-panel:decor".to_string());
    right_sheen.layout.styles.position_type = Some("absolute".to_string());
    right_sheen.layout.styles.right = Some("0".to_string());
    right_sheen.layout.styles.top = Some("0".to_string());
    right_sheen.layout.styles.bottom = Some("0".to_string());
    right_sheen.layout.styles.width = Some("10%".to_string());
    right_sheen.layout.styles.z_index = Some("-1".to_string());
    right_sheen.style.visuals.background_color = Some("#F0D4A206".to_string());

    let mut lower_ember = bui_node("info_panel_lower_ember", "node");
    lower_ember
        .markers
        .push("hero-info-panel:decor".to_string());
    lower_ember.layout.styles.position_type = Some("absolute".to_string());
    lower_ember.layout.styles.left = Some("18%".to_string());
    lower_ember.layout.styles.right = Some("4%".to_string());
    lower_ember.layout.styles.bottom = Some("0".to_string());
    lower_ember.layout.styles.height = Some("18%".to_string());
    lower_ember.layout.styles.z_index = Some("-1".to_string());
    lower_ember.style.visuals.background_color = Some("#7C4F3B08".to_string());

    let mut inner_band = bui_node("info_panel_inner_band", "node");
    inner_band.markers.push("hero-info-panel:decor".to_string());
    inner_band.layout.styles.position_type = Some("absolute".to_string());
    inner_band.layout.styles.left = Some("24%".to_string());
    inner_band.layout.styles.right = Some("14%".to_string());
    inner_band.layout.styles.top = Some("30%".to_string());
    inner_band.layout.styles.height = Some("1px".to_string());
    inner_band.layout.styles.z_index = Some("-1".to_string());
    inner_band.style.visuals.background_color = Some("#FFF0D608".to_string());

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
