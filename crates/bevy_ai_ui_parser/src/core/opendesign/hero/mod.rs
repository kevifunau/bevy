mod controls;
mod effects;
mod panel;
mod stats;

use crate::core::{
    model::{BuiBoxShadowConfig, BuiNode, BuiNodeType, BuiStateVisual, BuiStyles, BuiVisuals},
    opendesign::svg::{ensure_text_icon_child, is_decorative_icon_helper_node},
    support::tree::{find_bui_node_mut, find_bui_node_ref},
};

use controls::style_hero_game_ui_controls;
use effects::soften_hero_game_ui_effect_fallbacks;
use panel::inject_hero_info_panel_layers;
use stats::{hero_game_ui_base_stats, hero_game_ui_stat_row};

pub(crate) fn enhance_hero_game_ui_defaults(root: &mut BuiNode) {
    let is_hero_game_ui = root.custom_tags.iter().any(|tag| tag == "class:game-stage")
        && find_bui_node_ref(root, "hero_zone").is_some()
        && find_bui_node_ref(root, "info_panel").is_some()
        && find_bui_node_ref(root, "name_card").is_some();
    if !is_hero_game_ui {
        return;
    }

    if let Some(stars) = find_bui_node_mut(root, "stars")
        && stars.children.iter().all(is_decorative_icon_helper_node)
    {
        stars
            .children
            .retain(|child| !child.custom_tags.iter().any(|tag| tag == "svg:fallback"));
        if stars.children.is_empty() {
            for index in 0..5 {
                stars.children.push(crate::core::opendesign::build::text_node(
                    &format!("hero_star_text_{}", index + 1),
                    "★",
                    42.0,
                    "#F5C742",
                    Some("Hiragino Sans GB.ttc"),
                ));
            }
        }
    }

    if let Some(stats_list) = find_bui_node_mut(root, "statslist")
        && stats_list.children.is_empty()
    {
        for (index, (icon, label, base, bonus)) in hero_game_ui_base_stats().iter().enumerate() {
            stats_list
                .children
                .push(hero_game_ui_stat_row(index + 1, icon, label, base, bonus));
        }
    }

    if let Some(panel_section) = find_bui_node_mut(root, "panel_section") {
        panel_section.styles.display = Some("grid".to_string());
        panel_section.styles.row_gap = Some("18px".to_string());
    }

    if let Some(panel_section) = find_bui_node_mut(root, "panel_section_2") {
        panel_section.styles.display = Some("grid".to_string());
        panel_section.styles.row_gap = Some("14px".to_string());
    }

    if let Some(stats_list) = find_bui_node_mut(root, "statslist") {
        stats_list.styles.display = Some("grid".to_string());
        stats_list.styles.row_gap = Some("6px".to_string());
    }

    if let Some(crest) = find_bui_node_mut(root, "crest") {
        crest.visuals.background_color = Some("#2E394316".to_string());
        crest.visuals.border_color = Some("#51617030".to_string());
        crest.visuals.border_width = Some("2px".to_string());
        crest.visuals.border_radius = Some("50%".to_string());
    }

    if let Some(overlay_root) = find_bui_node_mut(root, "overlay_root") {
        overlay_root.visuals.background_color = Some("#47362B".to_string());
    }

    if let Some(hero_glow) = find_bui_node_mut(root, "hero_glow") {
        hero_glow.visuals.border_radius = Some("50%".to_string());
    }

    if let Some(hero_cutout) = find_bui_node_mut(root, "hero_cutout")
        && hero_cutout.visuals.border_radius.is_none()
    {
        hero_cutout.visuals.border_radius = Some("96px".to_string());
    }

    if let Some(info_panel) = find_bui_node_mut(root, "info_panel") {
        inject_hero_info_panel_layers(info_panel);
        info_panel.styles.top = Some("23.8%".to_string());
        info_panel.styles.right = Some("4.2%".to_string());
        info_panel.styles.bottom = Some("6.2%".to_string());
        info_panel.styles.width = Some("38.5%".to_string());
        info_panel.visuals.background_color = Some("#C6A784B2".to_string());
        info_panel.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("-14px".to_string()),
            offset_y: Some("0px".to_string()),
            blur_radius: Some("54px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#E2CAA130".to_string()),
        });
    }

    soften_hero_game_ui_effect_fallbacks(root);
    style_hero_game_ui_controls(root);

    for hidden_overlay_id in ["popover", "toast"] {
        if let Some(node) = find_bui_node_mut(root, hidden_overlay_id) {
            node.styles.visibility = Some("hidden".to_string());
        }
    }

    for meter_label_id in ["b", "b_2"] {
        if let Some(meter_label) = find_bui_node_mut(root, meter_label_id) {
            meter_label.styles.display = Some("flex".to_string());
            meter_label.styles.align_items = Some("center".to_string());
            meter_label.styles.justify_content = Some("flex-end".to_string());
            meter_label.styles.column_gap = Some("0".to_string());
        }
    }

    for semantic_icon_id in [
        "backbutton",
        "bar_icon",
        "bar_icon_2",
        "skill_button",
        "skill_button_2",
        "skill_button_3",
        "equip_slot",
        "equip_slot_2",
        "equip_slot_3",
        "equip_slot_4",
        "equip_slot_5",
    ] {
        ensure_text_icon_child(root, semantic_icon_id);
    }
}

pub(super) fn ensure_state_visual<'a>(node: &'a mut BuiNode, state: &str) -> &'a mut BuiStateVisual {
    node.state_visuals
        .entry(state.to_string())
        .or_insert_with(|| BuiStateVisual {
            styles: BuiStyles::default(),
            visuals: BuiVisuals::default(),
            text_color: None,
        })
}

pub(super) fn first_direct_text_child_mut(node: &mut BuiNode) -> Option<&mut BuiNode> {
    node.children
        .iter_mut()
        .find(|child| matches!(child.node_type, BuiNodeType::Text))
}
