use super::*;

#[test]
fn css_multiply_blend_fallback_color_darkens_and_softens_alpha() {
    assert_eq!(
        css_multiply_blend_fallback_color("#80A0C080"),
        Some("#648AB17A".to_string())
    );
    assert_eq!(
        css_multiply_blend_fallback_color("#C8E4F250"),
        Some("#A0C7E04D".to_string())
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
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-gradient-overlay")
        })
        .filter_map(|child| child.style.visuals.background_color.as_deref())
        .collect::<Vec<_>>();

    assert!(overlay_colors.iter().any(|color| color.starts_with("#3C")));
    assert!(overlay_colors.iter().any(|color| color.starts_with("#46")));
}

#[test]
fn multiply_scene_wash_linear_overlays_stay_conservative() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 160px; position: relative; }
      .wash {
        position: absolute;
        inset: 0;
        background:
          radial-gradient(circle at 18% 12%, oklch(84% 0.072 235 / 0.72), transparent 25%),
          linear-gradient(90deg, oklch(86% 0.064 230 / 0.16) 0 48%, transparent 59%),
          linear-gradient(90deg, transparent 0 54%, oklch(37% 0.042 53 / 0.24) 64%, oklch(28% 0.032 47 / 0.64) 100%);
        mix-blend-mode: multiply;
      }
    </style>
    <main class="game-stage"><div class="wash"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let wash = find_bui_node(&document.root, "wash");

    let conservative_linear_alphas = wash
        .children
        .iter()
        .filter(|child| child.markers.iter().any(|tag| tag == "css-gradient-overlay"))
        .filter(|child| child.style.visuals.border_radius.is_none())
        .filter(|child| {
            child.layout.styles.top.as_deref() == Some("0") && child.layout.styles.bottom.as_deref() == Some("0")
        })
        .filter_map(|child| child.style.visuals.background_color.as_deref())
        .filter_map(css_hex_rgba)
        .map(|(_, _, _, alpha)| alpha)
        .collect::<Vec<_>>();

    assert!(conservative_linear_alphas.iter().any(|alpha| *alpha <= 0.16));
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
      .game-stage { width: 320px; height: 180px; position: relative; }
      .panel {
        position: absolute;
        inset: 0;
        background: linear-gradient(90deg, #6A5848 0 55%, transparent 70%), #7A6A5A;
        border: 1px solid #9A8A7A;
        filter: brightness(1.08) saturate(1.04) contrast(1.02);
      }
    </style>
    <main class="game-stage"><div class="panel"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    let overlay_colors = panel
        .children
        .iter()
        .filter(|child| child.markers.iter().any(|tag| tag == "css-gradient-overlay"))
        .filter_map(|child| child.style.visuals.background_color.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(panel.style.visuals.background_color.as_deref(), Some("#847260"));
    assert_eq!(panel.style.visuals.border_color.as_deref(), Some("#A79583"));
    assert!(overlay_colors.contains(&"#735E4C"));
}