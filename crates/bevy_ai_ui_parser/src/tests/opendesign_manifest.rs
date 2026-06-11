use super::shared::*;
use crate::{
    opendesign_html_to_bui_json_str_with_manifest, validate_bui_json_str,
};
use crate::core::opendesign::manifest::OpenDesignAssetManifest;
use crate::core::opendesign::{
    html::opendesign_html_to_bui_document_with_manifest,
    manifest::apply_manifest_to_document,
};

#[test]
fn opendesign_manifest_injects_images_slicer_atlas_and_state_textures() {
    let manifest: OpenDesignAssetManifest =
        serde_json::from_str(BEVY_UI_ASSET_FLOW_MANIFEST).expect("manifest should parse");
    let mut document = opendesign_html_to_bui_document_with_manifest(
        BEVY_UI_ASSET_FLOW_HTML,
        Some(&manifest),
        None,
    )
    .expect("manifest-driven OpenDesign HTML should compile");

    apply_manifest_to_document(&mut document, &manifest, None)
        .expect("in-memory manifest application should succeed");

    let panel = find_bui_node(&document.root, "settings_panel");
    let panel_image = panel
        .content
        .image
        .as_ref()
        .expect("settings_panel should receive an image");
    assert_eq!(
        panel_image.texture_path,
        "./assets/panels/settings_panel_bg.png"
    );
    assert_eq!(panel_image.image_mode.as_deref(), Some("sliced"));
    assert_eq!(
        panel_image.slicer.as_ref().map(|slicer| slicer.border),
        Some(22.0)
    );

    let close_btn = find_bui_node(&document.root, "close_btn");
    let hover_image = close_btn
        .state_visuals
        .get("hovered")
        .and_then(|state| state.image.as_ref())
        .expect("close_btn hovered state should have an image");
    assert_eq!(
        hover_image.texture_path,
        "./assets/buttons/close_hover.png"
    );

    let skill_icon = find_bui_node(&document.root, "skill_icon");
    assert_eq!(
        skill_icon
            .content
            .image
            .as_ref()
            .map(|image| image.texture_path.as_str()),
        Some("./assets/icons/fireball.png")
    );

    let atlas_icon = find_bui_node(&document.root, "loading_spinner");
    let atlas = atlas_icon
        .content
        .image
        .as_ref()
        .and_then(|image| image.atlas.as_ref())
        .expect("loading spinner should receive atlas config");
    assert_eq!(atlas.tile_width, 24);
    assert_eq!(atlas.columns, 7);
}

#[test]
fn opendesign_manifest_json_entrypoint_validates() {
    let json =
        opendesign_html_to_bui_json_str_with_manifest(BEVY_UI_ASSET_FLOW_HTML, BEVY_UI_ASSET_FLOW_MANIFEST)
            .expect("manifest-driven OpenDesign HTML should compile to JSON");
    validate_bui_json_str(&json).expect("manifest-driven BUI JSON should validate");
}
