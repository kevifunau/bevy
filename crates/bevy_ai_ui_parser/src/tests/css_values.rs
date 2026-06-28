use super::shared::find_bui_node;
use crate::core::opendesign::html::opendesign_html_to_bui_document;
use crate::core::style::css_parser::parse_text_line_height;
use bevy_text::LineHeight;

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
        .content
        .text
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
        .content
        .text
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
        .content
        .text
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
        .content
        .text
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
        .content
        .text
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
        .content
        .text
        .as_ref()
        .expect("span text should have text config");

    assert_eq!(text_config.font_weight, Some(700));
}

#[test]
fn opendesign_font_shorthand_inherit_keeps_inherited_button_text_styles() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
        font-size: 24px;
        font-weight: 700;
        font-family: arial, sans-serif;
      }
      .login-button {
        font: inherit;
      }
    </style>
    <main class="game-stage">
      <button class="login-button">游客登录</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "login_button");
    assert_eq!(button.layout.styles.display.as_deref(), Some("flex"));
    assert_eq!(button.layout.styles.align_items.as_deref(), Some("center"));
    assert_eq!(
        button.layout.styles.justify_content.as_deref(),
        Some("center")
    );

    let text_node = find_bui_node(&document.root, "login_button_text_1");
    let text_config = text_node
        .content
        .text
        .as_ref()
        .expect("button text should have text config");

    assert_eq!(text_config.font_size, 24.0);
    assert_eq!(text_config.font_weight, Some(700));
    assert_eq!(text_config.linebreak.as_deref(), Some("word_or_character"));
    assert_eq!(text_config.line_height.as_deref(), Some("1"));
}

#[test]
fn opendesign_chinese_text_defaults_to_word_or_character_wrapping() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .copy {
        width: 120px;
        font-size: 20px;
        white-space: normal;
      }
    </style>
    <main class="game-stage">
      <div class="copy">萌宠解谜登录页面</div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text_node = find_bui_node(&document.root, "copy_text_1");
    let text_config = text_node
        .content
        .text
        .as_ref()
        .expect("copy text should have text config");

    assert_eq!(text_config.linebreak.as_deref(), Some("word_or_character"));
}

#[test]
fn opendesign_visibility_hidden_maps_to_ir_visibility() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .tip-overlay {
        visibility: hidden;
      }
    </style>
    <main class="game-stage">
      <div class="tip-overlay" id="tip_panel"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let tip_panel = find_bui_node(&document.root, "tip_panel");

    assert_eq!(
        tip_panel.layout.styles.visibility.as_deref(),
        Some("hidden")
    );
}

#[test]
fn opendesign_rgb_and_rgba_colors_compile_to_hex_colors() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .panel {
        background: rgba(20, 20, 35, 0.85);
        border: 2px solid rgba(255, 215, 0, 0.3);
      }
      .label {
        color: rgb(255, 215, 0);
      }
    </style>
    <main class="game-stage">
      <div class="panel">
        <span class="label">提示</span>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");
    let label_text = find_bui_node(&document.root, "label_text_1");

    assert_eq!(
        panel.style.visuals.background_color.as_deref(),
        Some("#141423D9")
    );
    assert_eq!(
        panel.style.visuals.border_color.as_deref(),
        Some("#FFD7004D")
    );
    assert_eq!(
        label_text
            .content
            .text
            .as_ref()
            .expect("label should have text")
            .font_color,
        "#FFD700"
    );
}

#[test]
fn login_scene_index_html_preserves_panel_visuals_and_hidden_tip() {
    let html = include_str!("../../examples/login_scene/webgameui/index.html");

    let document = opendesign_html_to_bui_document(html).expect("login scene HTML should compile");
    let login_panel = find_bui_node(&document.root, "login_panel");
    let tip_panel = find_bui_node(&document.root, "tip_panel");

    assert_eq!(
        login_panel.style.visuals.background_color.as_deref(),
        Some("#141423D9")
    );
    assert_eq!(
        login_panel.style.visuals.border_color.as_deref(),
        Some("#FFD7004D")
    );
    assert_eq!(tip_panel.layout.styles.display.as_deref(), Some("none"));
}

#[test]
fn opendesign_implicit_grid_falls_back_to_vertical_flex() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .actions {
        display: grid;
        gap: 40px;
      }
      .action {
        width: 342px;
        height: 74px;
      }
    </style>
    <main class="game-stage">
      <section class="actions">
        <button class="action">游客登录</button>
        <button class="action">微信登录</button>
      </section>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let actions = find_bui_node(&document.root, "actions");

    assert_eq!(actions.layout.styles.display.as_deref(), Some("flex"));
    assert_eq!(
        actions.layout.styles.flex_direction.as_deref(),
        Some("column")
    );
    assert_eq!(actions.layout.styles.row_gap.as_deref(), Some("40px"));
    assert_eq!(actions.layout.styles.column_gap.as_deref(), Some("40px"));
}

#[test]
fn opendesign_background_image_padding_moves_to_text_child() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .login-button {
        width: 342px;
        height: 74px;
        display: flex;
        align-items: center;
        justify-content: center;
        background-image: url("Asset/wechat-login-button.png");
        padding-left: 50px;
      }
    </style>
    <main class="game-stage">
      <button class="login-button">微信登录</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "login_button");
    assert!(button.content.image.is_some());
    assert_eq!(button.layout.styles.padding_left.as_deref(), None);

    let text_node = find_bui_node(&document.root, "login_button_text_1");
    assert_eq!(text_node.layout.styles.margin_left.as_deref(), Some("50px"));
}

#[test]
fn opendesign_background_size_contain_maps_to_auto_image_mode() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .portrait {
        width: 200px;
        height: 160px;
        background-image: url("Asset/cat-logo.png");
        background-size: contain;
        background-position: center;
      }
    </style>
    <main class="game-stage">
      <div class="portrait"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let portrait = find_bui_node(&document.root, "portrait");
    let image = portrait
        .content
        .image
        .as_ref()
        .expect("portrait should keep background image config");

    assert_eq!(image.image_mode.as_deref(), Some("auto"));
    assert_eq!(image.background_size.as_deref(), Some("contain"));
    assert_eq!(image.background_position.as_deref(), Some("center"));
}

#[test]
fn opendesign_background_size_before_image_is_preserved() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .portrait {
        width: 200px;
        height: 160px;
        background-size: contain;
        background-position: center;
        background-image: url("Asset/cat-logo.png");
      }
    </style>
    <main class="game-stage">
      <div class="portrait"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let portrait = find_bui_node(&document.root, "portrait");
    let image = portrait
        .content
        .image
        .as_ref()
        .expect("portrait should keep background image config");

    assert_eq!(image.image_mode.as_deref(), Some("auto"));
    assert_eq!(image.background_size.as_deref(), Some("contain"));
    assert_eq!(image.background_position.as_deref(), Some("center"));
}

#[test]
fn opendesign_background_full_stretch_preserves_stretch_mode() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .button-art {
        width: 342px;
        height: 74px;
        background-image: url("Asset/visitor-login-button.png");
        background-size: 100% 100%;
      }
    </style>
    <main class="game-stage">
      <div class="button-art"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button_art = find_bui_node(&document.root, "button_art");
    let image = button_art
        .content
        .image
        .as_ref()
        .expect("button art should keep background image config");

    assert_eq!(image.image_mode.as_deref(), Some("stretch"));
    assert_eq!(image.background_size.as_deref(), Some("100% 100%"));
}

#[test]
fn opendesign_box_sizing_content_box_is_preserved() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .panel {
        width: 200px;
        padding: 24px;
        box-sizing: content-box;
      }
    </style>
    <main class="game-stage">
      <section class="panel"></section>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    assert_eq!(
        panel.layout.styles.box_sizing.as_deref(),
        Some("content-box")
    );
    assert_eq!(panel.layout.styles.width.as_deref(), Some("200px"));
    assert_eq!(panel.layout.styles.padding.as_deref(), Some("24px"));
}

#[test]
fn opendesign_layout_shorthands_are_preserved() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .panel {
        display: block;
        flex: 2 0 160px;
        gap: 12px 20px;
        margin-inline: 8px 16px;
        margin-block: 4px 10px;
        padding-inline: 6px 14px;
        padding-block: 3px 9px;
      }
    </style>
    <main class="game-stage">
      <section class="panel"></section>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    assert_eq!(panel.layout.styles.display.as_deref(), Some("block"));
    assert_eq!(panel.layout.styles.flex_grow.as_deref(), Some("2"));
    assert_eq!(panel.layout.styles.flex_shrink.as_deref(), Some("0"));
    assert_eq!(panel.layout.styles.flex_basis.as_deref(), Some("160px"));
    assert_eq!(panel.layout.styles.row_gap.as_deref(), Some("12px"));
    assert_eq!(panel.layout.styles.column_gap.as_deref(), Some("20px"));
    assert_eq!(panel.layout.styles.margin_left.as_deref(), Some("8px"));
    assert_eq!(panel.layout.styles.margin_right.as_deref(), Some("16px"));
    assert_eq!(panel.layout.styles.margin_top.as_deref(), Some("4px"));
    assert_eq!(panel.layout.styles.margin_bottom.as_deref(), Some("10px"));
    assert_eq!(panel.layout.styles.padding_left.as_deref(), Some("6px"));
    assert_eq!(panel.layout.styles.padding_right.as_deref(), Some("14px"));
    assert_eq!(panel.layout.styles.padding_top.as_deref(), Some("3px"));
    assert_eq!(panel.layout.styles.padding_bottom.as_deref(), Some("9px"));
}

#[test]
fn opendesign_place_and_overflow_axes_are_preserved() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .panel {
        display: grid;
        place-items: center end;
        place-content: start center;
        place-self: stretch center;
        overflow-x: hidden;
        overflow-y: auto;
      }
    </style>
    <main class="game-stage">
      <section class="panel"></section>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    assert_eq!(panel.layout.styles.align_items.as_deref(), Some("center"));
    assert_eq!(panel.layout.styles.justify_items.as_deref(), Some("end"));
    assert_eq!(panel.layout.styles.align_content.as_deref(), Some("start"));
    assert_eq!(
        panel.layout.styles.justify_content.as_deref(),
        Some("center")
    );
    assert_eq!(panel.layout.styles.align_self.as_deref(), Some("stretch"));
    assert_eq!(panel.layout.styles.justify_self.as_deref(), Some("center"));
    assert_eq!(
        panel.layout.styles.overflow.as_deref(),
        Some("hidden scroll")
    );
}

#[test]
fn opendesign_grid_placement_and_auto_tracks_are_preserved() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .panel {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        grid-auto-rows: min-content;
        grid-auto-columns: fit-content(120px);
      }
      .hero {
        grid-column: 2;
        grid-row: span 2;
      }
      .meter {
        grid-area: 1 / 1 / span 2 / span 1;
      }
    </style>
    <main class="game-stage">
      <section class="panel">
        <div class="hero"></div>
        <div class="meter"></div>
      </section>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");
    let hero = find_bui_node(&document.root, "hero");
    let meter = find_bui_node(&document.root, "meter");

    assert_eq!(
        panel.layout.styles.grid_template_columns.as_deref(),
        Some("flex(2, 1)")
    );
    assert_eq!(
        panel.layout.styles.grid_auto_rows.as_deref(),
        Some("min_content")
    );
    assert_eq!(
        panel.layout.styles.grid_auto_columns.as_deref(),
        Some("fit_content_px(120)")
    );
    assert_eq!(hero.layout.styles.grid_column.as_deref(), Some("start(2)"));
    assert_eq!(hero.layout.styles.grid_row.as_deref(), Some("span(2)"));
    assert_eq!(meter.layout.styles.grid_column.as_deref(), Some("span(1)"));
    assert_eq!(meter.layout.styles.grid_row.as_deref(), Some("span(2)"));
}

#[test]
fn opendesign_stage_fit_semantics_and_marker_are_preserved() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
        bui-stage-fit: contain;
      }
    </style>
    <main class="game-stage"></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let stage = find_bui_node(&document.root, "stage_root");
    let fit = stage
        .semantics
        .stage_fit
        .as_ref()
        .expect("stage fit semantics should be emitted");

    assert_eq!(fit.design_width, 640.0);
    assert_eq!(fit.design_height, 360.0);
    assert_eq!(fit.mode, "contain");
    assert!(stage
        .markers
        .iter()
        .any(|marker| marker == "bui-stage-fit:640x360"));
}

#[test]
fn opendesign_bevy_ui_root_viewport_units_emit_stage_fit() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
    </style>
    <main class="bevy-ui-root"></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let stage = find_bui_node(&document.root, "stage_root");
    let fit = stage
        .semantics
        .stage_fit
        .as_ref()
        .expect("bevy-ui-root viewport units should emit stage fit semantics");

    assert_eq!(fit.design_width, 1920.0);
    assert_eq!(fit.design_height, 1080.0);
    assert_eq!(stage.layout.styles.width.as_deref(), Some("1920px"));
    assert_eq!(stage.layout.styles.height.as_deref(), Some("1080px"));
    assert!(stage
        .markers
        .iter()
        .any(|marker| marker == "bui-stage-fit:1920x1080"));
}

#[test]
fn opendesign_attribute_selector_applies_to_initial_state() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
      .nav-button {
        background-image: url("Asset/menu-button-idle.png");
      }
      .nav-button[aria-current="page"] {
        background-image: url("Asset/menu-button-active.png");
      }
    </style>
    <main class="bevy-ui-root">
      <button id="nav_garage" class="nav-button" aria-current="page">GARAGE</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "nav_garage");

    assert_eq!(
        button
            .content
            .image
            .as_ref()
            .map(|image| image.texture_path.as_str()),
        Some("Asset/menu-button-idle.png")
    );
    assert!(button
        .markers
        .iter()
        .any(|marker| marker == "initial-state:selected"));
    assert_eq!(
        button
            .state_visuals
            .get("selected")
            .and_then(|visual| visual.image.as_ref())
            .map(|image| image.texture_path.as_str()),
        Some("Asset/menu-button-active.png")
    );
}

#[test]
fn opendesign_body_text_color_inherits_into_stage_fragment_text() {
    let html = r#"
    <style>
      :root {
        --text: #f4fbff;
      }
      body {
        color: var(--text);
      }
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
    </style>
    <main class="bevy-ui-root">
      <strong id="player_name">KAI_SPEEDSTER</strong>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text = find_bui_node(&document.root, "player_name_text_1");

    assert_eq!(
        text.content
            .text
            .as_ref()
            .map(|text| text.font_color.to_ascii_lowercase()),
        Some("#f4fbff".to_string())
    );
}

#[test]
fn opendesign_data_binding_attaches_to_direct_text_content() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
    </style>
    <main class="bevy-ui-root">
      <strong id="race_status_text" data-binding="race.status">READY TO RACE</strong>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let text = find_bui_node(&document.root, "race_status_text_text_1");

    assert_eq!(text.bindings.len(), 1);
    assert_eq!(text.bindings[0].source, "race.status");
    assert_eq!(text.bindings[0].target, "text.content");
}

#[test]
fn opendesign_data_bui_actions_script_builds_interaction_model() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
    </style>
    <main class="bevy-ui-root">
      <button id="nav_events" data-action="open-events">EVENTS</button>
      <strong id="race_status_text" data-binding="race.status">READY TO RACE</strong>
    </main>
    <script type="application/json" data-bui-actions>
      {
        "actions": {
          "open-events": [
            {
              "op": "set-selected",
              "group": ["nav_garage", "nav_events"],
              "target": "nav_events",
              "state": "selected"
            },
            {
              "op": "set-text",
              "binding": "race.status",
              "node": "race_status_text_text_1",
              "value": "EVENTS OPENED"
            },
            {
              "op": "set-binding",
              "source": "race.loading",
              "value_type": "bool",
              "value": "true"
            },
            {
              "op": "set-visible",
              "target": "events_panel",
              "value": "visible"
            },
            {
              "op": "delay",
              "ms": 900
            }
          ]
        }
      }
    </script>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let steps = document
        .interaction_model
        .actions
        .get("open-events")
        .expect("open-events action should be parsed");

    assert_eq!(steps.len(), 5);
    assert_eq!(steps[0].op, "set-selected");
    assert_eq!(
        steps[0].group.as_deref(),
        Some(&["nav_garage".to_string(), "nav_events".to_string()][..])
    );
    assert_eq!(steps[0].target.as_deref(), Some("nav_events"));
    assert_eq!(steps[0].state.as_deref(), Some("selected"));
    assert_eq!(steps[1].op, "set-text");
    assert_eq!(steps[1].binding.as_deref(), Some("race.status"));
    assert_eq!(steps[1].node.as_deref(), Some("race_status_text_text_1"));
    assert_eq!(steps[1].value.as_deref(), Some("EVENTS OPENED"));
    assert_eq!(steps[2].op, "set-binding");
    assert_eq!(steps[2].source.as_deref(), Some("race.loading"));
    assert_eq!(steps[2].value_type.as_deref(), Some("bool"));
    assert_eq!(steps[2].value.as_deref(), Some("true"));
    assert_eq!(steps[3].op, "set-visible");
    assert_eq!(steps[3].target.as_deref(), Some("events_panel"));
    assert_eq!(steps[3].value.as_deref(), Some("visible"));
    assert_eq!(steps[4].op, "delay");
    assert_eq!(steps[4].ms, Some(900));
}

#[test]
fn opendesign_window_bui_actions_javascript_builds_interaction_model() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
    </style>
    <main class="bevy-ui-root">
      <button id="play_button" data-action="start-race">PLAY</button>
      <strong id="race_status_text" data-binding="race.status">READY</strong>
    </main>
    <script>
      window.BUI_ACTIONS = {
        "actions": {
          "start-race": [
            {
              "op": "set-text",
              "binding": "race.status",
              "node": "race_status_text_text_1",
              "value": "MATCHMAKING..."
            },
            {
              "op": "delay",
              "ms": 500
            },
            {
              "op": "set-text",
              "binding": "race.status",
              "node": "race_status_text_text_1",
              "value": "READY TO RACE"
            }
          ]
        }
      };
    </script>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let steps = document
        .interaction_model
        .actions
        .get("start-race")
        .expect("start-race action should be parsed from JavaScript");

    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].op, "set-text");
    assert_eq!(steps[0].value.as_deref(), Some("MATCHMAKING..."));
    assert_eq!(steps[1].op, "delay");
    assert_eq!(steps[1].ms, Some(500));
    assert_eq!(steps[2].value.as_deref(), Some("READY TO RACE"));
}

#[test]
fn opendesign_bui_register_actions_javascript_builds_interaction_model() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
    </style>
    <main class="bevy-ui-root">
      <button id="garage_button" data-action="open-garage">GARAGE</button>
    </main>
    <script>
      Bui.registerActions({
        "actions": {
          "open-garage": [
            {
              "op": "set-visible",
              "target": "garage_panel",
              "value": "visible"
            }
          ]
        }
      });
    </script>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let steps = document
        .interaction_model
        .actions
        .get("open-garage")
        .expect("open-garage action should be parsed from JavaScript");

    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].op, "set-visible");
    assert_eq!(steps[0].target.as_deref(), Some("garage_panel"));
    assert_eq!(steps[0].value.as_deref(), Some("visible"));
}

#[test]
fn opendesign_background_repeat_is_preserved_as_no_repeat_fallback() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .tile {
        background-image: url("Asset/tile.png");
        background-size: auto;
        background-repeat: repeat;
      }
    </style>
    <main class="game-stage">
      <div class="tile"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let tile = find_bui_node(&document.root, "tile");
    let image = tile
        .content
        .image
        .as_ref()
        .expect("tile should keep background image config");

    assert_eq!(image.background_repeat.as_deref(), Some("no-repeat"));
    assert_eq!(image.background_size.as_deref(), Some("auto"));
    assert_eq!(image.image_mode.as_deref(), Some("auto"));
}

#[test]
fn opendesign_generic_semantic_normalize_handles_tabs_and_progress() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .selected:hover {
        background: #ffffff;
      }
    </style>
    <main class="game-stage">
      <button class="selected" data-tab-group="menu" data-tab="map" aria-selected="true">Map</button>
      <section data-tab-panel="map"></section>
      <div class="health-progress" role="progressbar" aria-valuenow="72">
        <div class="health-bar"></div>
      </div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let selected = find_bui_node(&document.root, "selected");
    let panel = find_bui_node(&document.root, "section");
    let progress = find_bui_node(&document.root, "health_progress");
    let fill = find_bui_node(&document.root, "health_bar");

    assert_eq!(selected.semantics.tab_group_name.as_deref(), Some("menu"));
    assert_eq!(selected.semantics.tab_value.as_deref(), Some("map"));
    assert!(selected.state_visuals.contains_key("selected"));
    assert!(panel
        .markers
        .iter()
        .any(|marker| marker == "data-tab-panel:map"));
    assert_eq!(
        progress.semantics.progress_binding_source.as_deref(),
        Some("72")
    );
    assert!(fill.semantics.progress_fill);
}

#[test]
fn opendesign_unity_state_selectors_compile_to_runtime_state_visuals() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
      .btn:pressed {
        background: #ff8800;
      }
      .btn:disabled {
        opacity: 0.4;
      }
      .toggle:checked {
        background-image: url("./assets/buttons/toggle_on.png");
      }
    </style>
    <main class="game-stage">
      <button class="btn" disabled="true">Blocked</button>
      <div id="audio_toggle" class="toggle" role="switch" aria-checked="true"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "btn");
    let toggle = find_bui_node(&document.root, "audio_toggle");

    assert_eq!(button.kind, "button");
    assert!(button
        .markers
        .iter()
        .any(|marker| marker == "State_Disabled"));
    assert!(button.state_visuals.contains_key("pressed"));
    assert!(button.state_visuals.contains_key("disabled"));
    assert_eq!(
        button
            .state_visuals
            .get("pressed")
            .and_then(|state| state.visuals.background_color.as_deref()),
        Some("#ff8800")
    );

    assert_eq!(toggle.kind, "toggle");
    assert!(toggle
        .markers
        .iter()
        .any(|marker| marker == "State_Checked"));
    assert!(toggle
        .markers
        .iter()
        .any(|marker| marker == "initial-state:checked"));
    assert_eq!(
        toggle
            .state_visuals
            .get("checked")
            .and_then(|state| state.image.as_ref())
            .map(|image| image.texture_path.as_str()),
        Some("./assets/buttons/toggle_on.png")
    );
}

#[test]
fn opendesign_event_attributes_bind_press_and_hover_actions() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        height: 360px;
      }
    </style>
    <main class="game-stage">
      <button
        id="play_button"
        data-action="start-race"
        data-action-press="confirm-start"
        data-action-hover-enter="preview-start"
        data-action-hover-exit="clear-preview"
      >PLAY</button>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "play_button");
    let actions = button
        .actions
        .iter()
        .map(|action| (action.event.as_str(), action.emit.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        actions,
        vec![
            ("press", "start-race"),
            ("press", "confirm-start"),
            ("hover_enter", "preview-start"),
            ("hover_exit", "clear-preview"),
        ]
    );
}

#[test]
fn opendesign_input_elements_compile_to_text_input_actions_and_binding() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
      .field {
        color: #dff7ff;
        font-size: 18px;
      }
      .field:focus {
        border-color: #66ccff;
      }
    </style>
    <main class="bevy-ui-root">
      <input
        id="account_input"
        class="field"
        type="text"
        name="account"
        value="Racer"
        placeholder="Account"
        data-binding="login.account"
        data-action-change="account.changed"
        data-action-submit="account.submit"
        data-action-focus="account.focus"
        data-action-blur="account.blur"
      />
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let input = find_bui_node(&document.root, "account_input");
    let text = input
        .content
        .text
        .as_ref()
        .expect("input should compile to text config");
    let actions = input
        .actions
        .iter()
        .map(|action| (action.event.as_str(), action.emit.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(input.kind, "text_input");
    assert_eq!(text.content, "Racer");
    assert_eq!(text.placeholder.as_deref(), Some("Account"));
    assert_eq!(text.font_size, 18.0);
    assert_eq!(text.font_color.to_ascii_lowercase(), "#dff7ff");
    assert!(input.state_visuals.contains_key("focused"));
    assert_eq!(input.bindings.len(), 1);
    assert_eq!(input.bindings[0].source, "login.account");
    assert_eq!(input.bindings[0].target, "text.content");
    assert_eq!(
        actions,
        vec![
            ("value_changed", "account.changed"),
            ("submit", "account.submit"),
            ("focus", "account.focus"),
            ("blur", "account.blur"),
        ]
    );
}

#[test]
fn opendesign_slider_elements_compile_to_slider_semantics_and_actions() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
      .volume-slider {
        width: 240px;
        height: 24px;
      }
    </style>
    <main class="bevy-ui-root">
      <input
        id="volume_slider"
        class="volume-slider"
        type="range"
        min="0"
        max="100"
        value="65"
        step="5"
        data-binding="settings.volume"
        data-action-change="volume.changed"
      />
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let slider = find_bui_node(&document.root, "volume_slider");
    let semantics = slider
        .semantics
        .slider
        .as_ref()
        .expect("slider semantics should be present");

    assert_eq!(slider.kind, "slider");
    assert_eq!(semantics.min, 0.0);
    assert_eq!(semantics.max, 100.0);
    assert_eq!(semantics.value, 65.0);
    assert_eq!(semantics.step, Some(5.0));
    assert_eq!(slider.bindings.len(), 1);
    assert_eq!(slider.bindings[0].target, "value");
    assert_eq!(slider.bindings[0].source, "settings.volume");
    assert_eq!(slider.actions.len(), 1);
    assert_eq!(slider.actions[0].event, "value_changed");
    assert_eq!(slider.actions[0].emit, "volume.changed");
}

#[test]
fn opendesign_scroll_view_compiles_to_scroll_semantics_and_action() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
      .inventory-scroll {
        width: 360px;
        height: 220px;
        overflow-y: auto;
      }
    </style>
    <main class="bevy-ui-root">
      <section
        id="inventory_scroll"
        class="inventory-scroll"
        data-scroll-view="true"
        data-scroll-binding="inventory.list"
        data-action-scroll="inventory.scrolled"
      >
        <button data-action="item.select">Sword</button>
      </section>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let scroll_view = find_bui_node(&document.root, "inventory_scroll");
    let semantics = scroll_view
        .semantics
        .scroll_view
        .as_ref()
        .expect("scroll-view semantics should be present");

    assert_eq!(scroll_view.kind, "node");
    assert_eq!(
        scroll_view.layout.styles.overflow.as_deref(),
        Some("scroll_y")
    );
    assert_eq!(semantics.binding_source.as_deref(), Some("inventory.list"));
    assert_eq!(semantics.axis.as_deref(), Some("y"));
    assert_eq!(scroll_view.actions.len(), 1);
    assert_eq!(scroll_view.actions[0].event, "scroll");
    assert_eq!(scroll_view.actions[0].emit, "inventory.scrolled");
}

#[test]
fn opendesign_select_compiles_to_dropdown_semantics_and_actions() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
      .difficulty-select {
        display: flex;
        flex-direction: column;
      }
    </style>
    <main class="bevy-ui-root">
      <select
        id="difficulty_select"
        class="difficulty-select"
        name="difficulty"
        data-binding="settings.difficulty"
      >
        <option value="easy">Easy</option>
        <option value="hard" selected="true" data-action-select="difficulty.changed">Hard</option>
      </select>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let dropdown = find_bui_node(&document.root, "difficulty_select");
    let hard = find_bui_node(&document.root, "option_2");

    assert_eq!(dropdown.kind, "node");
    assert_eq!(
        dropdown.semantics.dropdown_group_name.as_deref(),
        Some("difficulty")
    );
    assert_eq!(
        dropdown.semantics.dropdown_binding_source.as_deref(),
        Some("settings.difficulty")
    );
    assert_eq!(hard.kind, "button");
    assert_eq!(
        hard.semantics.dropdown_group_name.as_deref(),
        Some("difficulty")
    );
    assert_eq!(hard.semantics.dropdown_value.as_deref(), Some("hard"));
    assert_eq!(hard.semantics.dropdown_label.as_deref(), Some("Hard"));
    assert!(hard
        .markers
        .iter()
        .any(|marker| marker == "initial-state:selected"));
    assert_eq!(hard.actions.len(), 1);
    assert_eq!(hard.actions[0].event, "selection_changed");
    assert_eq!(hard.actions[0].emit, "difficulty.changed");
}

#[test]
fn opendesign_toggle_compiles_to_checked_binding_and_change_action() {
    let html = r#"
    <style>
      .bevy-ui-root {
        width: 100vw;
        height: 100vh;
      }
      .audio-toggle:checked {
        background-image: url("./assets/buttons/toggle_on.png");
      }
    </style>
    <main class="bevy-ui-root">
      <input
        id="audio_toggle"
        class="audio-toggle"
        type="checkbox"
        checked="true"
        data-binding="settings.audio_enabled"
        data-action-change="audio.changed"
      />
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let toggle = find_bui_node(&document.root, "audio_toggle");

    assert_eq!(toggle.kind, "toggle");
    assert!(toggle
        .markers
        .iter()
        .any(|marker| marker == "initial-state:checked"));
    assert_eq!(toggle.bindings.len(), 1);
    assert_eq!(toggle.bindings[0].target, "checked");
    assert_eq!(toggle.bindings[0].source, "settings.audio_enabled");
    assert_eq!(toggle.actions.len(), 1);
    assert_eq!(toggle.actions[0].event, "value_changed");
    assert_eq!(toggle.actions[0].emit, "audio.changed");
}
