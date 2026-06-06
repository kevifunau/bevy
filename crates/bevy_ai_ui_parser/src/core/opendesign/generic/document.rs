use std::collections::HashMap;

use crate::core::{
    model::BuiDocument,
    opendesign::{
        dom::has_class,
        hero::enhance_hero_game_ui_defaults,
        preset::{apply_opendesign_preset, OpenDesignPreset},
        stylesheet::OpenDesignStylesheet,
    },
    parse::validate::validate_bui_document,
};

use super::tree::{generic_append_children, generic_element_node};

pub(crate) fn opendesign_html_to_generic_bui_document(
    stylesheet: &OpenDesignStylesheet,
    overlay: roxmltree::Node<'_, '_>,
) -> Result<BuiDocument, String> {
    let mut id_counts = HashMap::new();
    let mut root = generic_element_node("overlay_root", "node", stylesheet, overlay);
    apply_opendesign_preset(
        &mut root,
        if has_class(overlay, "game-stage") {
            OpenDesignPreset::GameStageRoot
        } else {
            OpenDesignPreset::OverlayRoot
        },
    );
    generic_append_children(&mut root, overlay, stylesheet, &mut id_counts);
    enhance_hero_game_ui_defaults(&mut root);

    let document = BuiDocument {
        version: "3.0-ir".to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        imports: Vec::new(),
        state_model: crate::core::model::BuiStateModel::default(),
        resources: crate::core::model::BuiResources::default(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}