use super::shared::*;
use crate::core::opendesign::html::opendesign_html_to_bui_document;
use crate::core::parse::ir::parse_bui_document;

#[test]
fn opendesign_ir_export_uses_3_0_shape() {
    let ir =
        opendesign_html_to_bui_document(VILLAGE_SHOP_HTML).expect("OpenDesign HTML should compile");

    assert_eq!(ir.version, "3.0-ir");
    assert_eq!(ir.root.kind, "node");

    let panel = ir
        .root
        .children
        .iter()
        .find(|child| child.id == "panel")
        .expect("panel should exist");
    assert_eq!(panel.layout.styles.max_width.as_deref(), Some("720px"));

    let buy_button = find_bui_node(&ir.root, "buy_btn_hut");
    assert_eq!(buy_button.kind, "button");
    assert!(buy_button.content.is_empty());
    assert!(buy_button
        .state_visuals
        .get("pressed")
        .and_then(|state| state.styles.ui_scale.as_deref())
        .is_some());
}

#[test]
fn checked_in_ir_fixture_loads_through_runtime_parser() {
    let document = parse_bui_document(VILLAGE_SHOP_IR).expect("checked-in IR should parse");

    let root = find_bui_node(&document.root, "overlay_root");
    assert_eq!(root.layout.styles.height.as_deref(), Some("100%"));

    let close_button = find_bui_node(&document.root, "close_btn");
    assert!(close_button.kind == "button");
    assert_eq!(
        close_button
            .actions
            .first()
            .map(|action| (action.event.as_str(), action.emit.as_str())),
        Some(("press", "close_shop_overlay"))
    );
}
