use crate::core::model::{BuiNode, BuiTextShadowConfig};

#[derive(Clone, Copy)]
pub(crate) struct SemanticSvgFallbackSpec {
    pub(crate) icon: &'static str,
    pub(crate) font_size: Option<f32>,
    pub(crate) color: &'static str,
    pub(crate) font_path: &'static str,
    pub(crate) shadow_color: Option<&'static str>,
    pub(crate) shadow_offset_y: f32,
}

impl SemanticSvgFallbackSpec {
    pub(crate) fn text_shadow(self) -> Option<BuiTextShadowConfig> {
        self.shadow_color.map(|color| BuiTextShadowConfig {
            offset_x: Some(0.0),
            offset_y: Some(self.shadow_offset_y),
            color: Some(color.to_string()),
        })
    }
}

pub(crate) fn semantic_svg_fallback_spec(parent: &BuiNode) -> Option<SemanticSvgFallbackSpec> {
    if let Some(spec) = semantic_svg_fallback_spec_from_tags(parent) {
        return Some(spec);
    }

    if parent.id == "backbutton" {
        return Some(SemanticSvgFallbackSpec {
            icon: "←",
            font_size: Some(28.0),
            color: "#F5C85A",
            font_path: "Apple Symbols.ttf",
            shadow_color: Some("#5A3F18A0"),
            shadow_offset_y: 2.0,
        });
    }

    if let Some(spec) = indexed_semantic_svg_fallback_spec(
        &parent.id,
        "bar_icon",
        &["★", "✦"],
        22.0,
        "#F5E6B8",
        Some("#3D2A1A8F"),
        2.0,
    ) {
        return Some(spec);
    }

    if let Some(spec) = indexed_semantic_svg_fallback_spec(
        &parent.id,
        "skill_button",
        &["⚡", "♛", "▤"],
        22.0,
        "#F6ECDD",
        None,
        0.0,
    ) {
        return Some(if parent.id == "skill_button" {
            SemanticSvgFallbackSpec {
                font_size: Some(24.0),
                ..spec
            }
        } else {
            spec
        });
    }

    if let Some(spec) = indexed_semantic_svg_fallback_spec(
        &parent.id,
        "equip_slot",
        &["⚔", "⬢", "✦", "♞", "◎"],
        22.0,
        "#F3E3C6",
        None,
        0.0,
    ) {
        return Some(if parent.id == "equip_slot" {
            SemanticSvgFallbackSpec {
                font_size: Some(24.0),
                ..spec
            }
        } else {
            spec
        });
    }

    let has_class = |class_name: &str| {
        parent
            .markers
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    if has_class("round-button") {
        return Some(SemanticSvgFallbackSpec {
            icon: "←",
            font_size: Some(28.0),
            color: "#F5C85A",
            font_path: "Apple Symbols.ttf",
            shadow_color: Some("#5A3F18A0"),
            shadow_offset_y: 2.0,
        });
    }
    if has_class("bar-icon") {
        return Some(SemanticSvgFallbackSpec {
            icon: "★",
            font_size: Some(22.0),
            color: "#F5E6B8",
            font_path: "Apple Symbols.ttf",
            shadow_color: Some("#3D2A1A8F"),
            shadow_offset_y: 2.0,
        });
    }
    if has_class("star") || parent.id == "stars" {
        return Some(SemanticSvgFallbackSpec {
            icon: "★",
            font_size: Some(42.0),
            color: "#F5C742",
            font_path: "Apple Symbols.ttf",
            shadow_color: Some("#5A341CA0"),
            shadow_offset_y: 3.0,
        });
    }

    None
}

fn semantic_svg_fallback_spec_from_tags(parent: &BuiNode) -> Option<SemanticSvgFallbackSpec> {
    let find_tag_value = |prefix: &str| {
        parent
            .markers
            .iter()
            .find_map(|tag| tag.strip_prefix(prefix))
    };

    if let Some(skill) = find_tag_value("data-skill:") {
        return semantic_skill_icon_spec(skill);
    }
    if let Some(equip) = find_tag_value("data-equip:") {
        return semantic_equip_icon_spec(equip);
    }
    if let Some(label) = find_tag_value("aria-label:") {
        if let Some(spec) = semantic_skill_icon_spec(label) {
            return Some(spec);
        }
        if let Some(spec) = semantic_equip_icon_spec(label) {
            return Some(spec);
        }
        if let Some(spec) = semantic_aria_icon_spec(label) {
            return Some(spec);
        }
    }

    None
}

fn semantic_skill_icon_spec(skill: &str) -> Option<SemanticSvgFallbackSpec> {
    let icon = if skill.contains("震击") || skill.contains("雷") || skill.contains("击") {
        "⚡"
    } else if skill.contains("号令") || skill.contains("军团") {
        "♛"
    } else if skill.contains("战策") || skill.contains("圣卷") || skill.contains("卷") {
        "▤"
    } else {
        return None;
    };

    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(22.0),
        color: "#F6ECDD",
        font_path: "Apple Symbols.ttf",
        shadow_color: None,
        shadow_offset_y: 0.0,
    })
}

fn semantic_equip_icon_spec(equip: &str) -> Option<SemanticSvgFallbackSpec> {
    let icon = if equip.contains("弓") {
        "⚔"
    } else if equip.contains("盾") {
        "⬢"
    } else if equip.contains("矛") {
        "✦"
    } else if equip.contains("坐骑") || equip.contains("战马") || equip.contains("骑") {
        "♞"
    } else if equip.contains("徽章") || equip.contains("鹰眼") || equip.contains("徽") {
        "◎"
    } else {
        return None;
    };

    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(22.0),
        color: "#F3E3C6",
        font_path: "Apple Symbols.ttf",
        shadow_color: None,
        shadow_offset_y: 0.0,
    })
}

fn semantic_aria_icon_spec(label: &str) -> Option<SemanticSvgFallbackSpec> {
    let icon = if label.contains("返回") {
        "←"
    } else {
        return None;
    };

    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(28.0),
        color: "#F5C85A",
        font_path: "Apple Symbols.ttf",
        shadow_color: Some("#5A3F18A0"),
        shadow_offset_y: 2.0,
    })
}

fn indexed_semantic_svg_fallback_spec(
    id: &str,
    base_id: &str,
    icons: &[&'static str],
    font_size: f32,
    color: &'static str,
    shadow_color: Option<&'static str>,
    shadow_offset_y: f32,
) -> Option<SemanticSvgFallbackSpec> {
    let index = if id == base_id {
        0
    } else if let Some(suffix) = id.strip_prefix(&format!("{base_id}_")) {
        suffix.parse::<usize>().ok()?.checked_sub(1)?
    } else {
        return None;
    };

    let icon = *icons.get(index)?;
    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(font_size),
        color,
        font_path: "Apple Symbols.ttf",
        shadow_color,
        shadow_offset_y,
    })
}
