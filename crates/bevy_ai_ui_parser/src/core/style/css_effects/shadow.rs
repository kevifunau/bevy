use crate::core::{
    model::{bui_node, BuiBoxShadowConfig, BuiNode, BuiTextShadowConfig},
    style::{
        css_sizing::{css_size_tokens, is_simple_css_size},
        css_values::{css_color, split_css_layers},
    },
};

pub(crate) fn css_text_shadow(value: &str) -> Option<BuiTextShadowConfig> {
    let layer = split_css_layers(value).into_iter().next()?;
    let mut offset_x = None;
    let mut offset_y = None;
    let mut color = css_color(&layer);

    for token in css_size_tokens(&layer) {
        if color.is_none()
            && let Some(parsed) = css_color(&token)
        {
            color = Some(parsed);
            continue;
        }

        if let Some(number) = css_text_shadow_length(&token) {
            if offset_x.is_none() {
                offset_x = Some(number);
            } else if offset_y.is_none() {
                offset_y = Some(number);
                break;
            }
        }
    }

    (offset_x.is_some() || offset_y.is_some() || color.is_some()).then_some(BuiTextShadowConfig {
        offset_x,
        offset_y,
        color,
    })
}

fn css_text_shadow_length(token: &str) -> Option<f32> {
    let token = token.trim();
    if token == "0" {
        return Some(0.0);
    }

    token
        .strip_suffix("px")
        .and_then(|part| part.parse::<f32>().ok())
}

pub(crate) fn css_box_shadow(value: &str) -> Option<BuiBoxShadowConfig> {
    let inset = value.split_whitespace().any(|token| token == "inset");
    let mut offset_x = None;
    let mut offset_y = None;
    let mut blur_radius = None;
    let mut spread_radius = None;
    let (mut color, size_source) = css_extract_box_shadow_color(value)
        .map(|(color, source)| (Some(color), source))
        .unwrap_or((None, value.to_string()));

    let tokens: Vec<&str> = size_source
        .split_whitespace()
        .filter(|token| !matches!(*token, "inset"))
        .collect();

    let mut sizes = Vec::new();
    for token in tokens {
        if color.is_none()
            && let Some(parsed) = css_color(token)
        {
            color = Some(parsed);
            continue;
        }

        if is_simple_css_size(token) {
            sizes.push(token.to_string());
        }
    }

    if let Some(value) = sizes.first() {
        offset_x = Some(value.clone());
    }
    if let Some(value) = sizes.get(1) {
        offset_y = Some(value.clone());
    }
    if let Some(value) = sizes.get(2) {
        blur_radius = Some(value.clone());
    }
    if let Some(value) = sizes.get(3) {
        spread_radius = Some(value.clone());
    }

    (offset_x.is_some()
        || offset_y.is_some()
        || blur_radius.is_some()
        || spread_radius.is_some()
        || color.is_some())
    .then_some(BuiBoxShadowConfig {
        inset,
        offset_x,
        offset_y,
        blur_radius,
        spread_radius,
        color,
    })
}

fn css_extract_box_shadow_color(value: &str) -> Option<(String, String)> {
    let value = value.trim();
    let candidates = [
        "color-mix(",
        "oklch(",
        "rgb(",
        "rgba(",
        "hsl(",
        "hsla(",
        "lab(",
        "lch(",
    ];

    for candidate in candidates {
        let Some(start) = value.find(candidate) else {
            continue;
        };
        let mut depth = 0usize;
        let mut end = None;
        for (offset, character) in value[start..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(start + offset + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end?;
        let candidate_value = value[start..end].trim();
        let color = css_color(candidate_value)?;
        let mut remaining = String::new();
        remaining.push_str(value[..start].trim());
        if !remaining.is_empty() && !value[end..].trim().is_empty() {
            remaining.push(' ');
        }
        remaining.push_str(value[end..].trim());
        return Some((color, remaining));
    }

    let mut tokens = value.split_whitespace().collect::<Vec<_>>();
    let color_index = tokens.iter().position(|token| css_color(token).is_some())?;
    let color = css_color(tokens[color_index])?;
    tokens.remove(color_index);
    Some((color, tokens.join(" ")))
}

pub(crate) fn css_box_shadow_layers(value: &str) -> Vec<BuiBoxShadowConfig> {
    split_css_layers(value)
        .into_iter()
        .filter_map(css_box_shadow)
        .take(4)
        .collect()
}

pub(crate) fn apply_box_shadow_fallback(node: &mut BuiNode, value: &str) {
    node.children.retain(|child| {
        !child
            .markers
            .iter()
            .any(|tag| tag == "css-box-shadow-layer")
    });

    let shadows = css_box_shadow_layers(value);
    if shadows.is_empty() {
        node.style.visuals.box_shadow = None;
        return;
    }

    let primary_index = shadows.iter().position(|shadow| !shadow.inset).unwrap_or(0);
    node.style.visuals.box_shadow = Some(shadows[primary_index].clone());

    if shadows.len() == 1 {
        return;
    }

    if node.layout.styles.position_type.is_none() {
        node.layout.styles.position_type = Some("relative".to_string());
    }

    for (index, shadow) in shadows.into_iter().enumerate() {
        if index == primary_index {
            continue;
        }
        push_box_shadow_layer(
            node,
            shadow,
            "css-box-shadow-layer",
            &format!("box_shadow_layer_{}", index + 1),
        );
    }
}

pub(crate) fn push_box_shadow_layer(
    node: &mut BuiNode,
    shadow: BuiBoxShadowConfig,
    custom_tag: &str,
    id_suffix: &str,
) {
    if node.layout.styles.position_type.is_none() {
        node.layout.styles.position_type = Some("relative".to_string());
    }

    let layer_count = node
        .children
        .iter()
        .filter(|child| {
            child
                .markers
                .iter()
                .any(|tag| tag == "css-box-shadow-layer" || tag == "css-filter-drop-shadow")
        })
        .count();

    let mut layer = bui_node(&format!("{}_{}", node.id, id_suffix), "node");
    layer.markers.push(custom_tag.to_string());
    layer.layout.styles.position_type = Some("absolute".to_string());
    layer.layout.styles.z_index = Some(format!("-{}", layer_count + 1));
    layer.style.visuals.box_shadow = Some(shadow);
    layer.style.visuals.border_radius = node.style.visuals.border_radius.clone();

    if let Some((left, right, top, bottom, border_radius)) = clip_contour_shadow_bounds(node) {
        layer.layout.styles.left = Some(left);
        layer.layout.styles.right = Some(right);
        layer.layout.styles.top = Some(top);
        layer.layout.styles.bottom = Some(bottom);
        layer.style.visuals.border_radius = Some(border_radius);
    } else {
        layer.layout.styles.left = Some("0".to_string());
        layer.layout.styles.right = Some("0".to_string());
        layer.layout.styles.top = Some("0".to_string());
        layer.layout.styles.bottom = Some("0".to_string());
    }

    node.children.insert(0, layer);
}

pub(crate) fn node_has_shadow_casting_paint(node: &BuiNode) -> bool {
    if let Some(color) = node.style.visuals.background_color.as_deref()
        && !color_is_fully_transparent(color)
    {
        return true;
    }

    if node.content.image.is_some() {
        return true;
    }

    if let Some(color) = node.style.visuals.border_color.as_deref()
        && !color_is_fully_transparent(color)
        && node
            .style
            .visuals
            .border_width
            .as_deref()
            .map(|width| width.trim() != "0" && width.trim() != "0px")
            .unwrap_or(false)
    {
        return true;
    }

    false
}

fn color_is_fully_transparent(color: &str) -> bool {
    let trimmed = color.trim();
    if trimmed.eq_ignore_ascii_case("transparent") {
        return true;
    }

    if let Some(alpha) = trimmed.strip_prefix('#') {
        return match alpha.len() {
            4 => alpha.ends_with('0'),
            8 => alpha.ends_with("00"),
            _ => false,
        };
    }

    false
}

fn clip_contour_shadow_bounds(node: &BuiNode) -> Option<(String, String, String, String, String)> {
    let contour = node.children.iter().find(|child| {
        child.markers.iter().any(|tag| tag == "css-clip-contour")
            && child.layout.styles.left.is_some()
            && child.layout.styles.right.is_some()
            && child.layout.styles.top.is_some()
            && child.layout.styles.bottom.is_some()
    })?;

    Some((
        contour.layout.styles.left.clone()?,
        contour.layout.styles.right.clone()?,
        contour.layout.styles.top.clone()?,
        contour.layout.styles.bottom.clone()?,
        contour
            .style
            .visuals
            .border_radius
            .clone()
            .unwrap_or_else(|| "44%".to_string()),
    ))
}
