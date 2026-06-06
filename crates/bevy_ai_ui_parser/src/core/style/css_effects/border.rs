use crate::core::{
    model::{BuiNode, bui_node},
    style::{
        css_sizing::css_first_size,
        css_values::css_color,
    },
};

pub(crate) fn apply_css_border(bui_node: &mut BuiNode, value: &str) {
    if let Some(width) = css_first_size(value) {
        bui_node.style.visuals.border_width = Some(width);
    }
    if let Some(color) = css_color(value) {
        bui_node.style.visuals.border_color = Some(color);
    }
}

pub(crate) fn apply_css_edge_border(bui_node: &mut BuiNode, edge: &str, value: &str) {
    if let Some(color) = css_color(value) {
        ensure_edge_border_node(bui_node, edge).style.visuals.background_color = Some(color);
    }
    apply_css_edge_border_width(bui_node, edge, value);
}

pub(crate) fn apply_css_edge_border_color(bui_node: &mut BuiNode, edge: &str, value: &str) {
    if let Some(color) = css_color(value) {
        ensure_edge_border_node(bui_node, edge).style.visuals.background_color = Some(color);
    }
}

pub(crate) fn apply_css_edge_border_width(bui_node: &mut BuiNode, edge: &str, value: &str) {
    let Some(width) = css_first_size(value) else {
        return;
    };

    let border = ensure_edge_border_node(bui_node, edge);
    match edge {
        "top" | "bottom" => border.layout.styles.height = Some(width),
        "left" | "right" => border.layout.styles.width = Some(width),
        _ => {}
    }
}

fn ensure_edge_border_node<'a>(node: &'a mut BuiNode, edge: &str) -> &'a mut BuiNode {
    let border_id = format!("{}_border_{edge}", node.id);
    if let Some(index) = node.children.iter().position(|child| child.id == border_id) {
        return &mut node.children[index];
    }

    if node.layout.styles.position_type.is_none() {
        node.layout.styles.position_type = Some("relative".to_string());
    }

    let mut border = bui_node(&border_id, "node");
    border.markers.push(format!("css-edge-border:{edge}"));
    border.layout.styles.position_type = Some("absolute".to_string());

    match edge {
        "top" => {
            border.layout.styles.left = Some("0".to_string());
            border.layout.styles.right = Some("0".to_string());
            border.layout.styles.top = Some("0".to_string());
        }
        "bottom" => {
            border.layout.styles.left = Some("0".to_string());
            border.layout.styles.right = Some("0".to_string());
            border.layout.styles.bottom = Some("0".to_string());
        }
        "left" => {
            border.layout.styles.left = Some("0".to_string());
            border.layout.styles.top = Some("0".to_string());
            border.layout.styles.bottom = Some("0".to_string());
        }
        "right" => {
            border.layout.styles.right = Some("0".to_string());
            border.layout.styles.top = Some("0".to_string());
            border.layout.styles.bottom = Some("0".to_string());
        }
        _ => {}
    }

    node.children.push(border);
    node.children
        .last_mut()
        .expect("just inserted edge border child")
}