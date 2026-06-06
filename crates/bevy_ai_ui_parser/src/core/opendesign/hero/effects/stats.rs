use crate::core::{
    model::BuiNode,
    support::tree::find_bui_node_mut,
};

pub(super) fn soften_stat_rows(root: &mut BuiNode) {
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
}