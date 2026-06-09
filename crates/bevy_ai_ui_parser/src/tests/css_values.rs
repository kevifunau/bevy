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
