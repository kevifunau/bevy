use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{BuiImageConfig, BuiStyles, BuiTextConfig, BuiVisuals};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum BuiNodeType {
    Node,
    Text,
    TextInput,
    Toggle,
    Button,
    Image,
}

pub(crate) fn kind_to_node_type(kind: &str) -> Result<BuiNodeType, String> {
    match kind {
        "node" => Ok(BuiNodeType::Node),
        "text" => Ok(BuiNodeType::Text),
        "text_input" => Ok(BuiNodeType::TextInput),
        "toggle" => Ok(BuiNodeType::Toggle),
        "button" => Ok(BuiNodeType::Button),
        "image" => Ok(BuiNodeType::Image),
        other => Err(format!("Unsupported BUI kind '{other}'.")),
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<BuiImageConfig>,
}

impl BuiStateVisual {
    pub(crate) fn is_empty(&self) -> bool {
        self.styles.is_empty()
            && self.visuals.is_empty()
            && self.text_color.is_none()
            && self.image.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiDocument {
    pub(crate) version: String,
    pub(crate) scene_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) imports: Vec<String>,
    #[serde(default, skip_serializing_if = "BuiStateModel::is_empty")]
    pub(crate) state_model: BuiStateModel,
    #[serde(default, skip_serializing_if = "BuiResources::is_empty")]
    pub(crate) resources: BuiResources,
    pub(crate) root: BuiNode,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiStateModel {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) values: HashMap<String, String>,
}

impl BuiStateModel {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiResources {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    values: HashMap<String, String>,
}

impl BuiResources {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiNode {
    pub(crate) id: String,
    pub(crate) kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) markers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) actions: Vec<BuiActionBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bindings: Vec<BuiBinding>,
    #[serde(default, skip_serializing_if = "BuiLayout::is_empty")]
    pub(crate) layout: BuiLayout,
    #[serde(default, skip_serializing_if = "BuiStyle::is_empty")]
    pub(crate) style: BuiStyle,
    #[serde(default, skip_serializing_if = "BuiContent::is_empty")]
    pub(crate) content: BuiContent,
    #[serde(default, skip_serializing_if = "BuiSemantics::is_empty")]
    pub(crate) semantics: BuiSemantics,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) state_visuals: HashMap<String, BuiStateVisual>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) children: Vec<BuiNode>,
}

impl BuiNode {
    pub(crate) fn node_type(&self) -> BuiNodeType {
        kind_to_node_type(&self.kind).unwrap_or(BuiNodeType::Node)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiLayout {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    pub(crate) styles: BuiStyles,
}

impl BuiLayout {
    fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiStyle {
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    pub(crate) visuals: BuiVisuals,
}

impl BuiStyle {
    fn is_empty(&self) -> bool {
        self.visuals.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<BuiTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<BuiImageConfig>,
}

impl BuiContent {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiSemantics {
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
}

impl BuiSemantics {
    fn is_empty(&self) -> bool {
        self.tab_group_name.is_none()
            && self.tab_binding_source.is_none()
            && self.tab_value.is_none()
            && self.progress_binding_source.is_none()
            && !self.progress_fill
            && self.list_binding_source.is_none()
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub(crate) fn bui_node(id: &str, kind: &str) -> BuiNode {
    BuiNode {
        id: id.to_string(),
        kind: kind.to_string(),
        markers: Vec::new(),
        classes: Vec::new(),
        actions: Vec::new(),
        bindings: Vec::new(),
        layout: BuiLayout::default(),
        style: BuiStyle::default(),
        content: BuiContent::default(),
        semantics: BuiSemantics::default(),
        state_visuals: HashMap::new(),
        children: Vec::new(),
    }
}

pub(crate) fn text_node(
    id: &str,
    content: impl Into<String>,
    font_size: f32,
    font_color: &str,
    font_path: Option<&str>,
) -> BuiNode {
    let mut node = bui_node(id, "text");
    node.content.text = Some(BuiTextConfig {
        content: content.into(),
        placeholder: None,
        font_size,
        font_color: font_color.to_string(),
        font_path: font_path.map(str::to_string),
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

pub(crate) fn ensure_state_visual<'a>(
    node: &'a mut BuiNode,
    state: &str,
) -> &'a mut BuiStateVisual {
    node.state_visuals
        .entry(state.to_string())
        .or_insert_with(|| BuiStateVisual {
            styles: BuiStyles::default(),
            visuals: BuiVisuals::default(),
            text_color: None,
            image: None,
        })
}
