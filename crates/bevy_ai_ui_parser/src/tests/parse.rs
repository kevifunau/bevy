use super::shared::*;
use crate::{opendesign_html_to_bui_ir_json_str, validate_bui_ir_json_str};
use crate::core::model::BuiNodeType;
use crate::core::parse::validate::EXPECTED_VERSION;
use crate::core::parse::document::parse_bui_document;

#[test]
fn opendesign_ir_snapshot_can_load_through_runtime_parser() {
    let ir_json = opendesign_html_to_bui_ir_json_str(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile to IR");
    let document = parse_bui_document(&ir_json).expect("BUI IR should parse for runtime");

    assert_eq!(document.version, EXPECTED_VERSION);

    let panel = find_bui_node(&document.root, "panel");
    assert_eq!(panel.styles.max_height.as_deref(), Some("648px"));

    let buy_button = find_bui_node(&document.root, "buy_btn_hut");
    assert!(matches!(buy_button.node_type, BuiNodeType::Button));
    assert_eq!(
        buy_button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("0.95 0.95")
    );

    validate_bui_ir_json_str(&ir_json).expect("BUI IR validator should accept generated IR");
}
