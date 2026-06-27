use bevy_ai_ui_parser::{
    BuiNode, BuiStyles, BuiVisuals, BuiTextConfig, BuiImageConfig,
    BuiSliderSemantics,
};

use crate::app_state::LibraryItem;

/// Create a BuiNode with sensible defaults for the given library item type.
pub fn create_library_node(item: &LibraryItem) -> BuiNode {
    let id = format!("node_{}", timestamp());

    match item {
        LibraryItem::Node => basic_node(&id, "node"),
        LibraryItem::Text => text_node(&id),
        LibraryItem::Button => button_node(&id),
        LibraryItem::Image => image_node(&id),
        LibraryItem::TextInput => text_input_node(&id),
        LibraryItem::Toggle => toggle_node(&id),
        LibraryItem::Slider => slider_node(&id),
        LibraryItem::Row => {
            let mut node = basic_node(&id, "node");
            node.layout.styles.display = Some("flex".to_string());
            node.layout.styles.flex_direction = Some("row".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.row_gap = Some("8px".to_string());
            node
        }
        LibraryItem::Column => {
            let mut node = basic_node(&id, "node");
            node.layout.styles.display = Some("flex".to_string());
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.row_gap = Some("8px".to_string());
            node
        }
        LibraryItem::ButtonWithText => {
            let mut button = button_node(&id);
            let text_id = format!("{}_text", id);
            button.children.push(text_node(&text_id));
            button
        }
    }
}

fn basic_node(id: &str, kind: &str) -> BuiNode {
    BuiNode {
        id: id.to_string(),
        kind: kind.to_string(),
        markers: vec![],
        classes: vec![],
        actions: vec![],
        bindings: vec![],
        layout: Default::default(),
        style: Default::default(),
        content: Default::default(),
        semantics: Default::default(),
        state_visuals: Default::default(),
        children: vec![],
    }
}

fn text_node(id: &str) -> BuiNode {
    let mut node = basic_node(id, "text");
    node.content.text = Some(BuiTextConfig {
        content: "Text".to_string(),
        placeholder: None,
        font_size: 16.0,
        font_color: "#FFFFFF".to_string(),
        font_path: None,
        font_weight: None,
        line_height: None,
        letter_spacing: None,
        text_align: None,
        text_shadow: None,
        linebreak: None,
        visible_width: None,
        allow_newlines: None,
    });
    node
}

fn button_node(id: &str) -> BuiNode {
    let mut node = basic_node(id, "button");
    node.layout.styles = BuiStyles {
        display: Some("flex".to_string()),
        flex_direction: Some("row".to_string()),
        align_items: Some("center".to_string()),
        justify_content: Some("center".to_string()),
        padding: Some("8px 16px".to_string()),
        width: Some("auto".to_string()),
        height: Some("auto".to_string()),
        ..Default::default()
    };
    node.style.visuals = BuiVisuals {
        background_color: Some("#4A90D9".to_string()),
        border_radius: Some("4px".to_string()),
        ..Default::default()
    };
    node
}

fn image_node(id: &str) -> BuiNode {
    let mut node = basic_node(id, "image");
    node.content.image = Some(BuiImageConfig {
        texture_path: "Asset/placeholder.png".to_string(),
        image_mode: None,
        background_size: None,
        background_position: None,
        background_repeat: None,
        atlas: None,
        slicer: None,
        flip_x: false,
        flip_y: false,
    });
    node.layout.styles.width = Some("100px".to_string());
    node.layout.styles.height = Some("100px".to_string());
    node
}

fn text_input_node(id: &str) -> BuiNode {
    let mut node = basic_node(id, "text_input");
    node.layout.styles = BuiStyles {
        display: Some("flex".to_string()),
        padding: Some("6px 10px".to_string()),
        width: Some("200px".to_string()),
        height: Some("auto".to_string()),
        ..Default::default()
    };
    node.style.visuals = BuiVisuals {
        background_color: Some("#2A2A2A".to_string()),
        border_color: Some("#555555".to_string()),
        border_width: Some("1px".to_string()),
        border_radius: Some("3px".to_string()),
        ..Default::default()
    };
    node
}

fn toggle_node(id: &str) -> BuiNode {
    let mut node = basic_node(id, "toggle");
    node.layout.styles = BuiStyles {
        width: Some("20px".to_string()),
        height: Some("20px".to_string()),
        ..Default::default()
    };
    node.style.visuals = BuiVisuals {
        background_color: Some("#333333".to_string()),
        border_color: Some("#666666".to_string()),
        border_width: Some("1px".to_string()),
        border_radius: Some("3px".to_string()),
        ..Default::default()
    };
    node
}

fn slider_node(id: &str) -> BuiNode {
    let mut node = basic_node(id, "slider");
    node.layout.styles = BuiStyles {
        width: Some("200px".to_string()),
        height: Some("4px".to_string()),
        ..Default::default()
    };
    node.style.visuals = BuiVisuals {
        background_color: Some("#444444".to_string()),
        border_radius: Some("2px".to_string()),
        ..Default::default()
    };
    node.semantics.slider = Some(BuiSliderSemantics {
        value: 50.0,
        min: 0.0,
        max: 100.0,
        step: Some(1.0),
        orientation: None,
    });
    node
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
