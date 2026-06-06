use super::shared::*;
use crate::{
    opendesign_html_to_bui_json_str,
    validate_bui_json_str,
};
use crate::core::opendesign::html::opendesign_html_to_bui_document;

#[test]
fn generic_opendesign_overlay_compiles_without_shop_structure() {
    let document = opendesign_html_to_bui_document(QUEST_NOTICE_HTML)
        .expect("generic OpenDesign overlay should compile");

    let title = find_bui_node(&document.root, "notice_title_text_1");
    assert_eq!(
        title.content.text.as_ref().map(|text| text.content.as_str()),
        Some("新的委托")
    );

    let accept = find_bui_node(&document.root, "primary_btn");
    assert!(accept.kind == "button");
    assert_eq!(
        accept
            .actions
            .first()
            .map(|action| (action.event.as_str(), action.emit.as_str())),
        Some(("press", "accept_quest"))
    );
    assert_eq!(
        accept
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("0.96 0.96")
    );

    let bui_json = opendesign_html_to_bui_json_str(QUEST_NOTICE_HTML)
        .expect("generic OpenDesign overlay should compile to BUI JSON");
    validate_bui_json_str(&bui_json).expect("generic BUI JSON should validate");
}