use crate::core::{
    model::BuiNode,
    style::css_sizing::{css_eval_length_function, css_size_to_px, css_size_tokens},
};

pub(crate) fn normalize_token(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

pub(crate) fn css_font_size(value: &str) -> Option<f32> {
    if let Some(size) = css_eval_length_function(value) {
        return size.strip_suffix("px")?.parse::<f32>().ok();
    }

    css_size_tokens(value)
        .into_iter()
        .filter_map(|part| {
            part.strip_suffix("px")
                .and_then(|number| number.parse::<f32>().ok())
        })
        .next()
}

pub(crate) fn css_letter_spacing(value: &str) -> Option<f32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") || value == "0" {
        return Some(0.0);
    }

    value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
}

pub(crate) fn css_line_height(value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return None;
    }

    if value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
        .is_some()
    {
        return Some(value.to_string());
    }

    value
        .parse::<f32>()
        .ok()
        .filter(|line_height| *line_height > 0.0)
        .map(|line_height| line_height.to_string())
}

pub(crate) fn css_text_align(value: &str) -> Option<&str> {
    matches!(
        value,
        "left" | "center" | "right" | "justify" | "justified" | "start" | "end"
    )
    .then_some(value)
}

pub(crate) fn css_transform_scale(value: &str) -> Option<String> {
    let value = value.trim();
    let args = value
        .strip_prefix("scale(")
        .and_then(|value| value.strip_suffix(')'))?
        .trim();
    if args.is_empty() {
        return None;
    }
    if let Some((x, y)) = args.split_once(',') {
        let x = x.trim().parse::<f32>().ok()?;
        let y = y.trim().parse::<f32>().ok()?;
        return Some(format!("{x} {y}"));
    }

    let scale = args.parse::<f32>().ok()?;
    Some(format!("{scale} {scale}"))
}

pub(crate) fn apply_css_transform(bui_node: &mut BuiNode, value: &str) {
    let functions = css_transform_functions(value);
    for func in &functions {
        match func.name {
            "translate" | "translateX" | "translateY" => {
                if let Some(translation) = css_transform_translation(func) {
                    bui_node.layout.styles.ui_translation = Some(translation);
                }
            }
            "rotate" => {
                if let Some(rotation) = css_transform_rotation(func) {
                    bui_node.layout.styles.ui_rotation = Some(rotation);
                }
            }
            "scale" => {
                if let Some(scale) = css_transform_scale(&func.raw) {
                    bui_node.layout.styles.ui_scale = Some(scale);
                }
            }
            _ => {}
        }
    }
}

pub(crate) struct CssTransformFunction {
    pub(crate) name: &'static str,
    pub(crate) args: String,
    pub(crate) raw: String,
}

pub(crate) fn css_transform_functions(value: &str) -> Vec<CssTransformFunction> {
    let value = value.trim();
    let mut functions = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let bytes = value.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let segment = value[start..i + 1].trim();
                    start = i + 1;
                    if let Some((name, args)) = css_transform_function_split(segment) {
                        functions.push(CssTransformFunction {
                            name,
                            args,
                            raw: segment.to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    functions
}

fn css_transform_function_split(segment: &str) -> Option<(&'static str, String)> {
    let paren_pos = segment.find('(')?;
    let name = &segment[..paren_pos];
    let args = segment[paren_pos + 1..segment.len() - 1].trim().to_string();
    let name_static: &'static str = match name {
        "translate" => "translate",
        "translateX" => "translateX",
        "translateY" => "translateY",
        "rotate" => "rotate",
        "scale" => "scale",
        "scaleX" => "scaleX",
        "scaleY" => "scaleY",
        "skew" => "skew",
        "skewX" => "skewX",
        "skewY" => "skewY",
        _ => return None,
    };
    Some((name_static, args))
}

pub(crate) fn css_transform_translation(func: &CssTransformFunction) -> Option<String> {
    match func.name {
        "translate" => {
            let parts: Vec<&str> = func.args.split(',').map(|p| p.trim()).collect();
            match parts.as_slice() {
                [x] => Some(css_translate_value(x, "0")),
                [x, y] => Some(css_translate_value(x, y)),
                _ => None,
            }
        }
        "translateX" => Some(css_translate_value(&func.args, "0")),
        "translateY" => Some(css_translate_value("0", &func.args)),
        _ => None,
    }
}

fn css_translate_value(x: &str, y: &str) -> String {
    let x_resolved = css_resolve_translate_component(x);
    let y_resolved = css_resolve_translate_component(y);
    format!("{} {}", x_resolved, y_resolved)
}

fn css_resolve_translate_component(value: &str) -> String {
    let value = value.trim();
    if value == "0" {
        return "0px".to_string();
    }
    if value.ends_with('%')
        || value.ends_with("px")
        || value.ends_with("vw")
        || value.ends_with("vh")
    {
        return value.to_string();
    }
    if let Some(px) = css_size_to_px(value) {
        return format!("{px:.1}px");
    }
    value.to_string()
}

pub(crate) fn css_transform_rotation(func: &CssTransformFunction) -> Option<String> {
    let value = func.args.trim();
    if let Some(deg) = value.strip_suffix("deg") {
        let degrees: f32 = deg.trim().parse::<f32>().ok()?;
        return Some(format!("{degrees:.1}deg"));
    }
    if let Ok(degrees) = value.parse::<f32>() {
        return Some(format!("{degrees:.1}deg"));
    }
    None
}
