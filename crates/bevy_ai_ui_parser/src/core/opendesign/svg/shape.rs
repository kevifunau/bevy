use crate::core::{model::BuiNode, style::css_parser::normalize_token};

use super::semantic::semantic_svg_fallback_spec;

#[derive(Default)]
struct SvgShapeProfile {
    path_count: usize,
    circle_count: usize,
    filled_path_count: usize,
    stroked_path_count: usize,
    round_stroke_path_count: usize,
    horizontal_path_hints: usize,
    vertical_path_hints: usize,
    curved_path_hints: usize,
}

pub(crate) fn svg_fallback_icon(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
) -> Option<&'static str> {
    if let Some(spec) = semantic_svg_fallback_spec(parent) {
        return Some(spec.icon);
    }

    if let Some(icon) = svg_shape_fallback_icon(parent, svg_node) {
        return Some(icon);
    }

    let signature = svg_signature(svg_node);

    if parent
        .custom_tags
        .iter()
        .any(|tag| tag == "class:round-button")
        || signature.contains("M38 13 19 32l19 19")
    {
        return Some("←");
    }
    if signature.contains("M16 2.2 20.2 11") || signature.contains("M16 2 20 11l10 1") {
        return Some("★");
    }
    if signature.contains("M9 4h8l2 7 6 2") {
        return Some("✦");
    }
    if signature.contains("M20 4 7 20h8l-3 12") {
        return Some("⚡");
    }
    if signature.contains("M18 4c5 0 9 4 9 9v6l4 6") {
        return Some("♛");
    }
    if signature.contains("M8 6h17c2 0 4 2 4 4v20") {
        return Some("▤");
    }
    if signature.contains("M28 4c-10 7-14 18-9 32") {
        return Some("➶");
    }
    if signature.contains("M20 4 32 9v9c0 8-5 14-12 18") {
        return Some("⛨");
    }
    if signature.contains("M28 3 36 12 18 29 11 22") {
        return Some("⟡");
    }
    if signature.contains("M5 25c7-12 15-13 28-8") {
        return Some("♞");
    }
    if signature.contains("circle:20:20:14") || signature.contains("M16 5v22M5 16h22") {
        return Some("◎");
    }
    if signature.contains("M11 8a5 5 0 0 1 10 0") {
        return Some("智");
    }
    if signature.contains("M18 2 8 18h8l-2 12") {
        return Some("速");
    }
    if signature.contains("M4 13 16 5l12 8H4") {
        return Some("政");
    }
    if signature.contains("M7 25 25 7M20 5l7 7") {
        return Some("武");
    }
    if signature.contains("M5 19c5-11 16-13 22-9") {
        return Some("统");
    }
    if signature.contains("M16 4 27 9v8c0 7-4.5 12-11 15") {
        return Some("守");
    }

    None
}

fn svg_shape_fallback_icon(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
) -> Option<&'static str> {
    let has_class = |class_name: &str| {
        parent
            .custom_tags
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    let profile = svg_shape_profile(svg_node);

    if has_class("equip-slot") {
        if profile.circle_count >= 1 && profile.stroked_path_count >= 2 {
            return Some("◎");
        }
        if profile.path_count >= 3
            && profile.stroked_path_count == profile.path_count
            && profile.round_stroke_path_count >= 1
        {
            return Some("➶");
        }
        if profile.path_count >= 2
            && profile.stroked_path_count == profile.path_count
            && profile.filled_path_count == 0
            && profile.horizontal_path_hints >= 1
        {
            return Some("⛨");
        }
        if profile.filled_path_count >= 1 && profile.round_stroke_path_count >= 1 {
            return Some("⟡");
        }
        if profile.filled_path_count >= 1 && profile.stroked_path_count >= 1 {
            return Some("♞");
        }
    }

    if has_class("skill-button") && profile.path_count == 1 && profile.filled_path_count == 1 {
        let path_data = svg_first_path_data(svg_node)?.to_ascii_lowercase();
        if path_data.contains('h') && path_data.contains('v') {
            return Some("▤");
        }
        if path_data.contains('c') {
            return Some("♛");
        }
        return Some("⚡");
    }

    if has_class("bar-icon") && profile.path_count == 1 && profile.filled_path_count == 1 {
        let path_data = svg_first_path_data(svg_node)?.to_ascii_lowercase();
        if path_data.contains('h') && profile.vertical_path_hints >= 1 {
            return Some("✦");
        }
        return Some("★");
    }

    None
}

fn svg_shape_profile(svg_node: roxmltree::Node<'_, '_>) -> SvgShapeProfile {
    let mut profile = SvgShapeProfile::default();

    for node in svg_node.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "path" => {
                profile.path_count += 1;
                if svg_path_has_fill(node) {
                    profile.filled_path_count += 1;
                }
                if node.attribute("stroke").is_some() {
                    profile.stroked_path_count += 1;
                }
                if node.attribute("stroke-linecap") == Some("round") {
                    profile.round_stroke_path_count += 1;
                }
                if let Some(data) = node.attribute("d") {
                    let data = data.to_ascii_lowercase();
                    if data.contains('h') {
                        profile.horizontal_path_hints += 1;
                    }
                    if data.contains('v') {
                        profile.vertical_path_hints += 1;
                    }
                    if data.contains('c') || data.contains('q') || data.contains('a') {
                        profile.curved_path_hints += 1;
                    }
                }
            }
            "circle" => {
                profile.circle_count += 1;
            }
            _ => {}
        }
    }

    profile
}

fn svg_path_has_fill(node: roxmltree::Node<'_, '_>) -> bool {
    matches!(node.attribute("fill"), Some(fill) if normalize_token(fill) != "none")
}

fn svg_first_path_data(svg_node: roxmltree::Node<'_, '_>) -> Option<String> {
    svg_node
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "path")
        .and_then(|node| node.attribute("d"))
        .map(ToString::to_string)
}

fn svg_signature(svg_node: roxmltree::Node<'_, '_>) -> String {
    let mut parts = Vec::new();
    for node in svg_node.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "path" => {
                if let Some(value) = node.attribute("d") {
                    parts.push(value.replace(char::is_whitespace, " "));
                }
            }
            "circle" => {
                let cx = node.attribute("cx").unwrap_or_default();
                let cy = node.attribute("cy").unwrap_or_default();
                let r = node.attribute("r").unwrap_or_default();
                parts.push(format!("circle:{cx}:{cy}:{r}"));
            }
            _ => {}
        }
    }
    parts.join("|")
}
