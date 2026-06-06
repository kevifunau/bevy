use crate::core::{
    model::{BuiNode, BuiTextShadowConfig, text_node},
    opendesign::{
        build::apply_opendesign_styles,
        generic::apply_inherited_text_styles,
        stylesheet::OpenDesignStylesheet,
    },
    support::tree::find_bui_node_mut,
};

use super::{
    semantic::semantic_svg_fallback_spec,
    shape::svg_fallback_icon,
};

pub(crate) fn ensure_text_icon_child(root: &mut BuiNode, id: &str) {
    let Some(node) = find_bui_node_mut(root, id) else {
        return;
    };
    let Some(spec) = semantic_svg_fallback_spec(node) else {
        return;
    };
    if node
        .children
        .iter()
        .any(|child| !is_decorative_icon_helper_node(child))
    {
        return;
    }
    node.children
        .retain(|child| !child.markers.iter().any(|tag| tag == "svg:fallback"));
    let mut icon_node = text_node(
        &format!("{id}_icon_text"),
        spec.icon,
        spec.font_size.unwrap_or(20.0),
        spec.color,
        Some(spec.font_path),
    );
    if let Some(text_shadow) = spec.text_shadow()
        && let Some(text_config) = icon_node.content.text.as_mut()
    {
        text_config.text_shadow = Some(text_shadow);
    }
    node.children.push(icon_node);
}

pub(crate) fn is_decorative_icon_helper_node(node: &BuiNode) -> bool {
    node.markers.iter().any(|tag| {
        tag == "pseudo:before"
            || tag == "pseudo:after"
            || tag == "class:cooldown"
            || tag == "svg:fallback"
    })
}

pub(crate) fn is_svg_tag(tag: &str) -> bool {
    matches!(
        tag,
        "svg" | "path" | "circle" | "ellipse" | "rect" | "line" | "polyline" | "polygon" | "g"
    )
}

pub(crate) fn svg_fallback_text_node(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
    index: usize,
) -> Option<BuiNode> {
    let icon = svg_fallback_icon(parent, svg_node)?;
    let fallback_style = svg_fallback_style(parent, icon);
    let mut text_node = text_node(
        &format!("{}_svg_fallback_{}", parent.id, index),
        icon,
        fallback_style
            .font_size
            .unwrap_or_else(|| svg_fallback_font_size(parent, svg_node, stylesheet)),
        fallback_style.color,
        Some(fallback_style.font_path),
    );
    if let Some(text_shadow) = fallback_style.text_shadow
        && let Some(text_config) = text_node.content.text.as_mut()
    {
        text_config.text_shadow = Some(text_shadow);
    }
    text_node.markers.push("svg:fallback".to_string());
    Some(text_node)
}

struct SvgFallbackStyle {
    font_size: Option<f32>,
    color: &'static str,
    font_path: &'static str,
    text_shadow: Option<BuiTextShadowConfig>,
}

fn svg_fallback_style(parent: &BuiNode, icon: &str) -> SvgFallbackStyle {
    if let Some(spec) = semantic_svg_fallback_spec(parent) {
        return SvgFallbackStyle {
            font_size: spec.font_size,
            color: spec.color,
            font_path: spec.font_path,
            text_shadow: spec.text_shadow(),
        };
    }

    let has_class = |class_name: &str| {
        parent
            .markers
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    if has_class("round-button") {
        return SvgFallbackStyle {
            font_size: Some(28.0),
            color: "#F5C85A",
            font_path: "Apple Symbols.ttf",
            text_shadow: Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(2.0),
                color: Some("#5A3F18A0".to_string()),
            }),
        };
    }

    if has_class("bar-icon") {
        return SvgFallbackStyle {
            font_size: Some(22.0),
            color: "#F5E6B8",
            font_path: "Apple Symbols.ttf",
            text_shadow: Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(2.0),
                color: Some("#3D2A1A8F".to_string()),
            }),
        };
    }

    if has_class("star") || parent.id == "stars" {
        return SvgFallbackStyle {
            font_size: Some(42.0),
            color: "#F5C742",
            font_path: "Apple Symbols.ttf",
            text_shadow: Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(3.0),
                color: Some("#5A341CA0".to_string()),
            }),
        };
    }

    if has_class("stat-label") {
        return SvgFallbackStyle {
            font_size: Some(22.0),
            color: "#E9DDC8",
            font_path: "Apple Symbols.ttf",
            text_shadow: None,
        };
    }

    let (font_size, color) = match icon {
        "⚡" => (Some(24.0), "#F6ECDD"),
        "♛" | "▤" => (Some(22.0), "#F6ECDD"),
        "➶" => (Some(24.0), "#F3E3C6"),
        "⛨" | "⟡" | "♞" | "◎" => (Some(22.0), "#F3E3C6"),
        "★" => (Some(22.0), "#F5E6B8"),
        "✦" => (Some(22.0), "#F5E6B8"),
        _ => (None, "#F4E7CA"),
    };

    SvgFallbackStyle {
        font_size,
        color,
        font_path: "Apple Symbols.ttf",
        text_shadow: None,
    }
}

fn svg_fallback_font_size(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
) -> f32 {
    let mut probe = text_node("svg_fallback_probe", "•", 16.0, "#FFFFFF", None);
    apply_inherited_text_styles(stylesheet, &mut probe, svg_node);
    apply_opendesign_styles(stylesheet, &mut probe, svg_node);
    if let Some(text_config) = probe.content.text.as_ref() {
        return text_config.font_size.clamp(16.0, 28.0);
    }
    if parent
        .markers
        .iter()
        .any(|tag| tag == "class:star" || tag == "class:bar-icon")
    {
        return 22.0;
    }
    20.0
}