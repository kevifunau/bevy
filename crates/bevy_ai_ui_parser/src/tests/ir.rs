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

#[test]
fn opendesign_json_list_declarations_compile_to_ir_semantics() {
    let html = r#"
    <main class="game-stage">
      <div id="right_content"
           data-bui-list="server.servers"
           data-bui-json-src="Asset/ServerInfo.json"
           data-bui-json-mode="page"
           data-bui-page-size="5"
           data-bui-page-source="server.region">
        <button id="server_item_{{id}}" data-action="server.selectServer">
          <span>{{id}}区  {{name}}</span>
        </button>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let list = find_bui_node(&document.root, "right_content");

    assert_eq!(
        list.semantics.list_binding_source.as_deref(),
        Some("server.servers")
    );
    assert_eq!(
        list.semantics.list_json_source.as_deref(),
        Some("Asset/ServerInfo.json")
    );
    assert_eq!(list.semantics.list_json_mode.as_deref(), Some("page"));
    assert_eq!(list.semantics.list_page_size, Some(5));
    assert_eq!(
        list.semantics.list_page_source.as_deref(),
        Some("server.region")
    );
    assert_eq!(list.children.len(), 1);
    assert_eq!(list.children[0].id, "server_item_{{id}}");
}
