use super::*;

#[test]
fn css_property_support_matrix_classifies_p0_properties() {
    let p0_properties = [
        "display",
        "position",
        "width",
        "height",
        "z-index",
        "background-color",
        "color",
        "font-size",
        "font-family",
        "line-height",
        "white-space",
        "aspect-ratio",
    ];
    for prop in p0_properties {
        let info = css_property_info(prop);
        assert_eq!(info.level, CssPropertySupportLevel::P0);
    }
}

#[test]
fn css_property_support_matrix_classifies_p1_properties() {
    for prop in ["background", "box-shadow", "filter"] {
        let info = css_property_info(prop);
        assert_eq!(info.level, CssPropertySupportLevel::P1);
    }
}

#[test]
fn css_property_support_matrix_classifies_p2_properties() {
    for prop in ["mask-image", "clip-path", "mix-blend-mode"] {
        let info = css_property_info(prop);
        assert_eq!(info.level, CssPropertySupportLevel::P2);
    }
}

#[test]
fn css_effect_fallback_registry_documents_all_fallback_entries() {
    let registry = css_effect_fallback_registry();
    assert!(registry.len() >= 10);
    let gradient_entry = registry
        .iter()
        .find(|e| e.css_property == "background (gradient)");
    assert!(gradient_entry.is_some());
    assert_eq!(
        gradient_entry.expect("gradient entry").helper_tag,
        "css-gradient-overlay"
    );
    let mask_entry = registry
        .iter()
        .find(|e| e.css_property == "mask-image: linear-gradient(...)");
    assert!(mask_entry.is_some());
    assert_eq!(mask_entry.expect("mask entry").helper_tag, "css-mask-fade");
    let clip_entry = registry
        .iter()
        .find(|e| e.css_property == "clip-path: polygon(...)");
    assert!(clip_entry.is_some());
    assert_eq!(
        clip_entry.expect("clip entry").helper_tag,
        "css-clip-contour"
    );
}
