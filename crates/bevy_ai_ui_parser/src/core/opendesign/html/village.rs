use crate::core::{
    model::{BuiActionBinding, BuiNode, BuiNodeType},
    opendesign::{
        build::{apply_opendesign_styles, bui_node, text_node},
        cases::{shop_card_node, stabilize_village_shop_overlay_defaults},
        dom::{first_text_by_class, has_class},
        preset::{OpenDesignPreset, apply_opendesign_preset},
        stylesheet::OpenDesignStylesheet,
    },
};

pub(super) fn compile_village_shop_overlay_document(
    stylesheet: &OpenDesignStylesheet,
    overlay: roxmltree::Node<'_, '_>,
) -> Result<BuiNode, String> {
    let panel_source = overlay
        .descendants()
        .find(|node| has_class(*node, "panel"))
        .ok_or_else(|| "OpenDesign HTML is missing a .panel node.".to_string())?;
    let panel_header_source = panel_source
        .descendants()
        .find(|node| has_class(*node, "panel-header"));
    let title_board_source = panel_header_source.and_then(|panel_header_source| {
        panel_header_source
            .descendants()
            .find(|node| has_class(*node, "title-board"))
    });
    let title_text_source = title_board_source.and_then(|title_board_source| {
        title_board_source
            .descendants()
            .find(|node| has_class(*node, "title-text"))
    });
    let close_button_source = panel_header_source.and_then(|panel_header_source| {
        panel_header_source
            .descendants()
            .find(|node| has_class(*node, "close-btn"))
    });
    let title_board_source = panel_header_source.and(title_board_source);
    let title_text_source = title_board_source.and(title_text_source);
    let shop_body_source = panel_source
        .descendants()
        .find(|node| has_class(*node, "shop-body"));
    let shop_scroll_source = shop_body_source.and_then(|shop_body_source| {
        shop_body_source
            .descendants()
            .find(|node| has_class(*node, "shop-scroll"))
    });
    let foot_hint_source = panel_source
        .descendants()
        .find(|node| has_class(*node, "foot-hint"));

    let (
        Some(panel_header_source),
        Some(title_board_source),
        Some(title_text_source),
        Some(close_button_source),
        Some(shop_body_source),
        Some(shop_scroll_source),
    ) = (
        panel_header_source,
        title_board_source,
        title_text_source,
        close_button_source,
        shop_body_source,
        shop_scroll_source,
    )
    else {
        return Err("overlay does not match village shop structure".to_string());
    };

    let title = first_text_by_class(overlay, "title-text").unwrap_or_else(|| "UI".to_string());
    let footer = first_text_by_class(overlay, "foot-hint").unwrap_or_default();

    let mut root = bui_node("overlay_root", BuiNodeType::Node);
    apply_opendesign_preset(&mut root, OpenDesignPreset::OverlayRoot);
    apply_opendesign_styles(stylesheet, &mut root, overlay);

    let mut panel = bui_node("panel", BuiNodeType::Node);
    apply_opendesign_preset(&mut panel, OpenDesignPreset::Panel);
    apply_opendesign_styles(stylesheet, &mut panel, panel_source);

    let mut panel_header = bui_node("panel_header", BuiNodeType::Node);
    apply_opendesign_preset(&mut panel_header, OpenDesignPreset::PanelHeader);
    apply_opendesign_styles(stylesheet, &mut panel_header, panel_header_source);

    let mut title_board = bui_node("title_board", BuiNodeType::Node);
    apply_opendesign_preset(&mut title_board, OpenDesignPreset::TitleBoard);
    apply_opendesign_styles(stylesheet, &mut title_board, title_board_source);
    let mut title_text = text_node(
        "title_text",
        title,
        36.0,
        "#FFFFFF",
        Some("STHeiti Medium.ttc"),
    );
    apply_opendesign_styles(stylesheet, &mut title_text, title_text_source);
    title_board.children.push(title_text);

    let mut close_btn = bui_node("close_btn", BuiNodeType::Button);
    close_btn.custom_tags.push("Action_Close_Shop".to_string());
    close_btn.actions.push(BuiActionBinding {
        event: "press".to_string(),
        emit: "close_shop_overlay".to_string(),
    });
    apply_opendesign_preset(&mut close_btn, OpenDesignPreset::CloseButton);
    apply_opendesign_styles(stylesheet, &mut close_btn, close_button_source);
    close_btn.children.push(text_node(
        "close_btn_text",
        "X",
        22.0,
        "#FFFFFF",
        Some("STHeiti Medium.ttc"),
    ));

    panel_header.children.push(title_board);
    panel_header.children.push(close_btn);

    let mut shop_body = bui_node("shop_body", BuiNodeType::Node);
    apply_opendesign_preset(&mut shop_body, OpenDesignPreset::ShopBody);
    apply_opendesign_styles(stylesheet, &mut shop_body, shop_body_source);

    let mut shop_scroll = bui_node("shop_scroll", BuiNodeType::Node);
    apply_opendesign_preset(&mut shop_scroll, OpenDesignPreset::ShopScroll);
    apply_opendesign_styles(stylesheet, &mut shop_scroll, shop_scroll_source);

    for article in overlay
        .descendants()
        .filter(|node| has_class(*node, "shop-card"))
    {
        shop_scroll.children.push(shop_card_node(article, stylesheet)?);
    }

    shop_body.children.push(shop_scroll);

    let mut foot_hint = bui_node("foot_hint", BuiNodeType::Node);
    apply_opendesign_preset(&mut foot_hint, OpenDesignPreset::FootHint);
    if let Some(foot_hint_source) = foot_hint_source {
        apply_opendesign_styles(stylesheet, &mut foot_hint, foot_hint_source);
    }
    let mut foot_hint_text = text_node(
        "foot_hint_text",
        footer,
        12.0,
        "#79614B",
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(foot_hint_source) = foot_hint_source {
        apply_opendesign_styles(stylesheet, &mut foot_hint_text, foot_hint_source);
    }
    foot_hint.children.push(foot_hint_text);

    panel.children.push(panel_header);
    panel.children.push(shop_body);
    panel.children.push(foot_hint);
    root.children.push(panel);
    stabilize_village_shop_overlay_defaults(&mut root);

    Ok(root)
}
