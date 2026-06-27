use crate::core::model::BuiNode;

pub(crate) struct SvgAssetEntry {
    pub(crate) key: String,
    pub(crate) svg_markup: String,
    pub(crate) render_width: u32,
    pub(crate) render_height: u32,
}

pub(crate) fn extract_svg_markup(svg_node: roxmltree::Node<'_, '_>) -> String {
    serialize_xml_node(svg_node)
}

pub(crate) fn svg_asset_key(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
    index: usize,
) -> String {
    let svg_id = svg_node
        .attribute("id")
        .filter(|id| !id.trim().is_empty());
    match svg_id {
        Some(id) => format!("{}__{}", parent.id, id),
        None => format!("{}__svg_{}", parent.id, index),
    }
}

pub(crate) fn svg_viewbox_size(svg_node: roxmltree::Node<'_, '_>) -> (f32, f32) {
    if let Some(viewbox) = svg_node.attribute("viewBox") {
        let parts: Vec<&str> = viewbox.split_whitespace().collect();
        if parts.len() >= 4 {
            if let (Ok(w), Ok(h)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                if w > 0.0 && h > 0.0 {
                    return (w, h);
                }
            }
        }
    }
    let w = parse_dimension_attribute(svg_node, "width").unwrap_or(32.0);
    let h = parse_dimension_attribute(svg_node, "height").unwrap_or(32.0);
    (w.max(1.0), h.max(1.0))
}

pub(crate) fn svg_render_scale(svg_node: roxmltree::Node<'_, '_>) -> (u32, u32) {
    let (w, h) = svg_viewbox_size(svg_node);
    ((w * 2.0).round() as u32, (h * 2.0).round() as u32)
}

fn parse_dimension_attribute(
    svg_node: roxmltree::Node<'_, '_>,
    attr: &str,
) -> Option<f32> {
    svg_node
        .attribute(attr)
        .and_then(|v| {
            v.strip_suffix("px")
                .or_else(|| v.strip_suffix("pt"))
                .or_else(|| Some(v))
        })
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| *v > 0.0)
}

fn serialize_xml_node(node: roxmltree::Node<'_, '_>) -> String {
    let mut output = String::new();
    serialize_node_recursive(node, &mut output);
    output
}

fn serialize_node_recursive(node: roxmltree::Node<'_, '_>, output: &mut String) {
    if node.is_element() {
        let tag = node.tag_name().name();
        output.push('<');
        output.push_str(tag);
        if tag == "svg" {
            output.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
        }
        for attr in node.attributes() {
            output.push(' ');
            output.push_str(attr.name());
            output.push('=');
            output.push('"');
            output.push_str(&escape_xml_value(attr.value()));
            output.push('"');
        }
        let has_children = node
            .children()
            .any(|c| c.is_element() || c.text().is_some());
        if !has_children {
            output.push_str("/>");
            return;
        }
        output.push('>');
        for child in node.children() {
            serialize_node_recursive(child, output);
        }
        output.push_str("</");
        output.push_str(tag);
        output.push('>');
    } else if let Some(text) = node.text() {
        output.push_str(&escape_xml_text(text));
    }
}

fn escape_xml_value(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
