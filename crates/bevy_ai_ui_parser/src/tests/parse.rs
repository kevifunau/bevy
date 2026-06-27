use super::shared::*;
use crate::core::parse::ir::parse_bui_document;
use crate::{opendesign_html_to_bui_json_str, validate_bui_json_str};

#[test]
fn opendesign_ir_snapshot_can_load_through_runtime_parser() {
    let bui_json = opendesign_html_to_bui_json_str(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile to BUI JSON");
    let document = parse_bui_document(&bui_json).expect("BUI JSON should parse for runtime");

    assert_eq!(document.version, "3.0-ir");

    let panel = find_bui_node(&document.root, "panel");
    assert_eq!(panel.layout.styles.max_height.as_deref(), Some("648px"));

    let buy_button = find_bui_node(&document.root, "buy_btn_hut");
    assert!(buy_button.kind == "button");
    assert_eq!(
        buy_button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("0.95 0.95")
    );

    validate_bui_json_str(&bui_json).expect("BUI validator should accept generated BUI JSON");
}
