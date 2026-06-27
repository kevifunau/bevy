use crate::core::{
    model::BuiTextConfig,
    style::css_parser::{css_font_size, css_line_height, normalize_token},
};

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

pub(crate) fn apply_css_font_shorthand(text_config: &mut BuiTextConfig, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("inherit") {
        return;
    }

    let mut family_parts = Vec::new();
    let mut saw_size = false;
    for token in split_font_shorthand_tokens(value) {
        let normalized = normalize_token(&token);
        if !saw_size {
            if let Some(weight) = css_font_weight(&normalized) {
                text_config.font_weight = Some(weight);
                continue;
            }
            if normalized == "italic"
                || normalized == "oblique"
                || normalized == "normal"
                || normalized == "small_caps"
            {
                continue;
            }
        }

        if !saw_size {
            if let Some((size, line_height)) = token.split_once('/') {
                if let Some(font_size) = css_font_size(size) {
                    text_config.font_size = font_size;
                    saw_size = true;
                }
                if let Some(line_height) = css_line_height(line_height) {
                    text_config.line_height = Some(line_height);
                }
                continue;
            }
            if let Some(font_size) = css_font_size(&token) {
                text_config.font_size = font_size;
                saw_size = true;
                continue;
            }
        } else {
            family_parts.push(token);
        }
    }

    if !family_parts.is_empty() {
        let family = family_parts.join(" ");
        let mapped = css_font_family_to_path(&family);
        text_config.font_path = Some(adjust_font_path_for_content(&mapped, &text_config.content));
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

pub(crate) fn normalize_cjk_linebreak(text_config: &mut BuiTextConfig) {
    if !contains_cjk(&text_config.content) {
        return;
    }

    if text_config.linebreak.as_deref().is_none()
        || text_config.linebreak.as_deref() == Some("word_boundary")
    {
        text_config.linebreak = Some("word_or_character".to_string());
    }
}

fn split_font_shorthand_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;

    for character in value.chars() {
        match character {
            '"' | '\'' => {
                if quote == Some(character) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(character);
                }
                current.push(character);
            }
            character if character.is_whitespace() && quote.is_none() => {
                if !current.trim().is_empty() {
                    tokens.push(
                        current
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if !current.trim().is_empty() {
        tokens.push(
            current
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string(),
        );
    }

    tokens
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
