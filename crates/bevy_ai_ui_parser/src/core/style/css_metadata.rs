#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CssPropertySupportLevel {
    P0,
    P1,
    P2,
    Unsupported,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CssFallbackStrategy {
    Native,
    HelperLayer,
    ColorApproximation,
    SemanticFallback,
    None,
}

#[allow(dead_code)]
pub(crate) struct CssPropertyInfo {
    pub(crate) level: CssPropertySupportLevel,
    pub(crate) strategy: CssFallbackStrategy,
    pub(crate) helper_tag: Option<&'static str>,
}

#[allow(dead_code)]
pub(crate) fn css_property_info(name: &str) -> CssPropertyInfo {
    match name {
        "display"
        | "position"
        | "width"
        | "height"
        | "min-width"
        | "min-height"
        | "max-width"
        | "max-height"
        | "inset"
        | "left"
        | "right"
        | "top"
        | "bottom"
        | "margin"
        | "margin-left"
        | "margin-right"
        | "margin-top"
        | "margin-bottom"
        | "padding"
        | "padding-left"
        | "padding-right"
        | "padding-top"
        | "padding-bottom"
        | "padding-inline"
        | "padding-block"
        | "gap"
        | "row-gap"
        | "column-gap"
        | "flex-direction"
        | "flex-wrap"
        | "flex-grow"
        | "flex-shrink"
        | "flex-basis"
        | "align-items"
        | "align-self"
        | "align-content"
        | "justify-content"
        | "justify-items"
        | "justify-self"
        | "place-items"
        | "overflow"
        | "overflow-x"
        | "overflow-y"
        | "grid-template-columns"
        | "grid-template-rows"
        | "aspect-ratio"
        | "z-index" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "background-color" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "background" => CssPropertyInfo {
            level: CssPropertySupportLevel::P1,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-gradient-overlay"),
        },
        "background-image" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "background-size" | "background-position" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "border"
        | "border-top"
        | "border-bottom"
        | "border-left"
        | "border-right"
        | "border-color"
        | "border-top-color"
        | "border-bottom-color"
        | "border-left-color"
        | "border-right-color"
        | "border-width"
        | "border-top-width"
        | "border-bottom-width"
        | "border-left-width"
        | "border-right-width"
        | "border-radius" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-edge-border"),
        },
        "color" | "font-size" | "font-family" | "font-weight" | "line-height"
        | "letter-spacing" | "text-align" | "text-shadow" | "white-space" | "opacity" => {
            CssPropertyInfo {
                level: CssPropertySupportLevel::P0,
                strategy: CssFallbackStrategy::Native,
                helper_tag: None,
            }
        }
        "box-shadow" => CssPropertyInfo {
            level: CssPropertySupportLevel::P1,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-box-shadow-layer"),
        },
        "filter" => CssPropertyInfo {
            level: CssPropertySupportLevel::P1,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-filter-drop-shadow"),
        },
        "mask-image" => CssPropertyInfo {
            level: CssPropertySupportLevel::P2,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-mask-fade"),
        },
        "clip-path" => CssPropertyInfo {
            level: CssPropertySupportLevel::P2,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-clip-contour"),
        },
        "mix-blend-mode" => CssPropertyInfo {
            level: CssPropertySupportLevel::P2,
            strategy: CssFallbackStrategy::ColorApproximation,
            helper_tag: None,
        },
        "transform" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "cursor"
        | "pointer-events"
        | "transition"
        | "content"
        | "isolation"
        | "-webkit-tap-highlight-color" => CssPropertyInfo {
            level: CssPropertySupportLevel::Unsupported,
            strategy: CssFallbackStrategy::None,
            helper_tag: None,
        },
        _ => CssPropertyInfo {
            level: CssPropertySupportLevel::Unsupported,
            strategy: CssFallbackStrategy::None,
            helper_tag: None,
        },
    }
}

#[allow(dead_code)]
pub(crate) struct CssEffectFallbackEntry {
    pub(crate) css_property: &'static str,
    pub(crate) helper_tag: &'static str,
    pub(crate) fallback_fn: &'static str,
    pub(crate) description: &'static str,
}

#[allow(dead_code)]
pub(crate) fn css_effect_fallback_registry() -> Vec<CssEffectFallbackEntry> {
    vec![
        CssEffectFallbackEntry {
            css_property: "background (gradient)",
            helper_tag: "css-gradient-overlay",
            fallback_fn: "apply_simple_gradient_overlays",
            description: "Gradient decomposed into positioned solid-color overlay bands",
        },
        CssEffectFallbackEntry {
            css_property: "box-shadow (multi-layer)",
            helper_tag: "css-box-shadow-layer",
            fallback_fn: "apply_box_shadow_fallback",
            description: "Primary shadow to node box_shadow; secondary shadows to absolute-positioned helper children",
        },
        CssEffectFallbackEntry {
            css_property: "filter: drop-shadow(...)",
            helper_tag: "css-filter-drop-shadow",
            fallback_fn: "css_filter_drop_shadows + push_box_shadow_layer",
            description: "Each drop-shadow becomes a box-shadow layer child; on text nodes becomes text_shadow",
        },
        CssEffectFallbackEntry {
            css_property: "filter: blur(...)",
            helper_tag: "css-filter-blur",
            fallback_fn: "apply_filter_blur_fallback",
            description: "Approximated as a zero-offset box-shadow with spread; on text nodes becomes low-alpha text_shadow",
        },
        CssEffectFallbackEntry {
            css_property: "filter: brightness/contrast/saturate",
            helper_tag: "N/A",
            fallback_fn: "css_filter_color_adjustment + apply_filter_color_adjustment",
            description: "Applied as direct color channel adjustment to background/border/text colors",
        },
        CssEffectFallbackEntry {
            css_property: "mask-image: linear-gradient(...)",
            helper_tag: "css-mask-fade",
            fallback_fn: "apply_mask_image_fallback",
            description: "Three gradient-fade child layers at decreasing alpha (62/34/16%) approximating edge fade",
        },
        CssEffectFallbackEntry {
            css_property: "clip-path: polygon(...)",
            helper_tag: "css-clip-contour",
            fallback_fn: "apply_clip_path_fallback",
            description: "Fill, contour and accent child nodes approximating clipped shape; bounded inner fill extracted",
        },
        CssEffectFallbackEntry {
            css_property: "mix-blend-mode: multiply",
            helper_tag: "N/A",
            fallback_fn: "apply_mix_blend_mode_fallback",
            description: "Darkens color channels of gradient overlays and helper shadow layers to approximate multiply",
        },
        CssEffectFallbackEntry {
            css_property: "inline SVG icon",
            helper_tag: "svg:fallback",
            fallback_fn: "svg_shape_fallback_profile + semantic_svg_fallback",
            description: "SVG paths replaced with Unicode text characters via semantic matching and shape profile recognition",
        },
        CssEffectFallbackEntry {
            css_property: "::before / ::after",
            helper_tag: "pseudo:before / pseudo:after",
            fallback_fn: "apply_opendesign_styles (pseudo path)",
            description: "Pseudo-element declarations applied as child nodes with pseudo markers",
        },
        CssEffectFallbackEntry {
            css_property: "per-edge border",
            helper_tag: "css-edge-border:{edge}",
            fallback_fn: "apply_css_edge_border + ensure_edge_border_node",
            description: "Individual edge borders (top/right/bottom/left) created as absolute-positioned child nodes",
        },
    ]
}
