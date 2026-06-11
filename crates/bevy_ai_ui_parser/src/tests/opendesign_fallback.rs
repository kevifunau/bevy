use super::shared::*;
use crate::{opendesign_html_to_bui_json_str, validate_bui_json_str};
use crate::core::opendesign::html::opendesign_html_to_bui_document_with_manifest;

#[test]
fn opendesign_html_without_manifest_preserves_html_only_path() {
    let document = opendesign_html_to_bui_document_with_manifest(
        BEVY_UI_TEXT_BASELINE_HTML,
        None,
        None,
    )
    .expect("html-only OpenDesign UI should compile without a manifest");

    let launch_btn = find_bui_node(&document.root, "launch_btn");
    assert!(
        launch_btn.content.image.is_none(),
        "html-only fallback should not inject manifest-backed images"
    );
    assert!(
        launch_btn.state_visuals.contains_key("hovered"),
        "css hover state should still compile on the legacy path"
    );

    let shield_stat = find_bui_node(&document.root, "shield_stat");
    assert!(
        shield_stat.content.image.is_none(),
        "plain card nodes should remain asset-free on html-only projects"
    );
}

#[test]
fn opendesign_html_only_json_entrypoint_still_validates() {
    let json = opendesign_html_to_bui_json_str(BEVY_UI_TEXT_BASELINE_HTML)
        .expect("html-only OpenDesign baseline should compile to JSON");
    validate_bui_json_str(&json).expect("html-only OpenDesign baseline JSON should validate");
}
