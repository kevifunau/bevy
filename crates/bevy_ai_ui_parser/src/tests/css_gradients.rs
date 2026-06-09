use super::shared::{css_simple_linear_gradient_overlay, find_bui_node};
use crate::core::opendesign::html::opendesign_html_to_bui_document;
use crate::core::style::css_gradients::{
    css_gradient_stops, css_linear_gradient_direction_from_degrees,
    css_simple_conic_gradient_overlays, css_simple_gradient_bands_from_stops,
    css_simple_linear_gradient_direction, css_simple_linear_gradient_overlays,
    css_simple_radial_gradient_overlays, css_simple_radial_gradient_ring_overlay,
    SimpleGradientOverlayDirection, SimpleGradientOverlayKind,
};
use crate::core::style::css_values::{blend_hex_colors, css_hex_rgba};

#[test]
fn css_linear_gradient_direction_supports_default_and_keyword_directions() {
    assert_eq!(
        css_simple_linear_gradient_direction(&["to right", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::LeftToRight, None, 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["to left", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::RightToLeft, None, 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["to top", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::BottomToTop, None, 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::TopToBottom, None, 0))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["to bottom right", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::LeftToRight, Some(135.0), 1))
    );
    assert_eq!(
        css_simple_linear_gradient_direction(&["to top left", "#fff", "transparent"]),
        Some((SimpleGradientOverlayDirection::RightToLeft, Some(315.0), 1))
    );
}

#[test]
fn css_linear_gradient_direction_maps_diagonal_angles_to_dominant_axis() {
    assert_eq!(
        css_linear_gradient_direction_from_degrees(90.0),
        Some((SimpleGradientOverlayDirection::LeftToRight, None))
    );
    assert_eq!(
        css_linear_gradient_direction_from_degrees(135.0),
        Some((SimpleGradientOverlayDirection::LeftToRight, Some(135.0)))
    );
    assert_eq!(
        css_linear_gradient_direction_from_degrees(180.0),
        Some((SimpleGradientOverlayDirection::TopToBottom, None))
    );
    assert_eq!(
        css_linear_gradient_direction_from_degrees(315.0),
        Some((SimpleGradientOverlayDirection::RightToLeft, Some(315.0)))
    );
}

#[test]
fn css_simple_linear_gradient_overlay_supports_default_direction() {
    let overlays = css_simple_linear_gradient_overlays("linear-gradient(#6d5a3d, #2d2119)");

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
    let overlays = css_simple_linear_gradient_overlays("linear-gradient(180deg, #f7edd6, #8c6c52)");

    assert!(overlays.len() >= 2);
    assert_eq!(overlays[0].color.to_ascii_uppercase(), "#F7EDD6");
    let last = overlays.last().expect("should have trailing band");
    assert_eq!(last.color.to_ascii_uppercase(), "#8C6C52");
    if let SimpleGradientOverlayKind::Linear { end_ratio, .. } = &last.kind {
        assert!((*end_ratio - 1.0).abs() < 0.01);
    }
}

#[test]
fn css_simple_linear_gradient_overlays_extract_trailing_segments_from_solid_multi_stop_gradients() {
    let overlays = css_simple_linear_gradient_overlays(
        "linear-gradient(180deg, #f5e7c8 0%, #d1b48c 46%, #8b6c53 100%)",
    );

    assert!(overlays.len() >= 4);

    match &overlays[0].kind {
        SimpleGradientOverlayKind::Linear {
            direction,
            diagonal_angle,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::TopToBottom);
            assert_eq!(*diagonal_angle, None);
            assert!(overlays[0].color.starts_with('#'));
            assert!(*start_ratio >= 0.0);
            assert!(*end_ratio > *start_ratio);
        }
        _ => panic!("expected linear overlay"),
    }

    match &overlays.last().expect("should have overlays").kind {
        SimpleGradientOverlayKind::Linear {
            direction,
            diagonal_angle,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::TopToBottom);
            assert_eq!(*diagonal_angle, None);
            assert!(overlays
                .last()
                .expect("should have overlays")
                .color
                .starts_with('#'));
            assert!(*start_ratio < 1.0);
            assert!(*end_ratio > *start_ratio);
            assert!(*end_ratio <= 1.0);
        }
        _ => panic!("expected linear overlay"),
    }
}

#[test]
fn css_simple_linear_gradient_overlays_add_more_bands_for_wide_two_stop_gradients() {
    let overlays =
        css_simple_linear_gradient_overlays("linear-gradient(90deg, #201818 0%, #f6e7c1 100%)");

    assert!(overlays.len() >= 6, "expected smoother gradient banding");
}

#[test]
fn css_simple_linear_gradient_overlay_supports_diagonal_highlight_bands() {
    let overlay = css_simple_linear_gradient_overlay(
        "linear-gradient(135deg, #f7edd6 0 22%, transparent 22% 48%)",
    )
    .expect("diagonal gradient should produce an overlay");

    assert_eq!(overlay.0, SimpleGradientOverlayDirection::LeftToRight);
    assert_eq!(overlay.1, Some(135.0));
    assert_eq!(overlay.2, "#f7edd6");
    assert_eq!(overlay.3, 0.0);
    assert_eq!(overlay.4, 0.48);
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
            diagonal_angle,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::LeftToRight);
            assert_eq!(*diagonal_angle, Some(110.0));
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
            diagonal_angle,
            start_ratio,
            end_ratio,
        } => {
            assert_eq!(*direction, SimpleGradientOverlayDirection::LeftToRight);
            assert_eq!(*diagonal_angle, Some(110.0));
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

    assert!(overlays.len() >= 2);

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
fn css_simple_radial_gradient_overlays_expand_into_soft_layers() {
    let overlays = css_simple_radial_gradient_overlays(
        "radial-gradient(circle at 34% 26%, #fff4d8 0%, transparent 29%)",
    );

    assert!(
        overlays.len() >= 3,
        "expected radial gradient to emit multiple soft layers"
    );
    assert!(overlays.iter().all(
        |overlay| matches!(overlay.kind, SimpleGradientOverlayKind::Radial { .. })
    ));
    assert!(overlays.iter().all(|overlay| matches!(
        overlay.kind,
        SimpleGradientOverlayKind::Radial {
            preserve_circle: true,
            ..
        }
    )));
}

#[test]
fn css_simple_radial_gradient_large_scene_wash_stays_more_conservative() {
    let overlays = css_simple_radial_gradient_overlays(
        "radial-gradient(circle at 24% 16%, #B7D5EC33 0%, transparent 32%)",
    );

    assert_eq!(overlays.len(), 3);
    assert!(overlays.iter().all(
        |overlay| matches!(overlay.kind, SimpleGradientOverlayKind::Radial { .. })
    ));
    let alphas = overlays
        .iter()
        .filter_map(|overlay| css_hex_rgba(&overlay.color).map(|(_, _, _, alpha)| alpha))
        .collect::<Vec<_>>();
    assert!(alphas.iter().all(|alpha| *alpha <= 0.08));
}

#[test]
fn opendesign_circle_radial_overlays_keep_geometry_in_wide_containers() {
    let html = r#"
    <style>
      .game-stage {
        width: 640px;
        aspect-ratio: 2;
        position: relative;
      }
      .wash {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 24% 16%, #B7D5EC33 0%, transparent 32%);
      }
    </style>
    <main class="game-stage">
      <div class="wash"></div>
    </main>
    "#;

    let document = opendesign_html_to_bui_document(html).expect("HTML should compile");
    let wash = find_bui_node(&document.root, "wash");
    let radial_overlay = wash
        .children
        .iter()
        .find(|child| child.id == "wash_gradient_overlay")
        .expect("expected radial overlay");

    assert!(radial_overlay
        .markers
        .iter()
        .any(|tag| tag == "css-radial-circle"));
    assert_eq!(radial_overlay.layout.styles.width.as_deref(), Some("69%"));
    assert_eq!(radial_overlay.layout.styles.height.as_deref(), Some("138%"));
    assert_eq!(radial_overlay.layout.styles.left.as_deref(), Some("-11%"));
    assert_eq!(radial_overlay.layout.styles.top.as_deref(), Some("-53.5%"));
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
        meter_fill.style.visuals.background_color.as_deref(),
        Some("#31C4A4")
    );
    let overlay_colors = meter_fill
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
    let stops = css_gradient_stops(&["transparent 0 54%", "#6E5A5D3D 64%", "#463C49A3 100%"])
        .expect("gradient stops should parse");

    let bands = css_simple_gradient_bands_from_stops(&stops);
    assert!(bands.len() >= 4);
    let first = bands.first().expect("should have a leading fade-in band");
    assert_eq!(first.start_ratio, 0.54);
    assert!(first.end_ratio > first.start_ratio);

    let last = bands.last().expect("should have a terminal color band");
    assert_eq!(last.color, "#463C49A3");
    assert!(last.start_ratio < 1.0);
    assert_eq!(last.end_ratio, 1.0);
    assert!(bands
        .windows(2)
        .all(|pair| pair[1].start_ratio >= pair[0].start_ratio));
}

#[test]
fn css_simple_gradient_bands_soften_transparent_leading_segment_before_terminal_color() {
    let stops = css_gradient_stops(&["transparent 0 54%", "#6E5A5D3D 64%", "#463C49A3 100%"])
        .expect("gradient stops should parse");

    let bands = css_simple_gradient_bands_from_stops(&stops);
    let fade_in = bands
        .iter()
        .take_while(|band| band.start_ratio < 0.64)
        .collect::<Vec<_>>();

    assert!(
        fade_in.len() >= 3,
        "leading transparent segment should soften into multiple bands"
    );
    assert!(fade_in
        .iter()
        .filter_map(|band| css_hex_rgba(&band.color).map(|(_, _, _, alpha)| alpha))
        .collect::<Vec<_>>()
        .windows(2)
        .all(|pair| pair[1] >= pair[0]));
}

#[test]
fn css_simple_gradient_bands_keep_terminal_color_for_fully_opaque_gradients() {
    let stops = css_gradient_stops(&["#6d5a3d", "#2d2119"]).expect("gradient stops should parse");

    let bands = css_simple_gradient_bands_from_stops(&stops);
    assert!(bands.len() >= 2);
    assert_eq!(bands[0].color.to_ascii_uppercase(), "#6D5A3D");
    let last = bands.last().expect("should have trailing band");
    assert_eq!(last.color.to_ascii_uppercase(), "#2D2119");
    assert!((last.end_ratio - 1.0).abs() < 0.01);
}

#[test]
fn css_simple_gradient_bands_keep_low_contrast_long_panel_fills_conservative() {
    let stops = css_gradient_stops(&["#BD9B7299 13%", "#A27F5DA8 100%"])
        .expect("gradient stops should parse");

    let bands = css_simple_gradient_bands_from_stops(&stops);
    assert_eq!(bands.len(), 4);
    assert_eq!(
        bands.first().map(|band| band.color.to_ascii_uppercase()),
        Some("#BD9B7299".to_string())
    );
    assert_eq!(
        bands.last().map(|band| band.color.to_ascii_uppercase()),
        Some("#A27F5DA8".to_string())
    );
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
    assert_eq!(
        mid.color.to_ascii_uppercase(),
        blended.unwrap().to_ascii_uppercase()
    );
    assert_eq!(bands.last().unwrap().color.to_ascii_uppercase(), "#6A4A2A");
    assert!((bands.last().unwrap().end_ratio - 1.0).abs() < 0.01);
}

#[test]
fn blend_hex_colors_produces_correct_intermediate_colors() {
    let mid = blend_hex_colors("#FFFFFF", "#000000", 0.5).expect("blend should work");
    assert_eq!(mid, "#808080");

    let quarter = blend_hex_colors("#FF0000", "#0000FF", 0.25).expect("blend should work");
    assert_eq!(quarter.to_ascii_uppercase(), "#BF0040");

    let with_alpha =
        blend_hex_colors("#FF000080", "#00FF0080", 0.5).expect("blend should work with alpha");
    assert!(with_alpha.to_ascii_uppercase().starts_with("#8080"));
}
