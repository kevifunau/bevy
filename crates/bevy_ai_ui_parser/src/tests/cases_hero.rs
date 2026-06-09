use super::shared::*;
use crate::core::opendesign::html::opendesign_html_to_bui_document;
use crate::core::parse::ir::parse_bui_document;

#[test]
fn hero_game_ui_html_compiles_to_bui_document() {
    let document = opendesign_html_to_bui_document(HERO_GAME_UI_HTML)
        .expect("hero game UI HTML should compile");

    let root = find_bui_node(&document.root, "overlay_root");
    assert_eq!(
        root.style.visuals.background_color.as_deref(),
        Some("#47362B")
    );
    assert_eq!(root.layout.styles.width.as_deref(), Some("1680px"));
    assert_eq!(root.layout.styles.height.as_deref(), None);
    assert_eq!(
        root.layout.styles.aspect_ratio.as_deref(),
        Some("2.1383765")
    );

    let page_title = find_bui_node(&document.root, "page_title_text_1");
    assert_eq!(
        page_title
            .content
            .text
            .as_ref()
            .map(|text| text.content.as_str()),
        Some("英雄")
    );

    let hero_name = find_bui_node(&document.root, "hero_name_text_1");
    assert_eq!(
        hero_name
            .content
            .text
            .as_ref()
            .map(|text| text.content.as_str()),
        Some("Olympia")
    );
}

#[test]
fn hero_game_ui_html_and_ir_entry_paths_produce_identical_bui_documents() {
    let from_html =
        opendesign_html_to_bui_document(HERO_GAME_UI_HTML).expect("HTML should compile");
    let from_ir = parse_bui_document(HERO_GAME_UI_IR).expect("3.0-ir JSON should parse");

    let html_value = serde_json::to_value(&from_html).expect("HTML document should serialize");
    let ir_value = serde_json::to_value(&from_ir).expect("IR document should serialize");

    assert_eq!(
        html_value, ir_value,
        "HTML and IR entry paths should produce identical BuiDocuments"
    );
}
