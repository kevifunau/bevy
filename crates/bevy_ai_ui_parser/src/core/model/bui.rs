use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    BuiImageConfig,
    BuiStyles,
    BuiTextConfig,
    BuiVisuals,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiDocument {
    pub(crate) version: String,
    pub(crate) scene_name: String,
    pub(crate) root: BuiNode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiNode {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) node_type: BuiNodeType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) custom_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) actions: Vec<BuiActionBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bindings: Vec<BuiBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tab_group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tab_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tab_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) progress_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub(crate) progress_fill: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) list_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) state_visuals: HashMap<String, BuiStateVisual>,
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    pub(crate) styles: BuiStyles,
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    pub(crate) visuals: BuiVisuals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text_config: Option<BuiTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image_config: Option<BuiImageConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) children: Vec<BuiNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum BuiNodeType {
    Node,
    Text,
    TextInput,
    Toggle,
    Button,
    Image,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiActionBinding {
    pub(crate) event: String,
    pub(crate) emit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiBinding {
    pub target: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiStateVisual {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    pub(crate) styles: BuiStyles,
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    pub(crate) visuals: BuiVisuals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text_color: Option<String>,
}

impl BuiStateVisual {
    pub(crate) fn is_empty(&self) -> bool {
        self.styles.is_empty() && self.visuals.is_empty() && self.text_color.is_none()
    }
}

pub(crate) fn node_type_to_kind(node_type: &BuiNodeType) -> &'static str {
    match node_type {
        BuiNodeType::Node => "node",
        BuiNodeType::Text => "text",
        BuiNodeType::TextInput => "text_input",
        BuiNodeType::Toggle => "toggle",
        BuiNodeType::Button => "button",
        BuiNodeType::Image => "image",
    }
}

pub(crate) fn kind_to_node_type(kind: &str) -> Result<BuiNodeType, String> {
    match kind {
        "node" => Ok(BuiNodeType::Node),
        "text" => Ok(BuiNodeType::Text),
        "text_input" => Ok(BuiNodeType::TextInput),
        "toggle" => Ok(BuiNodeType::Toggle),
        "button" => Ok(BuiNodeType::Button),
        "image" => Ok(BuiNodeType::Image),
        other => Err(format!("Unsupported BUI IR kind '{other}'.")),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}
