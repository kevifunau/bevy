use super::media::media_query_matches;

pub(crate) fn css_declarations(body: &str) -> Vec<(String, String)> {
    body.split(';')
        .filter_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                None
            } else {
                Some((name.to_string(), value.to_string()))
            }
        })
        .collect()
}

pub(super) fn style_blocks(html: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        rest = &rest[start..];
        let Some(tag_end) = rest.find('>') else {
            break;
        };
        rest = &rest[tag_end + 1..];
        let Some(end) = rest.find("</style>") else {
            break;
        };
        blocks.push(&rest[..end]);
        rest = &rest[end + "</style>".len()..];
    }
    blocks
}

pub(super) fn css_rules(css: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut rules = Vec::new();
    let bytes = css.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let selector_start = index;
        while index < bytes.len() && bytes[index] != b'{' {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let selector = css[selector_start..index].trim();
        index += 1;

        let body_start = index;
        let mut depth = 1;
        while index < bytes.len() && depth > 0 {
            match bytes[index] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        if depth != 0 {
            break;
        }
        let body = &css[body_start..index - 1];
        if selector.starts_with("@media") {
            if media_query_matches(selector) {
                rules.extend(css_rules(body));
            }
            continue;
        }
        if selector.starts_with('@') {
            continue;
        }
        let declarations = css_declarations(body);
        if !selector.is_empty() && !declarations.is_empty() {
            rules.push((selector.to_string(), declarations));
        }
    }

    rules
}
