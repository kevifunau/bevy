use super::shared::find_bui_node;
use crate::core::model::{BuiNodeType, bui_node};
use crate::core::opendesign::html::opendesign_html_to_bui_document;
use crate::core::style::css_parser::apply_css_transform;

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

    assert_eq!(hovered.visuals.background_color.as_deref(), Some("#847261"));
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
fn css_transform_translate_parses_composite_values() {
    let mut node = bui_node("test", BuiNodeType::Node);
    apply_css_transform(&mut node, "translate(-34%, -36%) rotate(12deg)");
    assert_eq!(node.styles.ui_translation.as_deref(), Some("-34% -36%"));
    assert_eq!(node.styles.ui_rotation.as_deref(), Some("12.0deg"));
}

#[test]
fn css_transform_translate_parses_single_axis() {
    let mut node = bui_node("test", BuiNodeType::Node);
    apply_css_transform(&mut node, "translateY(14px)");
    let translation = node
        .styles
        .ui_translation
        .as_deref()
        .expect("translation should exist");
    assert!(translation.contains("14") && translation.starts_with("0px"));
}

#[test]
fn css_transform_state_handles_composite_transform() {
    let html = r#"
        <style>
          .game-stage {
            width: 320px;
            height: 180px;
          }
          .btn {
            color: #fff;
          }
          .btn:active {
            transform: translateY(2px) scale(0.98);
          }
        </style>
        <main class="game-stage">
          <button class="btn">Click</button>
        </main>
        "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let button = find_bui_node(&document.root, "btn");
    let pressed = button
        .state_visuals
        .get("pressed")
        .expect("pressed state should exist");
    let translation = pressed
        .styles
        .ui_translation
        .as_deref()
        .expect("translation should exist");
    assert!(translation.contains("2") && translation.starts_with("0px"));
    assert_eq!(pressed.styles.ui_scale.as_deref(), Some("0.98 0.98"));
    assert_eq!(
        button
            .state_visuals
            .get("normal")
            .and_then(|s| s.styles.ui_scale.as_deref()),
        Some("1 1")
    );
}

#[test]
fn diagonal_gradient_overlay_sets_ui_rotation_on_band() {
    let html = r#"
        <style>
          .game-stage {
            width: 200px;
            height: 100px;
          }
          .box {
            background: linear-gradient(135deg, #ff0 0 30%, #0ff 70% 100%);
          }
        </style>
        <main class="game-stage">
          <div class="box"></div>
        </main>
        "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let overlay = find_bui_node(&document.root, "box_gradient_overlay");
    assert_eq!(
        overlay.styles.ui_rotation.as_deref(),
        Some("45.0deg"),
        "135deg gradient should rotate overlay by 45deg (135 - 90)"
    );
}

#[test]
fn diagonal_keyword_gradient_sets_ui_rotation() {
    let html = r#"
        <style>
          .game-stage {
            width: 200px;
            height: 100px;
          }
          .box {
            background: linear-gradient(to bottom right, #ff0 0 30%, #0ff 70% 100%);
          }
        </style>
        <main class="game-stage">
          <div class="box"></div>
        </main>
        "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let overlay = find_bui_node(&document.root, "box_gradient_overlay");
    assert_eq!(
        overlay.styles.ui_rotation.as_deref(),
        Some("45.0deg"),
        "to bottom right keyword should produce 135deg equivalent rotation (135 - 90)"
    );
}

#[test]
fn css_opacity_stores_ui_opacity_and_adjusts_colors() {
    let html = r#"
        <style>
          .game-stage {
            width: 320px;
            height: 180px;
          }
          .crest {
            background: #ff0000;
            opacity: 0.22;
          }
        </style>
        <main class="game-stage">
          <div class="crest"></div>
        </main>
        "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let crest = find_bui_node(&document.root, "crest");
    assert_eq!(crest.styles.ui_opacity, Some(0.22));
    let bg = crest
        .visuals
        .background_color
        .as_deref()
        .expect("bg should exist");
    assert!(
        bg.ends_with("38"),
        "alpha should be ~0.22*255=56 (0x38), got: {}",
        bg
    );
}

#[test]
fn css_opacity_on_node_without_background_stores_ui_opacity() {
    let html = r#"
        <style>
          .game-stage {
            width: 320px;
            height: 180px;
          }
          .ghost {
            opacity: 0.5;
          }
        </style>
        <main class="game-stage">
          <div class="ghost"></div>
        </main>
        "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let ghost = find_bui_node(&document.root, "ghost");
    assert_eq!(ghost.styles.ui_opacity, Some(0.5));
}
