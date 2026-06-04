use super::*;

fn css_simple_linear_gradient_overlay(
    layer: &str,
) -> Option<(SimpleGradientOverlayDirection, String, f32, f32)> {
    let (direction, bands) = css_simple_linear_gradient_bands(layer)?;
    let band = bands.into_iter().next()?;
    Some((direction, band.color, band.start_ratio, band.end_ratio))
}

const VILLAGE_SHOP_HTML: &str = include_str!(
    "../../../examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.html"
);
const VILLAGE_SHOP_IR: &str = include_str!(
    "../../../examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.ir.json"
);
const QUEST_NOTICE_HTML: &str = include_str!(
    "../../../examples/UiParserTest/opendesignTest/quest_notice_overlay/quest-notice-overlay.html"
);
const HERO_GAME_UI_HTML: &str = include_str!(
    "../../../examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.html"
);
const HERO_GAME_UI_JSON: &str = include_str!(
    "../../../examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.json"
);
const HERO_GAME_UI_IR: &str = include_str!(
    "../../../examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.ir.json"
);

#[test]
fn opendesign_media_query_rules_resolve_into_bui_styles() {
    let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile");

    let panel = find_bui_node(&document.root, "panel");
    assert_eq!(panel.styles.width.as_deref(), Some("720px"));

    let card = find_bui_node(&document.root, "shop_card_hut");
    assert_eq!(
        card.styles.grid_template_columns.as_deref(),
        Some("flex(1) px(140)")
    );

    let item_main = find_bui_node(&document.root, "item_main_hut");
    assert_eq!(
        item_main.styles.grid_template_columns.as_deref(),
        Some("px(104) flex(1)")
    );
}

#[test]
fn opendesign_active_transform_compiles_to_pressed_state_scale() {
    let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile");

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
    let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile");

    let buy_text = find_bui_node(&document.root, "buy_btn_hut_text");
    for (_, state) in &buy_text.state_visuals {
        assert!(
            state.styles.ui_scale.is_none(),
            "text node should not inherit transform (ui_scale) from parent button"
        );
    }
    assert_eq!(
        buy_text
            .text_config
            .as_ref()
            .map(|config| config.font_color.as_str()),
        Some("#ffffff")
    );
}

#[test]
fn opendesign_ir_export_uses_3_0_shape() {
    let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile");
    let ir = BuiIrDocument::from_compat_document(&document);

    assert_eq!(ir.version, "3.0-ir");
    assert_eq!(ir.root.kind, "node");

    let panel = ir
        .root
        .children
        .iter()
        .find(|child| child.id == "panel")
        .expect("panel should exist");
    assert_eq!(panel.layout.styles.max_width.as_deref(), Some("720px"));

    let buy_button = find_ir_node(&ir.root, "buy_btn_hut");
    assert_eq!(buy_button.kind, "button");
    assert!(buy_button.content.is_empty());
    assert!(
        buy_button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref())
            .is_some()
    );
}

#[test]
fn opendesign_ir_snapshot_can_load_through_runtime_parser() {
    let ir_json = opendesign_html_to_bui_ir_json_str(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile to IR");
    let document = parse_bui_document(&ir_json).expect("BUI IR should parse for runtime");

    assert_eq!(document.version, EXPECTED_VERSION);

    let panel = find_bui_node(&document.root, "panel");
    assert_eq!(panel.styles.max_height.as_deref(), Some("648px"));

    let buy_button = find_bui_node(&document.root, "buy_btn_hut");
    assert!(matches!(buy_button.node_type, BuiNodeType::Button));
    assert_eq!(
        buy_button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.styles.ui_scale.as_deref()),
        Some("0.95 0.95")
    );

    validate_bui_ir_json_str(&ir_json).expect("BUI IR validator should accept generated IR");
}

#[test]
fn checked_in_ir_fixture_loads_through_runtime_parser() {
    let document = parse_bui_document(VILLAGE_SHOP_IR).expect("checked-in IR should parse");

    let root = find_bui_node(&document.root, "overlay_root");
    assert_eq!(root.styles.height.as_deref(), Some("100%"));

    let close_button = find_bui_node(&document.root, "close_btn");
    assert!(matches!(close_button.node_type, BuiNodeType::Button));
    assert_eq!(
        close_button
            .actions
            .first()
            .map(|action| (action.event.as_str(), action.emit.as_str())),
        Some(("press", "close_shop_overlay"))
    );
}

#[test]
fn generic_opendesign_overlay_compiles_without_shop_structure() {
    let document = opendesign_html_to_bui_document(QUEST_NOTICE_HTML)
        .expect("generic OpenDesign overlay should compile");

    let title = find_bui_node(&document.root, "notice_title_text_1");
    assert_eq!(
        title
            .text_config
            .as_ref()
            .map(|text| text.content.as_str()),
        Some("新的委托")
    );

    let accept = find_bui_node(&document.root, "primary_btn");
    assert!(matches!(accept.node_type, BuiNodeType::Button));
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

    let ir_json = opendesign_html_to_bui_ir_json_str(QUEST_NOTICE_HTML)
        .expect("generic OpenDesign overlay should compile to IR");
    validate_bui_ir_json_str(&ir_json).expect("generic IR should validate");
}

#[test]
fn opendesign_css_important_values_are_normalized() {
    assert_eq!(normalize_css_value("  48px !important "), "48px");
    assert_eq!(normalize_css_value("\"#ffffff\" !important"), "#ffffff");
}

#[test]
fn opendesign_color_mix_with_transparency_preserves_alpha() {
    assert_eq!(
        css_color("color-mix(in oklab, #3b2818, transparent 38%)").as_deref(),
        Some("#3B28189E")
    );
    assert_eq!(
        css_color("color-mix(in oklab, black, transparent 40%)").as_deref(),
        Some("#00000099")
    );
    assert_eq!(
        css_color("color-mix(in oklab, #fff, transparent)").as_deref(),
        Some("#FFFFFF80")
    );
}

#[test]
fn unsupported_pseudo_element_selectors_do_not_leak_into_node_styles() {
    let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
        .expect("OpenDesign HTML should compile");

    let scroll = find_bui_node(&document.root, "shop_scroll");
    assert_eq!(
        scroll.visuals.background_color, None,
        "::-webkit-scrollbar-thumb background should not leak into shop_scroll"
    );
    assert_eq!(
        scroll.visuals.border_radius, None,
        "::-webkit-scrollbar-thumb border-radius should not leak into shop_scroll"
    );
    assert_eq!(
        scroll.styles.width, None,
        "::-webkit-scrollbar width should not leak into shop_scroll"
    );
}

#[test]
fn opendesign_length_functions_resolve_against_default_viewport() {
    assert_eq!(css_eval_length_function("min(100%, 460px)").as_deref(), Some("460px"));
    assert_eq!(css_eval_length_function("min(66vh, 620px)").as_deref(), Some("475.2px"));
    assert_eq!(css_eval_length_function("min(92vw, 720px)").as_deref(), Some("720px"));
    assert_eq!(css_eval_length_function("clamp(28px, 7vw, 36px)").as_deref(), Some("36px"));
    assert_eq!(css_eval_length_function("clamp(20px, 4vw, 24px)").as_deref(), Some("24px"));
    assert_eq!(css_eval_length_function("max(18px, env(safe-area-inset-top))").as_deref(), Some("18px"));
}

fn find_ir_node<'a>(node: &'a BuiIrNode, id: &str) -> &'a BuiIrNode {
    find_ir_node_optional(node, id).unwrap_or_else(|| panic!("IR node '{id}' should exist"))
}

fn find_ir_node_optional<'a>(node: &'a BuiIrNode, id: &str) -> Option<&'a BuiIrNode> {
    if node.id == id {
        return Some(node);
    }

    node.children
        .iter()
        .find_map(|child| find_ir_node_optional(child, id))
}

fn find_bui_node<'a>(node: &'a BuiNode, id: &str) -> &'a BuiNode {
    if node.id == id {
        return node;
    }

    node.children
        .iter()
        .find_map(|child| find_bui_node_optional(child, id))
        .unwrap_or_else(|| panic!("BUI node '{id}' should exist"))
}

fn find_bui_node_optional<'a>(node: &'a BuiNode, id: &str) -> Option<&'a BuiNode> {
    if node.id == id {
        return Some(node);
    }

    node.children
        .iter()
        .find_map(|child| find_bui_node_optional(child, id))
}

#[test]
fn hero_game_ui_html_compiles_to_bui_document() {
    let document = opendesign_html_to_bui_document(HERO_GAME_UI_HTML)
        .expect("hero game UI HTML should compile");

    let root = find_bui_node(&document.root, "overlay_root");
    assert_eq!(
        root.visuals.background_color.as_deref(),
        Some("#362E36")
    );
    assert_eq!(root.styles.width.as_deref(), Some("1280px"));
    assert_eq!(root.styles.height.as_deref(), Some("100%"));

    let page_title = find_bui_node(&document.root, "page_title_text_1");
    assert_eq!(
        page_title
            .text_config
            .as_ref()
            .map(|text| text.content.as_str()),
        Some("英雄")
    );

    let hero_name = find_bui_node(&document.root, "hero_name_text_1");
    assert_eq!(
        hero_name
            .text_config
            .as_ref()
            .map(|text| text.content.as_str()),
        Some("Olympia")
    );
}

#[test]
fn hero_game_ui_three_entry_paths_produce_identical_bui_documents() {
    let from_html = opendesign_html_to_bui_document(HERO_GAME_UI_HTML)
        .expect("HTML should compile");
    let from_json = parse_bui_document(HERO_GAME_UI_JSON)
        .expect("2.x JSON should parse");
    let from_ir = parse_bui_document(HERO_GAME_UI_IR)
        .expect("3.0-ir JSON should parse");

    let html_value = serde_json::to_value(&from_html)
        .expect("HTML document should serialize");
    let json_value = serde_json::to_value(&from_json)
        .expect("JSON document should serialize");
    let ir_value = serde_json::to_value(&from_ir)
        .expect("IR document should serialize");

    assert_eq!(
        html_value, ir_value,
        "HTML and IR entry paths should produce identical BuiDocuments"
    );
    assert_eq!(
        json_value, ir_value,
        "JSON and IR entry paths should produce identical BuiDocuments"
    );
}

#[test]
fn opendesign_inherited_line_height_applies_to_text_nodes() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        color: #f4e7ca;
        font-size: 20px;
        line-height: 32px;
      }
    </style>
    <main class="game-stage">
      <div class="copy">
        <span>Line one</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "span_text_1");
    let text_config = text_node
        .text_config
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.font_size, 20.0);
    assert_eq!(text_config.line_height.as_deref(), Some("32px"));
    assert_eq!(text_config.font_color.to_ascii_lowercase(), "#f4e7ca");
}

#[test]
fn opendesign_inherited_unitless_line_height_preserves_relative_value() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        font-size: 20px;
        line-height: 0.9;
      }
    </style>
    <main class="game-stage">
      <div class="copy">
        <span>Line one</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "span_text_1");
    let text_config = text_node
        .text_config
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.line_height.as_deref(), Some("0.9"));
    assert_eq!(
        parse_text_line_height(
            text_config
                .line_height
                .as_deref()
                .expect("line-height should be present")
        )
        .expect("line-height should parse"),
        LineHeight::RelativeToFont(0.9)
    );
}

#[test]
fn opendesign_inherited_white_space_nowrap_sets_no_wrap_linebreak() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        white-space: nowrap;
      }
    </style>
    <main class="game-stage">
      <div class="copy">
        <span>Line one</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "span_text_1");
    let text_config = text_node
        .text_config
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.linebreak.as_deref(), Some("no_wrap"));
    assert_eq!(text_config.allow_newlines, Some(false));
}

#[test]
fn opendesign_inherited_white_space_normal_enables_wrapping() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        white-space: normal;
      }
    </style>
    <main class="game-stage">
      <div class="copy">
        <span>Line one</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "span_text_1");
    let text_config = text_node
        .text_config
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.linebreak.as_deref(), Some("word_boundary"));
    assert_eq!(text_config.allow_newlines, Some(false));
}

#[test]
fn opendesign_inherited_font_weight_preserves_numeric_weight() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        font-weight: 850;
      }
    </style>
    <main class="game-stage">
      <div class="copy">
        <span>Line one</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "span_text_1");
    let text_config = text_node
        .text_config
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.font_weight, Some(850));
}

#[test]
fn opendesign_inherited_font_weight_maps_bold_keyword() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        font-weight: bold;
      }
    </style>
    <main class="game-stage">
      <div class="copy">
        <span>Line one</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "span_text_1");
    let text_config = text_node
        .text_config
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.font_weight, Some(700));
}

#[test]
fn css_linear_gradient_direction_supports_default_and_keyword_directions() {
    assert_eq!(
        css_simple_linear_gradient_direction(&["to right", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::LeftToRight, 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["to left", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::RightToLeft, 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["to top", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::BottomToTop, 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::TopToBottom, 0))
    );
}

#[test]
fn css_linear_gradient_direction_maps_diagonal_angles_to_dominant_axis() {
    assert_eq!(
        css_linear_gradient_direction_from_degrees(90.0),
        Some(SimpleGradientOverlayDirection::LeftToRight)
    );
    assert_eq!(
        css_linear_gradient_direction_from_degrees(135.0),
        Some(SimpleGradientOverlayDirection::LeftToRight)
    );
    assert_eq!(
        css_linear_gradient_direction_from_degrees(180.0),
        Some(SimpleGradientOverlayDirection::TopToBottom)
    );
    assert_eq!(
        css_linear_gradient_direction_from_degrees(315.0),
        Some(SimpleGradientOverlayDirection::RightToLeft)
    );
}

#[test]
fn css_simple_linear_gradient_overlay_supports_default_direction() {
    let overlays =
        css_simple_linear_gradient_overlays("linear-gradient(#6d5a3d, #2d2119)");

    assert!(!overlays.is_empty());
    assert_eq!(
        overlays[0].color.to_ascii_uppercase(),
        "#6D5A3D",
        "first band should be the first gradient stop color"
    );
    let last = overlays.last().expect("should have trailing band");
    assert_eq!(last.color.to_ascii_uppercase(), "#2D2119");
}

#[test]
fn css_simple_linear_gradient_overlay_uses_trailing_band_for_solid_two_stop_gradients() {
    let overlays = css_simple_linear_gradient_overlays(
        "linear-gradient(180deg, #f7edd6, #8c6c52)",
    );

    assert!(overlays.len() >= 2);
    assert_eq!(overlays[0].color.to_ascii_uppercase(), "#F7EDD6");
    let last = overlays.last().expect("should have trailing band");
    assert_eq!(last.color.to_ascii_uppercase(), "#8C6C52");
    if let SimpleGradientOverlayKind::Linear { end_ratio, .. } = &last.kind {
        assert!((*end_ratio - 1.0).abs() < 0.01);
    }
}

#[test]
fn css_simple_linear_gradient_overlays_extract_trailing_segments_from_solid_multi_stop_gradients()
{
    let overlays = css_simple_linear_gradient_overlays(
        "linear-gradient(180deg, #f5e7c8 0%, #d1b48c 46%, #8b6c53 100%)",
    );

    assert_eq!(overlays.len(), 2);

    match &overlays[0].kind {
        SimpleGradientOverlayKind::Linear {
            direction,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::TopToBottom);
            assert_eq!(overlays[0].color, "#d1b48c");
            assert!((*start_ratio - 0.23).abs() < 0.01);
            assert!((*end_ratio - 0.46).abs() < 0.01);
        }
        _ => panic!("expected linear overlay"),
    }

    match &overlays[1].kind {
        SimpleGradientOverlayKind::Linear {
            direction,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::TopToBottom);
            assert_eq!(overlays[1].color, "#8b6c53");
            assert!((*start_ratio - 0.73).abs() < 0.01);
            assert!((*end_ratio - 1.0).abs() < 0.01);
        }
        _ => panic!("expected linear overlay"),
    }
}

#[test]
fn css_simple_linear_gradient_overlay_supports_diagonal_highlight_bands() {
    let overlay = css_simple_linear_gradient_overlay(
        "linear-gradient(135deg, #f7edd6 0 22%, transparent 22% 48%)",
    )
    .expect("diagonal gradient should produce an overlay");

    assert_eq!(overlay.0, SimpleGradientOverlayDirection::LeftToRight);
    assert_eq!(overlay.1, "#f7edd6");
    assert_eq!(overlay.2, 0.0);
    assert_eq!(overlay.3, 0.48);
}

#[test]
fn css_simple_linear_gradient_overlays_extract_multiple_highlight_bands() {
    let overlays = css_simple_linear_gradient_overlays(
        "linear-gradient(110deg, transparent 0 16%, #ffffff4d 16% 21%, transparent 21% 36%, #ffffff33 36% 41%, transparent 41%)",
    );

    assert_eq!(overlays.len(), 2);

    match &overlays[0].kind {
        SimpleGradientOverlayKind::Linear {
            direction,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::LeftToRight);
            assert_eq!(overlays[0].color, "#ffffff4d");
            assert_eq!(*start_ratio, 0.16);
            assert_eq!(*end_ratio, 0.36);
        }
        SimpleGradientOverlayKind::Radial { .. }
        | SimpleGradientOverlayKind::RadialRing { .. }
        | SimpleGradientOverlayKind::ConicArc { .. } => panic!("expected linear overlay"),
    }

    match &overlays[1].kind {
        SimpleGradientOverlayKind::Linear {
            direction,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::LeftToRight);
            assert_eq!(overlays[1].color, "#ffffff33");
            assert_eq!(*start_ratio, 0.36);
            assert_eq!(*end_ratio, 0.41);
        }
        SimpleGradientOverlayKind::Radial { .. }
        | SimpleGradientOverlayKind::RadialRing { .. }
        | SimpleGradientOverlayKind::ConicArc { .. } => panic!("expected linear overlay"),
    }
}

#[test]
fn css_simple_conic_gradient_overlays_extract_rotated_arc_bands() {
    let overlays = css_simple_conic_gradient_overlays(
        "conic-gradient(from 10deg, transparent 0 12%, #6b8190 12% 16%, transparent 16% 28%, #5e7382 28% 32%, transparent 32%)",
    );

    assert_eq!(overlays.len(), 2);

    match &overlays[0].kind {
        SimpleGradientOverlayKind::ConicArc {
            rotation_degrees,
            width,
            height,
            ..
        } => {
            assert!((*rotation_degrees - 60.4).abs() < 0.2);
            assert!((*width - 0.08).abs() < 0.01);
            assert!((*height - 0.06).abs() < f32::EPSILON);
            assert_eq!(overlays[0].color, "#6b8190");
        }
        SimpleGradientOverlayKind::Linear { .. }
        | SimpleGradientOverlayKind::Radial { .. }
        | SimpleGradientOverlayKind::RadialRing { .. } => panic!("expected conic arc"),
    }

    match &overlays[1].kind {
        SimpleGradientOverlayKind::ConicArc {
            rotation_degrees, ..
        } => {
            assert!((*rotation_degrees - 118.0).abs() < 0.2);
            assert_eq!(overlays[1].color, "#5e7382");
        }
        SimpleGradientOverlayKind::Linear { .. }
        | SimpleGradientOverlayKind::Radial { .. }
        | SimpleGradientOverlayKind::RadialRing { .. } => panic!("expected conic arc"),
    }
}

#[test]
fn opendesign_inline_custom_properties_resolve_inside_background_gradients() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .meter-fill {
        width: 180px;
        height: 24px;
        background:
          linear-gradient(110deg, transparent 0 16%, oklch(100% 0 0 / 0.3) 16% 21%, transparent 21% 36%, oklch(100% 0 0 / 0.2) 36% 41%, transparent 41%),
          linear-gradient(90deg, var(--from), var(--to));
      }
    </style>
    <main class="game-stage">
      <div class="meter-fill" style="--from: oklch(73% 0.17 132); --to: oklch(86% 0.12 118);"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let meter_fill = find_bui_node(&document.root, "meter_fill");

    assert_eq!(
        meter_fill.visuals.background_color.as_deref(),
        Some("#31C4A4")
    );
    let overlay_colors = meter_fill
        .children
        .iter()
        .filter(|child| child.custom_tags.iter().any(|tag| tag == "css-gradient-overlay"))
        .filter_map(|child| child.visuals.background_color.as_deref())
        .collect::<Vec<_>>();
    assert!(
        overlay_colors.contains(&"#96E4E1"),
        "resolved var(--to) gradient color should survive as an overlay"
    );
    assert!(
        overlay_colors.contains(&"#C4FFFF33"),
        "resolved highlight band should survive as a separate overlay"
    );
    assert!(
        overlay_colors.contains(&"#C4FFFF4D"),
        "resolved leading highlight band should survive as a separate overlay"
    );
}

#[test]
fn css_multiply_blend_fallback_color_darkens_and_softens_alpha() {
    assert_eq!(
        css_multiply_blend_fallback_color("#80A0C080"),
        Some("#647D9671".to_string())
    );
    assert_eq!(
        css_multiply_blend_fallback_color("#C8E4F250"),
        Some("#9CB2BD46".to_string())
    );
}

#[test]
fn opendesign_mix_blend_mode_multiply_darkens_gradient_overlays() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 160px;
        position: relative;
      }
      .wash {
        position: absolute;
        inset: 0;
        background:
          radial-gradient(circle at 18% 12%, oklch(84% 0.072 235 / 0.72), transparent 25%),
          linear-gradient(90deg, oklch(86% 0.064 230 / 0.16) 0 48%, transparent 59%);
        mix-blend-mode: multiply;
      }
    </style>
    <main class="game-stage">
      <div class="wash"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let wash = find_bui_node(&document.root, "wash");

    let overlay_colors = wash
        .children
        .iter()
        .filter(|child| child.custom_tags.iter().any(|tag| tag == "css-gradient-overlay"))
        .filter_map(|child| child.visuals.background_color.as_deref())
        .collect::<Vec<_>>();

    assert!(
        overlay_colors.contains(&"#37A4C7A2"),
        "multiply fallback should darken the radial wash overlay"
    );
    assert!(
        overlay_colors.contains(&"#42AAC724"),
        "multiply fallback should darken the linear wash overlay"
    );
}

#[test]
fn css_adjust_filter_color_applies_brightness_contrast_and_saturation() {
    let adjusted = css_adjust_filter_color(
        "#7A6A5A80",
        CssFilterColorAdjustment {
            brightness: 1.08,
            contrast: 1.02,
            saturate: 1.04,
        },
    )
    .expect("filter color adjustment should produce a color");

    assert_eq!(adjusted, "#84726080");
}

#[test]
fn opendesign_filter_color_adjustment_updates_node_and_overlay_colors() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
        position: relative;
      }
      .panel {
        position: absolute;
        inset: 0;
        background:
          linear-gradient(90deg, #6A5848 0 55%, transparent 70%),
          #7A6A5A;
        border: 1px solid #9A8A7A;
        filter: brightness(1.08) saturate(1.04) contrast(1.02);
      }
    </style>
    <main class="game-stage">
      <div class="panel"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    let overlay_colors = panel
        .children
        .iter()
        .filter(|child| child.custom_tags.iter().any(|tag| tag == "css-gradient-overlay"))
        .filter_map(|child| child.visuals.background_color.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(panel.visuals.background_color.as_deref(), Some("#847260"));
    assert_eq!(panel.visuals.border_color.as_deref(), Some("#A79583"));
    assert!(
        overlay_colors.contains(&"#735E4C"),
        "filter color adjustment should affect gradient overlays too"
    );
}

#[test]
fn opendesign_hover_filter_compiles_to_hovered_state_colors() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .btn {
        background: #7A6A5A;
        border: 1px solid #9A8A7A;
        color: #F3E8D5;
      }
      .btn:hover {
        filter: brightness(1.08);
      }
    </style>
    <main class="game-stage">
      <button class="btn">Hover</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "btn");
    let hovered = button
        .state_visuals
        .get("hovered")
        .expect("hovered state should exist");

    assert_eq!(
        hovered.visuals.background_color.as_deref(),
        Some("#847261")
    );
    assert_eq!(hovered.visuals.border_color.as_deref(), Some("#A69584"));
    assert_eq!(hovered.text_color.as_deref(), None);
}

#[test]
fn opendesign_hover_opacity_compiles_to_hovered_state_alpha_colors() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .btn {
        background: #7A6A5A;
        border: 1px solid #9A8A7A;
        color: #F3E8D5;
      }
      .btn:hover {
        opacity: 0.5;
      }
    </style>
    <main class="game-stage">
      <button class="btn">Hover</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "btn");
    let hovered = button
        .state_visuals
        .get("hovered")
        .expect("hovered state should exist");
    let text_child = button
        .children
        .iter()
        .find(|child| matches!(child.node_type, BuiNodeType::Text))
        .expect("button should have a direct text child");
    let hovered_text = text_child
        .state_visuals
        .get("hovered")
        .expect("text child should inherit hovered state");

    assert_eq!(
        hovered.visuals.background_color.as_deref(),
        Some("#7A6A5A80")
    );
    assert_eq!(hovered.visuals.border_color.as_deref(), Some("#9A8A7A80"));
    assert_eq!(hovered.text_color.as_deref(), None);
    assert_eq!(hovered_text.text_color.as_deref(), Some("#F3E8D580"));
}

#[test]
fn direct_text_children_inherit_parent_hover_text_color_state() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .btn {
        color: #E0D2C0;
      }
      .btn:hover {
        color: #FFF4DE;
      }
    </style>
    <main class="game-stage">
      <button class="btn">Hover</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "btn");
    let text_child = button
        .children
        .iter()
        .find(|child| matches!(child.node_type, BuiNodeType::Text))
        .expect("button should have a direct text child");

    let hovered = text_child
        .state_visuals
        .get("hovered")
        .expect("text child should inherit hovered state");

    assert_eq!(hovered.text_color.as_deref(), Some("#FFF4DE"));
}

#[test]
fn direct_text_children_inherit_parent_hover_opacity_text_state() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .btn {
        color: #E0D2C0;
      }
      .btn:hover {
        opacity: 0.5;
      }
    </style>
    <main class="game-stage">
      <button class="btn">Hover</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "btn");
    let text_child = button
        .children
        .iter()
        .find(|child| matches!(child.node_type, BuiNodeType::Text))
        .expect("button should have a direct text child");

    let hovered = text_child
        .state_visuals
        .get("hovered")
        .expect("text child should inherit hovered state");

    assert_eq!(hovered.text_color.as_deref(), Some("#E0D2C080"));
}

#[test]
fn css_simple_clip_polygon_contour_extracts_bounds_and_accent() {
    let spec = css_simple_clip_polygon_contour(
        "polygon(35% 0, 68% 4%, 81% 18%, 90% 38%, 80% 64%, 86% 100%, 24% 100%, 31% 70%, 13% 55%, 20% 31%)",
    )
    .expect("clip polygon should parse");

    assert!((spec.left - 0.13).abs() < 0.001);
    assert!((spec.right - 0.10).abs() < 0.001);
    assert!((spec.top - 0.0).abs() < 0.001);
    assert!((spec.bottom - 0.0).abs() < 0.001);
    assert!(spec.fill_left > spec.left);
    assert!(spec.fill_right > spec.right);
    assert!(spec.fill_top >= spec.top);
    assert!(spec.fill_bottom >= spec.bottom);
    assert!(spec.accent_width >= 0.08 && spec.accent_width <= 0.34);
    assert!(spec.accent_height >= 0.06 && spec.accent_height <= 0.24);
}

#[test]
fn opendesign_clip_path_polygon_adds_contour_children() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
        position: relative;
      }
      .cutout {
        position: absolute;
        inset: 0;
        background: #d7d1c6;
        border: 1px solid #fff3d6;
        clip-path: polygon(35% 0, 68% 4%, 81% 18%, 90% 38%, 80% 64%, 86% 100%, 24% 100%, 31% 70%, 13% 55%, 20% 31%);
      }
    </style>
    <main class="game-stage">
      <div class="cutout"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let cutout = find_bui_node(&document.root, "cutout");

    let contour_children = cutout
        .children
        .iter()
        .filter(|child| child.custom_tags.iter().any(|tag| tag == "css-clip-contour"))
        .collect::<Vec<_>>();

    assert_eq!(contour_children.len(), 3);
    assert_eq!(
        cutout.visuals.background_color.as_deref(),
        Some("transparent")
    );
    assert_eq!(
        cutout.visuals.border_color.as_deref(),
        Some("transparent")
    );
    assert_eq!(
        contour_children[0].visuals.background_color.as_deref(),
        Some("#d7d1c6")
    );
    assert_eq!(
        contour_children[1].visuals.border_color.as_deref(),
        Some("#fff3d6")
    );
    assert_eq!(
        contour_children[2].visuals.background_color.as_deref(),
        Some("#D7D1C694")
    );
}

#[test]
fn css_simple_mask_fade_supports_keyword_and_default_directions() {
    let left_fade = css_simple_mask_fade(
        "linear-gradient(to right, transparent 0, black 11%, black 100%)",
    )
    .expect("keyword mask fade should parse");
    assert!(matches!(left_fade.direction, MaskFadeDirection::LeftToRight));
    assert!((left_fade.fade_ratio - 0.11).abs() < 0.001);

    let top_fade = css_simple_mask_fade(
        "linear-gradient(transparent 0, black 18%, black 100%)",
    )
    .expect("default-direction mask fade should parse");
    assert!(matches!(top_fade.direction, MaskFadeDirection::TopToBottom));
    assert!((top_fade.fade_ratio - 0.18).abs() < 0.001);

    let right_fade = css_simple_mask_fade(
        "linear-gradient(to left, transparent 0, black 20%, black 100%)",
    )
    .expect("reverse keyword mask fade should parse");
    assert!(matches!(right_fade.direction, MaskFadeDirection::RightToLeft));
}

#[test]
fn opendesign_mask_image_none_clears_existing_mask_fade_children() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
        position: relative;
      }
      .panel {
        position: absolute;
        inset: 0;
        background: #c7a97a;
        mask-image: linear-gradient(90deg, transparent 0, black 11%, black 100%);
      }
      .game-stage.panel-open .panel {
        mask-image: none;
      }
    </style>
    <main class="game-stage panel-open">
      <div class="panel"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    let mask_children = panel
        .children
        .iter()
        .filter(|child| child.custom_tags.iter().any(|tag| tag == "css-mask-fade"))
        .count();

    assert_eq!(mask_children, 0);
}

#[test]
fn css_gradient_stops_expand_single_position_tail_stop_into_a_band() {
    let stops = css_gradient_stops(&["transparent 0 72%", "#1A0E1233 100%"])
        .expect("gradient stops should parse");

    assert_eq!(stops.len(), 2);
    assert_eq!(stops[0].color, "transparent");
    assert_eq!(stops[0].start_ratio, 0.0);
    assert_eq!(stops[0].end_ratio, 0.72);
    assert_eq!(stops[1].color, "#1A0E1233");
    assert_eq!(stops[1].start_ratio, 0.72);
    assert_eq!(stops[1].end_ratio, 1.0);
}

#[test]
fn css_gradient_stops_interpolate_unpositioned_middle_stop() {
    let stops = css_gradient_stops(&["transparent", "#6E5A5D3D", "#463C49A3 100%"])
        .expect("gradient stops should parse");

    assert_eq!(stops.len(), 3);
    assert_eq!(stops[0].start_ratio, 0.0);
    assert_eq!(stops[0].end_ratio, 0.0);
    assert_eq!(stops[1].start_ratio, 0.0);
    assert_eq!(stops[1].end_ratio, 0.5);
    assert_eq!(stops[2].start_ratio, 0.5);
    assert_eq!(stops[2].end_ratio, 1.0);
}

#[test]
fn css_gradient_stops_expand_single_position_head_stop_from_zero() {
    let stops = css_gradient_stops(&["#C4FFFF4D 21%", "transparent 21% 36%"])
        .expect("gradient stops should parse");

    assert_eq!(stops.len(), 2);
    assert_eq!(stops[0].start_ratio, 0.0);
    assert_eq!(stops[0].end_ratio, 0.21);
    assert_eq!(stops[1].start_ratio, 0.21);
    assert_eq!(stops[1].end_ratio, 0.36);
}

#[test]
fn css_simple_radial_gradient_ring_overlay_extracts_ring_band() {
    let ring = css_simple_radial_gradient_ring_overlay(
        "radial-gradient(circle, transparent 33%, #4A688C8C 34% 36%, transparent 37%)",
    )
    .expect("ring gradient should produce a ring overlay");

    assert_eq!(ring.color, "#4A688C8C");
    assert!(ring.width > 0.6 && ring.width < 0.9);
    assert!(ring.height > 0.6 && ring.height < 0.9);
    assert!(ring.border_width > 0.01 && ring.border_width < 0.1);
}

#[test]
fn css_simple_gradient_bands_split_contiguous_non_transparent_color_stops() {
    let stops = css_gradient_stops(&[
        "transparent 0 54%",
        "#6E5A5D3D 64%",
        "#463C49A3 100%",
    ])
    .expect("gradient stops should parse");

    let bands = css_simple_gradient_bands_from_stops(&stops);
    assert_eq!(bands.len(), 2);

    assert_eq!(bands[0].color, "#6E5A5D3D");
    assert_eq!(bands[0].start_ratio, 0.54);
    assert_eq!(bands[0].end_ratio, 0.64);

    assert_eq!(bands[1].color, "#463C49A3");
    assert_eq!(bands[1].start_ratio, 0.64);
    assert_eq!(bands[1].end_ratio, 1.0);
}

#[test]
fn css_simple_gradient_bands_keep_terminal_color_for_fully_opaque_gradients() {
    let stops = css_gradient_stops(&["#6d5a3d", "#2d2119"])
        .expect("gradient stops should parse");

    let bands = css_simple_gradient_bands_from_stops(&stops);
    assert!(bands.len() >= 2);
    assert_eq!(bands[0].color.to_ascii_uppercase(), "#6D5A3D");
    let last = bands.last().expect("should have trailing band");
    assert_eq!(last.color.to_ascii_uppercase(), "#2D2119");
    assert!((last.end_ratio - 1.0).abs() < 0.01);
}

#[test]
fn css_box_shadow_layers_split_multiple_shadow_entries() {
    let shadows = css_box_shadow_layers(
        "inset 0 0 0 3px #FFE7AA44, inset 0 -8px 14px #231A1840, 0 6px 16px #120C0F3D",
    );

    assert_eq!(shadows.len(), 3);
    assert!(shadows[0].inset);
    assert!(shadows[1].inset);
    assert!(!shadows[2].inset);
    assert_eq!(shadows[2].offset_y.as_deref(), Some("6px"));
    assert_eq!(shadows[2].blur_radius.as_deref(), Some("16px"));
}

#[test]
fn apply_box_shadow_fallback_keeps_primary_shadow_and_adds_helper_layers() {
    let mut node = bui_node("button", BuiNodeType::Button);
    node.visuals.border_radius = Some("999px".to_string());

    apply_box_shadow_fallback(
        &mut node,
        "inset 0 0 0 3px #FFE7AA44, 0 6px 16px #120C0F3D, inset 0 -8px 14px #231A1840",
    );

    let primary = node
        .visuals
        .box_shadow
        .as_ref()
        .expect("primary shadow should exist");
    assert!(!primary.inset);
    assert_eq!(primary.offset_y.as_deref(), Some("6px"));

    let helper_layers = node
        .children
        .iter()
        .filter(|child| {
            child
                .custom_tags
                .iter()
                .any(|tag| tag == "css-box-shadow-layer")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 2);
    assert!(helper_layers.iter().all(|child| child.visuals.box_shadow.is_some()));
    assert!(
        helper_layers
            .iter()
            .all(|child| child.visuals.border_radius.as_deref() == Some("999px"))
    );
}

#[test]
fn filter_drop_shadow_adds_a_helper_shadow_layer_without_overwriting_box_shadow() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .icon {
        width: 72px;
        height: 72px;
        border-radius: 999px;
        background: #6A5848;
        box-shadow: 0 6px 16px #120C0F3D;
        filter: drop-shadow(0 2px 0 #3D2A1A8F);
      }
    </style>
    <main class="game-stage">
      <div class="icon"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let icon = find_bui_node(&document.root, "icon");

    let primary = icon
        .visuals
        .box_shadow
        .as_ref()
        .expect("primary box shadow should exist");
    assert_eq!(primary.offset_y.as_deref(), Some("6px"));

    let helper_layers = icon
        .children
        .iter()
        .filter(|child| {
            child
                .custom_tags
                .iter()
                .any(|tag| tag == "css-filter-drop-shadow")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 1);

    let helper_shadow = helper_layers[0]
        .visuals
        .box_shadow
        .as_ref()
        .expect("filter shadow layer should carry a box shadow");
    assert_eq!(helper_shadow.offset_y.as_deref(), Some("2px"));
    assert_eq!(helper_shadow.color.as_deref(), Some("#3D2A1A8F"));
}

#[test]
fn filter_blur_adds_a_helper_shadow_layer() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .glow {
        width: 120px;
        height: 64px;
        border-radius: 999px;
        background: #D8B46C4D;
        filter: blur(8px);
      }
    </style>
    <main class="game-stage">
      <div class="glow"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let glow = find_bui_node(&document.root, "glow");

    let blur_layers = glow
        .children
        .iter()
        .filter(|child| {
            child
                .custom_tags
                .iter()
                .any(|tag| tag == "css-filter-blur")
        })
        .collect::<Vec<_>>();
    assert_eq!(blur_layers.len(), 1);

    let blur_shadow = blur_layers[0]
        .visuals
        .box_shadow
        .as_ref()
        .expect("blur layer should carry a shadow");
    assert_eq!(blur_shadow.blur_radius.as_deref(), Some("32px"));
    assert_eq!(blur_shadow.spread_radius.as_deref(), Some("12px"));
    assert_eq!(blur_shadow.color.as_deref(), Some("#D8B46CA6"));
}

#[test]
fn mix_blend_mode_multiply_darkens_filter_helper_shadow_layers() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .glow {
        width: 120px;
        height: 64px;
        border-radius: 999px;
        background: #D8B46C4D;
        filter: blur(8px);
        mix-blend-mode: multiply;
      }
    </style>
    <main class="game-stage">
      <div class="glow"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let glow = find_bui_node(&document.root, "glow");

    let blur_layer = glow
        .children
        .iter()
        .find(|child| {
            child
                .custom_tags
                .iter()
                .any(|tag| tag == "css-filter-blur")
        })
        .expect("blur helper layer should exist");

    let blur_shadow = blur_layer
        .visuals
        .box_shadow
        .as_ref()
        .expect("blur layer should carry a shadow");
    assert_eq!(blur_shadow.color.as_deref(), Some("#A88C5492"));
}

#[test]
fn filter_multiple_drop_shadows_add_multiple_helper_layers() {
    let html = r#"
    <style>
      .game-stage {
        width: 320px;
        height: 180px;
      }
      .cutout {
        width: 120px;
        height: 160px;
        background: #D7D1C6D8;
        filter:
          drop-shadow(24px 24px 0 #3D42543B)
          drop-shadow(0 24px 20px #2C21174D);
      }
    </style>
    <main class="game-stage">
      <div class="cutout"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let cutout = find_bui_node(&document.root, "cutout");

    let helper_layers = cutout
        .children
        .iter()
        .filter(|child| {
            child
                .custom_tags
                .iter()
                .any(|tag| tag == "css-filter-drop-shadow")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 2);

    let first_shadow = helper_layers[0]
        .visuals
        .box_shadow
        .as_ref()
        .expect("first drop shadow should exist");
    let second_shadow = helper_layers[1]
        .visuals
        .box_shadow
        .as_ref()
        .expect("second drop shadow should exist");

    assert_eq!(first_shadow.offset_x.as_deref(), Some("0"));
    assert_eq!(first_shadow.offset_y.as_deref(), Some("24px"));
    assert_eq!(first_shadow.blur_radius.as_deref(), Some("20px"));

    assert_eq!(second_shadow.offset_x.as_deref(), Some("24px"));
    assert_eq!(second_shadow.offset_y.as_deref(), Some("24px"));
    assert_eq!(second_shadow.blur_radius.as_deref(), Some("0"));
}

#[test]
fn semantic_svg_fallback_uses_parent_id_before_path_signature() {
    let parent = BuiNode {
        id: "skill_button_2".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec!["class:skill-button".to_string()],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "♛");
    assert_eq!(spec.font_size, Some(22.0));
    assert_eq!(spec.color, "#F6ECDD");
}

#[test]
fn semantic_svg_fallback_uses_data_skill_tags() {
    let parent = BuiNode {
        id: "skill_button_dynamic".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec![
            "class:skill-button".to_string(),
            "data-skill:军团号令".to_string(),
        ],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "♛");
    assert_eq!(spec.color, "#F6ECDD");
}

#[test]
fn semantic_svg_fallback_uses_data_equip_tags() {
    let parent = BuiNode {
        id: "equip_slot_dynamic".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec![
            "class:equip-slot".to_string(),
            "data-equip:鹰眼徽章".to_string(),
        ],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "◎");
    assert_eq!(spec.color, "#F3E3C6");
}

#[test]
fn semantic_svg_fallback_supports_indexed_id_patterns() {
    let parent = BuiNode {
        id: "equip_slot_4".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec!["class:equip-slot".to_string()],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let spec = semantic_svg_fallback_spec(&parent).expect("semantic fallback should exist");
    assert_eq!(spec.icon, "♞");
    assert_eq!(spec.font_size, Some(22.0));
    assert_eq!(spec.color, "#F3E3C6");
}

#[test]
fn svg_shape_fallback_recognizes_crosshair_badge_icons() {
    let parent = BuiNode {
        id: "equip_slot_dynamic".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec!["class:equip-slot".to_string()],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40">
            <circle cx="20" cy="20" r="14" fill="none" stroke="currentColor" stroke-width="3"/>
            <path d="M20 8v24M8 20h24M15 20a5 5 0 0 0 10 0 5 5 0 0 0-10 0Z" fill="none" stroke="currentColor" stroke-width="3"/>
          </svg>"#,
    )
    .expect("svg should parse");

    let icon = svg_fallback_icon(&parent, svg.root_element()).expect("shape fallback should exist");
    assert_eq!(icon, "◎");
}

#[test]
fn svg_shape_fallback_recognizes_shield_icons() {
    let parent = BuiNode {
        id: "equip_slot_dynamic".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec!["class:equip-slot".to_string()],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40">
            <path d="M20 4 32 9v9c0 8-5 14-12 18C13 32 8 26 8 18V9Z" fill="none" stroke="currentColor" stroke-width="3"/>
            <path d="M14 14h12M14 20h12" stroke="currentColor" stroke-width="3"/>
          </svg>"#,
    )
    .expect("svg should parse");

    let icon = svg_fallback_icon(&parent, svg.root_element()).expect("shape fallback should exist");
    assert_eq!(icon, "⛨");
}

#[test]
fn svg_shape_fallback_recognizes_scroll_skill_icons() {
    let parent = BuiNode {
        id: "skill_button_dynamic".to_string(),
        node_type: BuiNodeType::Button,
        custom_tags: vec!["class:skill-button".to_string()],
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    };

    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 36 36">
            <path d="M8 6h17c2 0 4 2 4 4v20H11c-2 0-4-2-4-4V7c0-.6.4-1 1-1Zm5 6v3h11v-3Zm0 6v3h9v-3Z" fill="currentColor"/>
          </svg>"#,
    )
    .expect("svg should parse");

    let icon = svg_fallback_icon(&parent, svg.root_element()).expect("shape fallback should exist");
    assert_eq!(icon, "▤");
}

#[test]
fn gradient_two_stop_produces_transition_bands() {
    let bands = css_simple_gradient_bands_from_stops(
        &css_gradient_stops(&["#F0E0C0", "#6A4A2A"]).expect("stops should parse"),
    );
    assert!(bands.len() >= 3);
    assert_eq!(bands[0].color.to_ascii_uppercase(), "#F0E0C0");
    let mid = &bands[bands.len() / 2];
    let blended = blend_hex_colors("#F0E0C0", "#6A4A2A", 0.5);
    assert!(blended.is_some());
    assert_eq!(mid.color.to_ascii_uppercase(), blended.unwrap().to_ascii_uppercase());
    assert_eq!(bands.last().unwrap().color.to_ascii_uppercase(), "#6A4A2A");
    assert!((bands.last().unwrap().end_ratio - 1.0).abs() < 0.01);
}

#[test]
fn blend_hex_colors_produces_correct_intermediate_colors() {
    let mid = blend_hex_colors("#FFFFFF", "#000000", 0.5).expect("blend should work");
    assert_eq!(mid, "#808080");

    let quarter = blend_hex_colors("#FF0000", "#0000FF", 0.25).expect("blend should work");
    assert_eq!(quarter.to_ascii_uppercase(), "#BF0040");

    let with_alpha = blend_hex_colors("#FF000080", "#00FF0080", 0.5).expect("blend should work with alpha");
    assert!(with_alpha.to_ascii_uppercase().starts_with("#8080"));
}

#[test]
fn css_property_support_matrix_classifies_p0_properties() {
    let p0_properties = [
        "display", "position", "width", "height", "z-index",
        "background-color", "color", "font-size", "font-family",
        "line-height", "white-space", "aspect-ratio",
    ];
    for prop in p0_properties {
        let info = css_property_info(prop);
        assert_eq!(info.level, CssPropertySupportLevel::P0);
    }
}

#[test]
fn css_property_support_matrix_classifies_p1_properties() {
    let p1_properties = ["background", "box-shadow", "filter"];
    for prop in p1_properties {
        let info = css_property_info(prop);
        assert_eq!(info.level, CssPropertySupportLevel::P1);
    }
}

#[test]
fn css_property_support_matrix_classifies_p2_properties() {
    let p2_properties = ["mask-image", "clip-path", "mix-blend-mode"];
    for prop in p2_properties {
        let info = css_property_info(prop);
        assert_eq!(info.level, CssPropertySupportLevel::P2);
    }
}

#[test]
fn css_effect_fallback_registry_documents_all_fallback_entries() {
    let registry = css_effect_fallback_registry();
    assert!(registry.len() >= 10);
    let gradient_entry = registry.iter().find(|e| e.css_property == "background (gradient)");
    assert!(gradient_entry.is_some());
    assert_eq!(gradient_entry.unwrap().helper_tag, "css-gradient-overlay");
    let mask_entry = registry.iter().find(|e| e.css_property == "mask-image: linear-gradient(...)");
    assert!(mask_entry.is_some());
    assert_eq!(mask_entry.unwrap().helper_tag, "css-mask-fade");
    let clip_entry = registry.iter().find(|e| e.css_property == "clip-path: polygon(...)");
    assert!(clip_entry.is_some());
    assert_eq!(clip_entry.unwrap().helper_tag, "css-clip-contour");
}
