use super::*;

#[test]
fn svg_asset_key_uses_svg_id_attribute() {
    let parent = bui_node("skill_button", "button");
    let svg = roxmltree::Document::parse(
        r#"<svg id="skill_icon" viewBox="0 0 40 40"><circle cx="20" cy="20" r="14" fill="red"/></svg>"#,
    )
    .expect("svg should parse");

    let key = svg_asset_key(&parent, svg.root_element(), 1);
    assert_eq!(key, "skill_button__skill_icon");
}

#[test]
fn svg_asset_key_falls_back_to_index_without_id() {
    let parent = bui_node("bar_icon", "node");
    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="14" fill="red"/></svg>"#,
    )
    .expect("svg should parse");

    let key = svg_asset_key(&parent, svg.root_element(), 3);
    assert_eq!(key, "bar_icon__svg_3");
}

#[test]
fn svg_viewbox_size_parses_viewbox_attribute() {
    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="14"/></svg>"#,
    )
    .expect("svg should parse");

    let (w, h) = svg_viewbox_size(svg.root_element());
    assert_eq!(w, 40.0);
    assert_eq!(h, 40.0);
}

#[test]
fn svg_viewbox_size_parses_width_height_without_viewbox() {
    let svg = roxmltree::Document::parse(
        r#"<svg width="200px" height="260px"><rect x="0" y="0" width="200" height="260"/></svg>"#,
    )
    .expect("svg should parse");

    let (w, h) = svg_viewbox_size(svg.root_element());
    assert_eq!(w, 200.0);
    assert_eq!(h, 260.0);
}

#[test]
fn svg_viewbox_size_defaults_to_32x32() {
    let svg = roxmltree::Document::parse(r#"<svg><circle cx="16" cy="16" r="16"/></svg>"#)
        .expect("svg should parse");

    let (w, h) = svg_viewbox_size(svg.root_element());
    assert_eq!(w, 32.0);
    assert_eq!(h, 32.0);
}

#[test]
fn svg_render_scale_is_2x_viewbox() {
    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="14"/></svg>"#,
    )
    .expect("svg should parse");

    let (w, h) = svg_render_scale(svg.root_element());
    assert_eq!(w, 80);
    assert_eq!(h, 80);
}

#[test]
fn rasterize_svg_to_png_produces_valid_png_bytes() {
    let svg_markup = r#"<svg viewBox="0 0 40 40" xmlns="http://www.w3.org/2000/svg"><circle cx="20" cy="20" r="14" fill="red"/></svg>"#;

    let png_bytes = rasterize_svg_to_png(svg_markup, 80, 80).expect("rasterization should succeed");
    assert!(!png_bytes.is_empty());

    assert_eq!(&png_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn extract_svg_markup_preserves_full_svg_structure() {
    let svg = roxmltree::Document::parse(
        r#"<svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="14" fill="none" stroke="currentColor" stroke-width="3"/></svg>"#,
    )
    .expect("svg should parse");

    let markup = extract_svg_markup(svg.root_element());
    assert!(markup.contains("svg"));
    assert!(markup.contains("circle"));
    assert!(markup.contains("viewBox"));
    assert!(markup.contains("currentColor"));
}
