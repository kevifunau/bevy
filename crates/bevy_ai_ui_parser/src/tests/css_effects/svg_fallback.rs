use super::*;

#[test]
fn semantic_svg_fallback_uses_parent_id_before_path_signature() {
    let mut parent = bui_node("skill_button_2", "button");
    parent.markers = vec!["class:skill-button".to_string()];

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "♛");
}

#[test]
fn semantic_svg_fallback_uses_data_skill_tags() {
    let mut parent = bui_node("skill_button_dynamic", "button");
    parent.markers = vec!["class:skill-button".to_string(), "data-skill:军团号令".to_string()];

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "♛");
}

#[test]
fn semantic_svg_fallback_uses_data_equip_tags() {
    let mut parent = bui_node("equip_slot_dynamic", "button");
    parent.markers = vec!["class:equip-slot".to_string(), "data-equip:鹰眼徽章".to_string()];

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "◎");
}

#[test]
fn semantic_svg_fallback_supports_indexed_id_patterns() {
    let mut parent = bui_node("equip_slot_4", "button");
    parent.markers = vec!["class:equip-slot".to_string()];

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "♞");
}

#[test]
fn svg_shape_fallback_recognizes_crosshair_badge_icons() {
    let mut parent = bui_node("equip_slot_dynamic", "button");
    parent.markers = vec!["class:equip-slot".to_string()];

    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="14" fill="none" stroke="currentColor" stroke-width="3"/><path d="M20 8v24M8 20h24M15 20a5 5 0 0 0 10 0 5 5 0 0 0-10 0Z" fill="none" stroke="currentColor" stroke-width="3"/></svg>"#,
    )
    .expect("svg should parse");

    let icon = svg_fallback_icon(&parent, svg.root_element()).expect("shape fallback should exist");
    assert_eq!(icon, "◎");
}

#[test]
fn svg_shape_fallback_recognizes_shield_icons() {
    let mut parent = bui_node("equip_slot_dynamic", "button");
    parent.markers = vec!["class:equip-slot".to_string()];

    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40"><path d="M20 4 32 9v9c0 8-5 14-12 18C13 32 8 26 8 18V9Z" fill="none" stroke="currentColor" stroke-width="3"/><path d="M14 14h12M14 20h12" stroke="currentColor" stroke-width="3"/></svg>"#,
    )
    .expect("svg should parse");

    let icon = svg_fallback_icon(&parent, svg.root_element()).expect("shape fallback should exist");
    assert_eq!(icon, "⛨");
}

#[test]
fn svg_shape_fallback_recognizes_scroll_skill_icons() {
    let mut parent = bui_node("skill_button_dynamic", "button");
    parent.markers = vec!["class:skill-button".to_string()];

    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 36 36"><path d="M8 6h17c2 0 4 2 4 4v20H11c-2 0-4-2-4-4V7c0-.6.4-1 1-1Zm5 6v3h11v-3Zm0 6v3h9v-3Z" fill="currentColor"/></svg>"#,
    )
    .expect("svg should parse");

    let icon = svg_fallback_icon(&parent, svg.root_element()).expect("shape fallback should exist");
    assert_eq!(icon, "▤");
}