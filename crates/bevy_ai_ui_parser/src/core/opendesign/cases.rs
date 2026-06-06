use crate::core::{
    model::{BuiActionBinding, BuiNode, BuiNodeType, bui_node, text_node},
    opendesign::{
        build::apply_opendesign_styles,
        dom::{first_text_by_class, has_class},
        preset::{OpenDesignPreset, apply_opendesign_preset},
        stylesheet::OpenDesignStylesheet,
    },
    support::{
        ids::{format_price, pascal_case, sanitize_id},
        tree::find_bui_node_mut,
    },
};

pub(crate) fn shop_card_node(
    article: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
) -> Result<BuiNode, String> {
    let item_id = article.attribute("data-item-id").unwrap_or("item");
    let id = sanitize_id(item_id);
    let asset_text = normalize_village_shop_asset_label(
        item_id,
        &first_text_by_class(article, "asset-slot").unwrap_or_default(),
    );
    let item_name = first_text_by_class(article, "item-name").unwrap_or_default();
    let item_meta = first_text_by_class(article, "item-meta").unwrap_or_default();
    let item_bonus = first_text_by_class(article, "item-bonus").unwrap_or_default();
    let price = first_text_by_class(article, "price-tag")
        .or_else(|| article.attribute("data-price").map(format_price))
        .unwrap_or_default();

    let mut card = bui_node(&format!("shop_card_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut card, OpenDesignPreset::ShopCard);
    apply_opendesign_styles(stylesheet, &mut card, article);

    let mut item_main = bui_node(&format!("item_main_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut item_main, OpenDesignPreset::ItemMain);
    let item_main_source = article.descendants().find(|node| has_class(*node, "item-main"));
    if let Some(source) = item_main_source {
        apply_opendesign_styles(stylesheet, &mut item_main, source);
    }

    let mut asset_stack = bui_node(&format!("asset_stack_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut asset_stack, OpenDesignPreset::AssetStack);
    let asset_stack_source = article.descendants().find(|node| has_class(*node, "asset-stack"));
    if let Some(source) = asset_stack_source {
        apply_opendesign_styles(stylesheet, &mut asset_stack, source);
    }

    let mut asset_slot = bui_node(&format!("asset_slot_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut asset_slot, OpenDesignPreset::AssetSlot);
    let asset_slot_source = article.descendants().find(|node| has_class(*node, "asset-slot"));
    if let Some(source) = asset_slot_source {
        apply_opendesign_styles(stylesheet, &mut asset_slot, source);
    }
    let mut asset_label = text_node(
        &format!("asset_slot_{id}_text"),
        asset_text,
        12.0,
        "#79614B",
        Some("Hiragino Sans GB.ttc"),
    );
    asset_label.styles.width = Some("72px".to_string());
    if let Some(text_config) = &mut asset_label.text_config {
        text_config.font_size = 11.0;
        text_config.linebreak = Some("word_or_character".to_string());
    }
    if let Some(source) = asset_slot_source {
        apply_opendesign_styles(stylesheet, &mut asset_label, source);
    }
    asset_slot.children.push(asset_label);

    let mut stars = bui_node(&format!("stars_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut stars, OpenDesignPreset::Stars);
    let stars_source = article.descendants().find(|node| has_class(*node, "stars"));
    if let Some(source) = stars_source {
        apply_opendesign_styles(stylesheet, &mut stars, source);
    }
    for index in 1..=4 {
        stars.children.push(text_node(
            &format!("star_{id}_{index}"),
            "★",
            18.0,
            if index == 1 { "#D89A1F" } else { "#79614BCC" },
            Some("Hiragino Sans GB.ttc"),
        ));
    }
    asset_stack.children.push(asset_slot);
    asset_stack.children.push(stars);

    let mut item_copy = bui_node(&format!("item_copy_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut item_copy, OpenDesignPreset::ItemCopy);
    let item_copy_source = article.descendants().find(|node| has_class(*node, "item-copy"));
    if let Some(source) = item_copy_source {
        apply_opendesign_styles(stylesheet, &mut item_copy, source);
    }
    let item_name_source = article.descendants().find(|node| has_class(*node, "item-name"));
    let mut item_name_node = text_node(
        &format!("item_name_{id}"),
        item_name,
        24.0,
        "#3B2818",
        Some("STHeiti Medium.ttc"),
    );
    if let Some(source) = item_name_source {
        apply_opendesign_styles(stylesheet, &mut item_name_node, source);
    }
    item_copy.children.push(item_name_node);
    let item_meta_source = article.descendants().find(|node| has_class(*node, "item-meta"));
    let mut item_meta_node = text_node(
        &format!("item_meta_{id}"),
        item_meta,
        13.0,
        "#79614B",
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(source) = item_meta_source {
        apply_opendesign_styles(stylesheet, &mut item_meta_node, source);
    }
    item_copy.children.push(item_meta_node);
    let mut bonus = bui_node(&format!("item_bonus_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut bonus, OpenDesignPreset::ItemBonus);
    let item_bonus_source = article.descendants().find(|node| has_class(*node, "item-bonus"));
    if let Some(source) = item_bonus_source {
        apply_opendesign_styles(stylesheet, &mut bonus, source);
    }
    let mut item_bonus_text = text_node(
        &format!("item_bonus_{id}_text"),
        item_bonus,
        12.0,
        "#8B5F3356",
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(source) = item_bonus_source {
        apply_opendesign_styles(stylesheet, &mut item_bonus_text, source);
    }
    bonus.children.push(item_bonus_text);
    item_copy.children.push(bonus);

    item_main.children.push(asset_stack);
    item_main.children.push(item_copy);

    let mut purchase = bui_node(&format!("purchase_node_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut purchase, OpenDesignPreset::Purchase);
    let purchase_source = article
        .descendants()
        .find(|node| has_class(*node, "purchase-node"));
    if let Some(source) = purchase_source {
        apply_opendesign_styles(stylesheet, &mut purchase, source);
    }

    let mut price_tag = bui_node(&format!("price_tag_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut price_tag, OpenDesignPreset::PriceTag);
    let price_tag_source = article.descendants().find(|node| has_class(*node, "price-tag"));
    if let Some(source) = price_tag_source {
        apply_opendesign_styles(stylesheet, &mut price_tag, source);
    }

    let mut coin = bui_node(&format!("price_coin_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut coin, OpenDesignPreset::PriceCoin);
    if let Some(source) = article.descendants().find(|node| has_class(*node, "price-coin")) {
        apply_opendesign_styles(stylesheet, &mut coin, source);
    }
    price_tag.children.push(coin);
    let mut price_text = text_node(
        &format!("price_{id}_text"),
        price,
        30.0,
        "#D89A1F",
        Some("STHeiti Medium.ttc"),
    );
    if let Some(source) = price_tag_source {
        apply_opendesign_styles(stylesheet, &mut price_text, source);
    }
    price_tag.children.push(price_text);

    let mut buy = bui_node(&format!("buy_btn_{id}"), BuiNodeType::Button);
    buy.custom_tags.push("Sound_Click".to_string());
    buy.custom_tags.push(format!("Action_Buy_{}", pascal_case(&id)));
    buy.actions.push(BuiActionBinding {
        event: "press".to_string(),
        emit: format!("buy_item_{id}"),
    });
    apply_opendesign_preset(&mut buy, OpenDesignPreset::BuyButton);
    let buy_source = article.descendants().find(|node| has_class(*node, "buy-btn"));
    if let Some(source) = buy_source {
        apply_opendesign_styles(stylesheet, &mut buy, source);
    }
    let mut buy_text = text_node(
        &format!("buy_btn_{id}_text"),
        "购买",
        20.0,
        "#FFFFFF",
        Some("STHeiti Medium.ttc"),
    );
    if let Some(source) = buy_source {
        apply_opendesign_styles(stylesheet, &mut buy_text, source);
    }
    buy.children.push(buy_text);

    purchase.children.push(price_tag);
    purchase.children.push(buy);
    card.children.push(item_main);
    card.children.push(purchase);

    Ok(card)
}

pub(crate) fn stabilize_village_shop_overlay_defaults(root: &mut BuiNode) {
    let is_village_shop = find_bui_node_mut(root, "shop_card_hut").is_some()
        && find_bui_node_mut(root, "shop_card_statue").is_some()
        && find_bui_node_mut(root, "title_board").is_some()
        && find_bui_node_mut(root, "close_btn").is_some();
    if !is_village_shop {
        return;
    }

    for id in ["panel", "title_board", "close_btn"] {
        if let Some(node) = find_bui_node_mut(root, id) {
            strip_effect_helper_children(node);
        }
    }

    if let Some(panel) = find_bui_node_mut(root, "panel") {
        panel.visuals.box_shadow = None;
    }
    if let Some(overlay_root) = find_bui_node_mut(root, "overlay_root") {
        overlay_root.styles.left = None;
        overlay_root.styles.right = None;
        overlay_root.styles.top = None;
        overlay_root.styles.bottom = None;
        overlay_root.styles.padding = Some("16px 16px".to_string());
    }
    if let Some(title_board) = find_bui_node_mut(root, "title_board") {
        title_board.visuals.box_shadow = None;
    }
    if let Some(close_btn) = find_bui_node_mut(root, "close_btn") {
        close_btn.visuals.box_shadow = None;
    }
    if let Some(title_text) = find_bui_node_mut(root, "title_text") {
        reset_village_text_node(title_text, "STHeiti Medium.ttc");
    }
    if let Some(foot_hint_text) = find_bui_node_mut(root, "foot_hint_text")
        && let Some(text_config) = &mut foot_hint_text.text_config
    {
        text_config.text_align = None;
    }

    for item_id in ["hut", "statue", "cart", "lantern", "fountain"] {
        let shop_card_id = format!("shop_card_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &shop_card_id) {
            strip_effect_helper_children(node);
            node.visuals.box_shadow = None;
            node.visuals.background_color = Some("#f8ecd0".to_string());
        }

        let asset_slot_id = format!("asset_slot_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &asset_slot_id) {
            strip_effect_helper_children(node);
            node.visuals.box_shadow = None;
            node.styles.position_type = None;
        }

        let price_tag_id = format!("price_tag_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &price_tag_id) {
            strip_effect_helper_children(node);
            node.visuals.box_shadow = None;
            node.visuals.background_color = Some("#f8ecd0".to_string());
            node.styles.position_type = None;
        }

        let price_coin_id = format!("price_coin_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &price_coin_id) {
            strip_effect_helper_children(node);
            node.styles.position_type = None;
            node.visuals.background_color = Some("#d89a1f".to_string());
            node.visuals.border_color = Some("#3b2818".to_string());
            node.visuals.border_width = Some("1px".to_string());
            node.visuals.border_radius = Some("999px".to_string());
        }

        let buy_btn_id = format!("buy_btn_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &buy_btn_id) {
            strip_effect_helper_children(node);
            node.visuals.box_shadow = None;
            node.visuals.background_color = Some("#3fb45a".to_string());
            node.styles.position_type = Some("relative".to_string());
            node.state_visuals.remove("hovered");
        }

        let asset_slot_text_id = format!("asset_slot_{item_id}_text");
        if let Some(node) = find_bui_node_mut(root, &asset_slot_text_id)
            && let Some(text_config) = &mut node.text_config
        {
            text_config.text_align = None;
        }

        let item_name_id = format!("item_name_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &item_name_id) {
            reset_village_text_node(node, "STHeiti Medium.ttc");
        }

        let item_meta_id = format!("item_meta_{item_id}");
        if let Some(node) = find_bui_node_mut(root, &item_meta_id) {
            clear_village_text_layout_enhancements(node);
        }

        let item_bonus_text_id = format!("item_bonus_{item_id}_text");
        if let Some(node) = find_bui_node_mut(root, &item_bonus_text_id) {
            clear_village_text_layout_enhancements(node);
            if let Some(text_config) = &mut node.text_config {
                text_config.font_weight = None;
            }
        }

        let price_text_id = format!("price_{item_id}_text");
        if let Some(node) = find_bui_node_mut(root, &price_text_id) {
            reset_village_text_node(node, "STHeiti Medium.ttc");
        }

        let buy_btn_text_id = format!("buy_btn_{item_id}_text");
        if let Some(node) = find_bui_node_mut(root, &buy_btn_text_id) {
            reset_village_text_node(node, "STHeiti Medium.ttc");
            node.state_visuals.remove("hovered");
        }
    }

    if let Some(shop_scroll) = find_bui_node_mut(root, "shop_scroll") {
        shop_scroll.styles.height = Some("475.2px".to_string());
        shop_scroll.styles.max_height = Some("475.2px".to_string());
    }
}

fn normalize_village_shop_asset_label(item_id: &str, asset_text: &str) -> String {
    match item_id {
        "hut" => "小屋".to_string(),
        "statue" => "雕像".to_string(),
        "cart" => "货车".to_string(),
        "lantern" => "灯".to_string(),
        _ => asset_text.to_string(),
    }
}

fn strip_effect_helper_children(node: &mut BuiNode) {
    node.children.retain(|child| {
        !child
            .custom_tags
            .iter()
            .any(|tag| tag == "css-gradient-overlay" || tag == "css-box-shadow-layer")
    });
}

fn reset_village_text_node(node: &mut BuiNode, font_path: &str) {
    clear_village_text_layout_enhancements(node);
    if let Some(text_config) = &mut node.text_config {
        text_config.font_path = Some(font_path.to_string());
        text_config.font_weight = None;
        text_config.linebreak = None;
        text_config.allow_newlines = None;
    }
}

fn clear_village_text_layout_enhancements(node: &mut BuiNode) {
    if let Some(text_config) = &mut node.text_config {
        text_config.line_height = None;
        text_config.letter_spacing = None;
        text_config.text_align = None;
        text_config.text_shadow = None;
    }
}
