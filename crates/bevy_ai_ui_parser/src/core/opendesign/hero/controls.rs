use crate::core::{
    model::{BuiBoxShadowConfig, BuiNode, BuiNodeType},
    opendesign::{
        build::bui_node,
        hero::{ensure_state_visual, first_direct_text_child_mut},
    },
    support::tree::find_bui_node_mut,
};

pub(super) fn style_hero_game_ui_controls(root: &mut BuiNode) {
    style_hero_tab_button(root, "tab_button", true);
    style_hero_tab_button(root, "tab_button_2", false);
    style_hero_tab_button(root, "tab_button_3", false);

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

fn style_hero_tab_button(root: &mut BuiNode, id: &str, selected: bool) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.styles.position_type = Some("relative".to_string());
    button.styles.min_height = Some("38px".to_string());
    button.styles.padding = Some("0 14px".to_string());
    button.visuals.border_width = Some("1px".to_string());
    button.visuals.border_radius = Some("3px".to_string());

    if selected {
        button.visuals.background_color = Some("#E7D4A7B8".to_string());
        button.visuals.border_color = Some("#E3D1A082".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
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
        button.visuals.background_color = Some("#BAA88A36".to_string());
        button.visuals.border_color = Some("#6B564132".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
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
        && let Some(text_config) = text.text_config.as_mut()
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

    button.styles.position_type = Some("relative".to_string());
    button.styles.min_height = Some("48px".to_string());
    button.styles.padding = Some("0 22px".to_string());
    button.visuals.border_width = Some("1px".to_string());
    button.visuals.border_radius = Some("3px".to_string());

    if primary {
        button.visuals.background_color = Some("#E7D9A8F0".to_string());
        button.visuals.border_color = Some("#E8D59DCE".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("5px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#8A603580".to_string()),
        });
    } else {
        button.visuals.background_color = Some("#43343E88".to_string());
        button.visuals.border_color = Some("#E7D7B56B".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("0px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("1px".to_string()),
            color: Some("#2C1E251E".to_string()),
        });
    }

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.text_config.as_mut()
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

    button.visuals.background_color = Some("#E7D8A9EB".to_string());
    button.visuals.border_color = Some("#EDD89DAD".to_string());
    button.visuals.border_width = Some("1px".to_string());
    button.visuals.border_radius = Some("999px".to_string());
    button.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("8px".to_string()),
        blur_radius: Some("24px".to_string()),
        spread_radius: Some("0px".to_string()),
        color: Some("#1610185C".to_string()),
    });

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = "#2E1B1E".to_string();
    }
}

fn style_hero_equip_slot(root: &mut BuiNode, id: &str, selected: bool) {
    let Some(slot) = find_bui_node_mut(root, id) else {
        return;
    };

    slot.styles.position_type = Some("relative".to_string());
    slot.visuals.background_color = Some("#5E566286".to_string());
    slot.visuals.border_color = Some(if selected {
        "#F0D48AA8".to_string()
    } else {
        "#C9B59B7A".to_string()
    });
    slot.visuals.border_width = Some("2px".to_string());
    slot.visuals.box_shadow = Some(BuiBoxShadowConfig {
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
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = if selected {
            "#F7E7C7".to_string()
        } else {
            "#F3E3C6".to_string()
        };
    }

    if selected && let Some(pseudo_after) = find_bui_node_mut(slot, &format!("{id}_pseudo_after")) {
        pseudo_after.visuals.border_color = Some("#F5E5C4A8".to_string());
        pseudo_after.visuals.border_width = Some("1px".to_string());
    }
}

fn style_hero_stat_row(root: &mut BuiNode, id: &str) {
    let Some(row) = find_bui_node_mut(root, id) else {
        return;
    };

    row.styles.position_type = Some("relative".to_string());
    row.visuals.background_color = Some("#6D5A6218".to_string());

    if row
        .children
        .iter()
        .any(|child| child.id == format!("{id}_sheen"))
    {
        return;
    }

    let mut sheen = bui_node(&format!("{id}_sheen"), BuiNodeType::Node);
    sheen.custom_tags.push("hero-stat-row:decor".to_string());
    sheen.styles.position_type = Some("absolute".to_string());
    sheen.styles.left = Some("42%".to_string());
    sheen.styles.right = Some("0".to_string());
    sheen.styles.top = Some("0".to_string());
    sheen.styles.bottom = Some("0".to_string());
    sheen.styles.z_index = Some("-1".to_string());
    sheen.visuals.background_color = Some("#D3B6900C".to_string());
    row.children.insert(0, sheen);
}
