use crate::core::{
    model::{bui_node, ensure_state_visual, BuiBoxShadowConfig, BuiNode},
    opendesign::hero::first_direct_text_child_mut,
    support::tree::find_bui_node_mut,
};

pub(super) fn style_hero_game_ui_controls(root: &mut BuiNode) {
    style_hero_tab_button(root, "tab_button", true);
    style_hero_tab_button(root, "tab_button_2", false);
    style_hero_tab_button(root, "tab_button_3", false);
    style_hero_section_title(root, "section_title");
    style_hero_section_title(root, "section_title_2");

    style_hero_action_button(root, "detailsbutton", false);
    style_hero_action_button(root, "upgradebutton", true);
    style_hero_mobile_toggle(root, "paneltoggle");

    style_hero_equip_slot(root, "equip_slot", true);
    style_hero_equip_slot(root, "equip_slot_2", false);
    style_hero_equip_slot(root, "equip_slot_3", false);
    style_hero_equip_slot(root, "equip_slot_4", false);
    style_hero_equip_slot(root, "equip_slot_5", false);

    for row_id in [
        "hero_stat_row_1",
        "hero_stat_row_2",
        "hero_stat_row_3",
        "hero_stat_row_4",
        "hero_stat_row_5",
    ] {
        style_hero_stat_row(root, row_id);
    }
}

fn style_hero_section_title(root: &mut BuiNode, id: &str) {
    let Some(title) = find_bui_node_mut(root, id) else {
        return;
    };

    for child in &mut title.children {
        match child.id.as_str() {
            "section_title_border_bottom" | "section_title_2_border_bottom" => {
                child.style.visuals.background_color = Some("#8B674F2B".to_string());
            }
            "section_title_text_1" | "section_title_2_text_1" => {
                if let Some(text_config) = child.content.text.as_mut() {
                    text_config.font_color = "#5C3113".to_string();
                    if let Some(shadow) = &mut text_config.text_shadow {
                        shadow.color = Some("#F6E2C233".to_string());
                    }
                }
            }
            _ => {}
        }
    }
}

fn style_hero_tab_button(root: &mut BuiNode, id: &str, selected: bool) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.semantics.tab_group_name = Some("hero_stats".to_string());
    button.semantics.tab_value = Some(
        match id {
            "tab_button" => "base",
            "tab_button_2" => "battle",
            "tab_button_3" => "march",
            _ => "unknown",
        }
        .to_string(),
    );

    button.layout.styles.position_type = Some("relative".to_string());
    button.layout.styles.min_height = Some("38px".to_string());
    button.layout.styles.padding = Some("0 14px".to_string());
    button.style.visuals.border_width = Some("1px".to_string());
    button.style.visuals.border_radius = Some("3px".to_string());

    if selected {
        button.style.visuals.background_color = Some("#E7D4A7B8".to_string());
        button.style.visuals.border_color = Some("#E3D1A082".to_string());
        button.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("1px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#FFF9E052".to_string()),
        });
        ensure_state_visual(button, "normal")
            .visuals
            .background_color = Some("#BAA88A36".to_string());
        ensure_state_visual(button, "normal").visuals.border_color = Some("#6B564132".to_string());
        ensure_state_visual(button, "selected")
            .visuals
            .background_color = Some("#E7D4A7B8".to_string());
        ensure_state_visual(button, "selected").visuals.border_color =
            Some("#E3D1A082".to_string());
        ensure_state_visual(button, "selected").visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("1px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#FFF9E052".to_string()),
        });
    } else {
        button.style.visuals.background_color = Some("#BAA88A36".to_string());
        button.style.visuals.border_color = Some("#6B564132".to_string());
        button.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("1px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#20161D14".to_string()),
        });
        ensure_state_visual(button, "normal")
            .visuals
            .background_color = Some("#BAA88A36".to_string());
        ensure_state_visual(button, "normal").visuals.border_color = Some("#6B564132".to_string());
        ensure_state_visual(button, "selected")
            .visuals
            .background_color = Some("#E7D4A7B8".to_string());
        ensure_state_visual(button, "selected").visuals.border_color =
            Some("#E3D1A082".to_string());
    }

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.content.text.as_mut()
    {
        text_config.font_color = if selected {
            "#2D1A1D".to_string()
        } else {
            "#4B383F".to_string()
        };
    }
}

fn style_hero_action_button(root: &mut BuiNode, id: &str, primary: bool) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.layout.styles.position_type = Some("relative".to_string());
    button.layout.styles.min_height = Some("48px".to_string());
    button.layout.styles.padding = Some("0 22px".to_string());
    button.style.visuals.border_width = Some("1px".to_string());
    button.style.visuals.border_radius = Some("3px".to_string());

    if primary {
        button.style.visuals.background_color = Some("#E7D9A8F0".to_string());
        button.style.visuals.border_color = Some("#E8D59DCE".to_string());
        button.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("5px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#8A603580".to_string()),
        });
    } else {
        button.style.visuals.background_color = Some("#43343E88".to_string());
        button.style.visuals.border_color = Some("#E7D7B56B".to_string());
        button.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("0px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("1px".to_string()),
            color: Some("#2C1E251E".to_string()),
        });
    }

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.content.text.as_mut()
    {
        text_config.font_color = if primary {
            "#2B1719".to_string()
        } else {
            "#F3E8D5".to_string()
        };
    }
}

fn style_hero_mobile_toggle(root: &mut BuiNode, id: &str) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.layout.styles.display = Some("none".to_string());
    button.layout.styles.visibility = Some("hidden".to_string());
    button.layout.styles.ui_opacity = Some(0.0);
    button.style.visuals.background_color = Some("#E7D8A9EB".to_string());
    button.style.visuals.border_color = Some("#EDD89DAD".to_string());
    button.style.visuals.border_width = Some("1px".to_string());
    button.style.visuals.border_radius = Some("999px".to_string());
    button.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("8px".to_string()),
        blur_radius: Some("24px".to_string()),
        spread_radius: Some("0px".to_string()),
        color: Some("#1610185C".to_string()),
    });

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.content.text.as_mut()
    {
        text_config.font_color = "#2E1B1E".to_string();
    }
}

fn style_hero_equip_slot(root: &mut BuiNode, id: &str, selected: bool) {
    let Some(slot) = find_bui_node_mut(root, id) else {
        return;
    };

    slot.layout.styles.position_type = Some("relative".to_string());
    slot.style.visuals.background_color = Some("#64524594".to_string());
    slot.style.visuals.border_color = Some(if selected {
        "#F0D48AA8".to_string()
    } else {
        "#C9B59B66".to_string()
    });
    slot.style.visuals.border_width = Some("2px".to_string());
    slot.style.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some(if selected { "5px" } else { "3px" }.to_string()),
        blur_radius: Some("0px".to_string()),
        spread_radius: Some(if selected { "0px" } else { "0px" }.to_string()),
        color: Some(if selected {
            "#875C346C".to_string()
        } else {
            "#6A443650".to_string()
        }),
    });

    if let Some(text) = first_direct_text_child_mut(slot)
        && let Some(text_config) = text.content.text.as_mut()
    {
        text_config.font_color = if selected {
            "#F7E7C7".to_string()
        } else {
            "#F3E3C6".to_string()
        };
    }

    if selected && let Some(pseudo_after) = find_bui_node_mut(slot, &format!("{id}_pseudo_after")) {
        pseudo_after.style.visuals.border_color = Some("#F5E5C4A8".to_string());
        pseudo_after.style.visuals.border_width = Some("1px".to_string());
    }
}

fn style_hero_stat_row(root: &mut BuiNode, id: &str) {
    let Some(row) = find_bui_node_mut(root, id) else {
        return;
    };

    row.layout.styles.position_type = Some("relative".to_string());
    row.style.visuals.background_color = Some("#6D5A6218".to_string());

    if row
        .children
        .iter()
        .any(|child| child.id == format!("{id}_sheen"))
    {
        return;
    }

    let mut sheen = bui_node(&format!("{id}_sheen"), "node");
    sheen.markers.push("hero-stat-row:decor".to_string());
    sheen.layout.styles.position_type = Some("absolute".to_string());
    sheen.layout.styles.left = Some("42%".to_string());
    sheen.layout.styles.right = Some("0".to_string());
    sheen.layout.styles.top = Some("0".to_string());
    sheen.layout.styles.bottom = Some("0".to_string());
    sheen.layout.styles.z_index = Some("-1".to_string());
    sheen.style.visuals.background_color = Some("#D3B6900C".to_string());
    row.children.insert(0, sheen);
}
