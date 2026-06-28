pub(crate) fn sanitize_id(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '{' && chars.peek() == Some(&'{') {
            chars.next();
            output.push_str("{{");
            while let Some(token_character) = chars.next() {
                output.push(token_character);
                if token_character == '}' && chars.peek() == Some(&'}') {
                    output.push(chars.next().expect("peeked closing brace should exist"));
                    break;
                }
            }
        } else if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else {
            output.push('_');
        }
    }

    output.trim_matches('_').to_string()
}

pub(crate) fn pascal_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

pub(crate) fn format_price(value: &str) -> String {
    let mut reversed = String::new();
    for (index, character) in value.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(character);
    }
    reversed.chars().rev().collect()
}
