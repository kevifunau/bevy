use crate::core::{
    model::{BuiNode, BuiNodeType},
    opendesign::build::{bui_node, text_node},
};

pub(super) fn hero_game_ui_base_stats() -> [(&'static str, &'static str, &'static str, &'static str); 5] {
    [
        ("⚔", "武力", "136.28", "+18"),
        ("♞", "统帅", "136.2", "+186"),
        ("✦", "智谋", "136.3", "+86"),
        ("⚡", "速度", "28.66", "+210"),
        ("⌂", "政务", "206.2", "+186"),
    ]
}

pub(super) fn hero_game_ui_stat_row(
    index: usize,
    icon: &str,
    label: &str,
    base: &str,
    bonus: &str,
) -> BuiNode {
    let mut row = bui_node(&format!("hero_stat_row_{index}"), BuiNodeType::Node);
    row.custom_tags.push("class:stat-row".to_string());
    row.styles.display = Some("grid".to_string());
    row.styles.grid_template_columns = Some("flex(1) auto auto".to_string());
    row.styles.align_items = Some("center".to_string());
    row.styles.column_gap = Some("10px".to_string());
    row.styles.padding = Some("0 8px".to_string());
    row.styles.min_height = Some("40px".to_string());
    row.visuals.background_color = Some("#6D5A6333".to_string());

    let mut label_node = bui_node(&format!("hero_stat_label_{index}"), BuiNodeType::Node);
    label_node.custom_tags.push("class:stat-label".to_string());
    label_node.styles.display = Some("flex".to_string());
    label_node.styles.align_items = Some("center".to_string());
    label_node.styles.column_gap = Some("11px".to_string());
    label_node.styles.min_width = Some("0".to_string());
    label_node.children.push(text_node(
        &format!("hero_stat_icon_text_{index}"),
        icon,
        22.0,
        "#E9DDC8",
        Some("Apple Symbols.ttf"),
    ));
    label_node.children.push(text_node(
        &format!("hero_stat_label_text_{index}"),
        label,
        24.0,
        "#F0E7D8",
        Some("Hiragino Sans GB.ttc"),
    ));

    let mut base_node = bui_node(&format!("hero_stat_base_{index}"), BuiNodeType::Node);
    base_node.custom_tags.push("class:stat-base".to_string());
    base_node.styles.display = Some("flex".to_string());
    base_node.styles.justify_content = Some("flex-end".to_string());
    base_node.styles.align_items = Some("center".to_string());
    base_node.children.push(text_node(
        &format!("hero_stat_base_text_{index}"),
        base,
        24.0,
        "#F6EBDD",
        Some("Palatino.ttc"),
    ));

    let mut bonus_node = bui_node(&format!("hero_stat_bonus_{index}"), BuiNodeType::Node);
    bonus_node.custom_tags.push("class:stat-bonus".to_string());
    bonus_node.styles.display = Some("flex".to_string());
    bonus_node.styles.justify_content = Some("flex-end".to_string());
    bonus_node.styles.align_items = Some("center".to_string());
    bonus_node.children.push(text_node(
        &format!("hero_stat_bonus_text_{index}"),
        bonus,
        24.0,
        "#B7DD6D",
        Some("Palatino.ttc"),
    ));

    row.children.push(label_node);
    row.children.push(base_node);
    row.children.push(bonus_node);
    row
}
