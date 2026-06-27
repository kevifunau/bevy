use super::shared::*;
use crate::core::opendesign::html::opendesign_html_to_bui_document;

#[test]
fn opendesign_media_query_rules_resolve_into_bui_styles() {
    let document =
        opendesign_html_to_bui_document(VILLAGE_SHOP_HTML).expect("OpenDesign HTML should compile");

    let panel = find_bui_node(&document.root, "panel");
    assert_eq!(panel.layout.styles.width.as_deref(), Some("720px"));

    let card = find_bui_node(&document.root, "shop_card_hut");
    assert_eq!(
        card.layout.styles.grid_template_columns.as_deref(),
        Some("flex(1) px(140)")
    );

    let item_main = find_bui_node(&document.root, "item_main_hut");
    assert_eq!(
        item_main.layout.styles.grid_template_columns.as_deref(),
        Some("px(104) flex(1)")
    );
}

#[test]
fn opendesign_active_transform_compiles_to_pressed_state_scale() {
    let document =
        opendesign_html_to_bui_document(VILLAGE_SHOP_HTML).expect("OpenDesign HTML should compile");

    let buy_button = find_bui_node(&document.root, "buy_btn_hut");
    assert_eq!(
        buy_button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("0.95 0.95")
    );
    assert_eq!(
        buy_button
            .state_visuals
            .get("normal")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("1 1")
    );

    let close_button = find_bui_node(&document.root, "close_btn");
    assert_eq!(
        close_button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("0.95 0.95")
    );
}

#[test]
fn opendesign_text_nodes_do_not_inherit_button_transform_styles() {
    let document =
        opendesign_html_to_bui_document(VILLAGE_SHOP_HTML).expect("OpenDesign HTML should compile");

    let buy_text = find_bui_node(&document.root, "buy_btn_hut_text");
    for (_, state) in &buy_text.state_visuals {
        assert!(
            state.styles.ui_scale.is_none(),
            "text node should not inherit transform (ui_scale) from parent button"
        );
    }
    assert_eq!(
        buy_text
            .content
            .text
            .as_ref()
            .map(|config| config.font_color.as_str()),
        Some("#ffffff")
    );
}

#[test]
fn unsupported_pseudo_element_selectors_do_not_leak_into_node_styles() {
    let document =
        opendesign_html_to_bui_document(VILLAGE_SHOP_HTML).expect("OpenDesign HTML should compile");

    let scroll = find_bui_node(&document.root, "shop_scroll");
    assert_eq!(
        scroll.style.visuals.background_color, None,
        "::-webkit-scrollbar-thumb background should not leak into shop_scroll"
    );
    assert_eq!(
        scroll.style.visuals.border_radius, None,
        "::-webkit-scrollbar-thumb border-radius should not leak into shop_scroll"
    );
    assert_eq!(
        scroll.layout.styles.width, None,
        "::-webkit-scrollbar width should not leak into shop_scroll"
    );
}
