use super::*;

#[test]
fn css_simple_clip_polygon_contour_extracts_bounds_and_accent() {
    let spec = css_simple_clip_polygon_contour(
        "polygon(35% 0, 68% 4%, 81% 18%, 90% 38%, 80% 64%, 86% 100%, 24% 100%, 31% 70%, 13% 55%, 20% 31%)",
    )
    .expect("clip polygon should parse");

    assert!((spec.left - 0.13).abs() < 0.001);
    assert!((spec.right - 0.10).abs() < 0.001);
    assert!(spec.accent_width >= 0.08 && spec.accent_width <= 0.34);
}

#[test]
fn opendesign_clip_path_polygon_adds_contour_children() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; position: relative; }
      .cutout {
        position: absolute;
        inset: 0;
        background: #d7d1c6;
        border: 1px solid #fff3d6;
        clip-path: polygon(35% 0, 68% 4%, 81% 18%, 90% 38%, 80% 64%, 86% 100%, 24% 100%, 31% 70%, 13% 55%, 20% 31%);
      }
    </style>
    <main class="game-stage"><div class="cutout"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let cutout = find_bui_node(&document.root, "cutout");
    let contour_children = cutout
        .children
        .iter()
        .filter(|child| child.markers.iter().any(|tag| tag == "css-clip-contour"))
        .collect::<Vec<_>>();

    assert_eq!(contour_children.len(), 3);
    assert_eq!(
        cutout.style.visuals.background_color.as_deref(),
        Some("transparent")
    );
}

#[test]
fn css_simple_mask_fade_supports_keyword_and_default_directions() {
    let left_fade =
        css_simple_mask_fade("linear-gradient(to right, transparent 0, black 11%, black 100%)")
            .expect("keyword mask fade should parse");
    assert!(matches!(
        left_fade.direction,
        MaskFadeDirection::LeftToRight
    ));

    let top_fade = css_simple_mask_fade("linear-gradient(transparent 0, black 18%, black 100%)")
        .expect("default-direction mask fade should parse");
    assert!(matches!(top_fade.direction, MaskFadeDirection::TopToBottom));
}

#[test]
fn opendesign_mask_image_none_clears_existing_mask_fade_children() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; position: relative; }
      .panel {
        position: absolute;
        inset: 0;
        background: #c7a97a;
        mask-image: linear-gradient(90deg, transparent 0, black 11%, black 100%);
      }
      .game-stage.panel-open .panel { mask-image: none; }
    </style>
    <main class="game-stage panel-open"><div class="panel"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let panel = find_bui_node(&document.root, "panel");

    let mask_children = panel
        .children
        .iter()
        .filter(|child| child.markers.iter().any(|tag| tag == "css-mask-fade"))
        .count();

    assert_eq!(mask_children, 0);
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
}

#[test]
fn apply_box_shadow_fallback_keeps_primary_shadow_and_adds_helper_layers() {
    let mut node = bui_node("button", "button");
    node.style.visuals.border_radius = Some("999px".to_string());

    apply_box_shadow_fallback(
        &mut node,
        "inset 0 0 0 3px #FFE7AA44, 0 6px 16px #120C0F3D, inset 0 -8px 14px #231A1840",
    );

    let helper_layers = node
        .children
        .iter()
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-box-shadow-layer")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 2);
}

#[test]
fn filter_drop_shadow_adds_a_helper_shadow_layer_without_overwriting_box_shadow() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; }
      .icon {
        width: 72px;
        height: 72px;
        border-radius: 999px;
        background: #6A5848;
        box-shadow: 0 6px 16px #120C0F3D;
        filter: drop-shadow(0 2px 0 #3D2A1A8F);
      }
    </style>
    <main class="game-stage"><div class="icon"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let icon = find_bui_node(&document.root, "icon");
    let helper_layers = icon
        .children
        .iter()
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-filter-drop-shadow")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 1);
}

#[test]
fn filter_blur_adds_a_helper_shadow_layer() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; }
      .glow {
        width: 120px;
        height: 64px;
        border-radius: 999px;
        background: #D8B46C4D;
        filter: blur(8px);
      }
    </style>
    <main class="game-stage"><div class="glow"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let glow = find_bui_node(&document.root, "glow");
    let blur_layers = glow
        .children
        .iter()
        .filter(|child| child.markers.iter().any(|tag| tag == "css-filter-blur"))
        .collect::<Vec<_>>();
    assert_eq!(blur_layers.len(), 1);
}

#[test]
fn mix_blend_mode_multiply_darkens_filter_helper_shadow_layers() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; }
      .glow {
        width: 120px;
        height: 64px;
        border-radius: 999px;
        background: #D8B46C4D;
        filter: blur(8px);
        mix-blend-mode: multiply;
      }
    </style>
    <main class="game-stage"><div class="glow"></div></main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let glow = find_bui_node(&document.root, "glow");
    let blur_layer = glow
        .children
        .iter()
        .find(|child| child.markers.iter().any(|tag| tag == "css-filter-blur"))
        .expect("blur helper layer should exist");

    let blur_shadow = blur_layer
        .style
        .visuals
        .box_shadow
        .as_ref()
        .expect("shadow");
    assert_eq!(blur_shadow.color.as_deref(), Some("#AF915794"));
}

#[test]
fn filter_multiple_drop_shadows_add_multiple_helper_layers() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; }
      .cutout {
        width: 120px;
        height: 160px;
        background: #D7D1C6D8;
        filter:
          drop-shadow(24px 24px 0 #3D42543B)
          drop-shadow(0 24px 20px #2C21174D);
      }
    </style>
    <main class="game-stage"><div class="cutout"></div></main>
    "#;
    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let cutout = find_bui_node(&document.root, "cutout");
    let helper_layers = cutout
        .children
        .iter()
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-filter-drop-shadow")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 2);
}

#[test]
fn filter_drop_shadow_parses_oklch_color_functions() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; }
      .cutout {
        width: 120px;
        height: 160px;
        clip-path: polygon(35% 0, 68% 4%, 81% 18%, 90% 38%, 80% 64%, 86% 100%, 24% 100%, 31% 70%, 13% 55%, 20% 31%);
        filter:
          drop-shadow(24px 24px 0 oklch(29% 0.055 245 / 0.23))
          drop-shadow(0 24px 20px oklch(17% 0.032 45 / 0.3));
      }
    </style>
    <main class="game-stage"><div class="cutout"></div></main>
    "#;
    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let cutout = find_bui_node(&document.root, "cutout");
    let helper_layers = cutout
        .children
        .iter()
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-filter-drop-shadow")
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_layers.len(), 2);
}

#[test]
fn filter_drop_shadow_on_fully_transparent_node_does_not_create_fake_silhouette_layers() {
    let html = r#"
    <style>
      .game-stage { width: 320px; height: 180px; }
      .cutout {
        width: 120px;
        height: 160px;
        clip-path: polygon(35% 0, 68% 4%, 81% 18%, 90% 38%, 80% 64%, 86% 100%, 24% 100%, 31% 70%, 13% 55%, 20% 31%);
        filter: drop-shadow(24px 24px 0 oklch(29% 0.055 245 / 0.23));
      }
    </style>
    <main class="game-stage"><div class="cutout"></div></main>
    "#;
    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let cutout = find_bui_node(&document.root, "cutout");
    let helper_layers = cutout
        .children
        .iter()
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-filter-drop-shadow")
        })
        .collect::<Vec<_>>();
    assert!(helper_layers.is_empty());
}
