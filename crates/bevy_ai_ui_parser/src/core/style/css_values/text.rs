use crate::core::{model::BuiTextConfig, style::css_parser::normalize_token};

pub(crate) fn css_font_weight(value: &str) -> Option<u16> {
    match normalize_token(value).as_str() {
        "normal" => Some(400),
        "bold" => Some(700),
        "bolder" => Some(700),
        "lighter" => Some(300),
        other => other
            .parse::<u16>()
            .ok()
            .map(|weight| weight.clamp(1, 1000)),
    }
}

pub(crate) fn css_font_family_to_path(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("sfmono")
        || lower.contains("menlo")
        || lower.contains("monospace")
        || lower.contains("consolas")
        || lower.contains("ui-monospace")
    {
        "Menlo.ttc".to_string()
    } else if lower.contains("palatino")
        || lower.contains("iowan")
        || lower.contains("georgia")
        || lower.contains("serif")
    {
        "Palatino.ttc".to_string()
    } else if lower.contains("songti") {
        "Songti.ttc".to_string()
    } else if lower.contains("pingfang") {
        "PingFang.ttc".to_string()
    } else if lower.contains("stheiti") {
        "STHeiti Medium.ttc".to_string()
    } else {
        "Hiragino Sans GB.ttc".to_string()
    }
}

pub(crate) fn adjust_font_path_for_content(font_path: &str, content: &str) -> String {
    if uses_latin_display_font(font_path) && contains_cjk(content) {
        return "Songti.ttc".to_string();
    }

    font_path.to_string()
}

pub(crate) fn apply_css_white_space(text_config: &mut BuiTextConfig, value: &str) {
    match normalize_token(value).as_str() {
        "normal" => {
            text_config.allow_newlines = Some(false);
            text_config.linebreak = Some("word_boundary".to_string());
        }
        "nowrap" | "no_wrap" => {
            text_config.allow_newlines = Some(false);
            text_config.linebreak = Some("no_wrap".to_string());
        }
        "pre" => {
            text_config.allow_newlines = Some(true);
            text_config.linebreak = Some("no_wrap".to_string());
        }
        "pre_line" => {
            text_config.allow_newlines = Some(true);
            text_config.linebreak = Some("word_boundary".to_string());
        }
        "pre_wrap" | "break_spaces" => {
            text_config.allow_newlines = Some(true);
            text_config.linebreak = Some("any_character".to_string());
        }
        _ => {}
    }
}

fn uses_latin_display_font(font_path: &str) -> bool {
    matches!(
        font_path,
        "Palatino.ttc" | "Georgia.ttf" | "Times New Roman.ttf"
    )
}

fn contains_cjk(content: &str) -> bool {
    content.chars().any(is_cjk_character)
}

fn is_cjk_character(character: char) -> bool {
    matches!(
        character as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
            | 0x30000..=0x3134F
    )
}
