use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{BuiImageConfig, BuiStyles, BuiTextConfig, BuiVisuals};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BuiNodeType {
    Node,
    Text,
    TextInput,
    Toggle,
    Button,
    Slider,
    Image,
}

pub fn kind_to_node_type(kind: &str) -> Result<BuiNodeType, String> {
    match kind {
        "node" => Ok(BuiNodeType::Node),
        "text" => Ok(BuiNodeType::Text),
        "text_input" => Ok(BuiNodeType::TextInput),
        "toggle" => Ok(BuiNodeType::Toggle),
        "button" => Ok(BuiNodeType::Button),
        "slider" => Ok(BuiNodeType::Slider),
        "image" => Ok(BuiNodeType::Image),
        other => Err(format!("Unsupported BUI kind '{other}'.")),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiActionBinding {
    pub event: String,
    pub emit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiBinding {
    pub target: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiStateVisual {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    pub styles: BuiStyles,
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    pub visuals: BuiVisuals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<BuiImageConfig>,
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
pub struct BuiDocument {
    pub version: String,
    pub scene_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<String>,
    #[serde(default, skip_serializing_if = "BuiStateModel::is_empty")]
    pub state_model: BuiStateModel,
    #[serde(default, skip_serializing_if = "BuiInteractionModel::is_empty")]
    pub interaction_model: BuiInteractionModel,
    #[serde(default, skip_serializing_if = "BuiResources::is_empty")]
    pub resources: BuiResources,
    pub root: BuiNode,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiStateModel {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, String>,
}

impl BuiStateModel {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiInteractionModel {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub actions: BTreeMap<String, Vec<BuiInteractionStep>>,
}

impl BuiInteractionModel {
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiInteractionStep {
    pub op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiResources {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, String>,
}

impl BuiResources {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiNode {
    pub id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<BuiActionBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BuiBinding>,
    #[serde(default, skip_serializing_if = "BuiLayout::is_empty")]
    pub layout: BuiLayout,
    #[serde(default, skip_serializing_if = "BuiStyle::is_empty")]
    pub style: BuiStyle,
    #[serde(default, skip_serializing_if = "BuiContent::is_empty")]
    pub content: BuiContent,
    #[serde(default, skip_serializing_if = "BuiSemantics::is_empty")]
    pub semantics: BuiSemantics,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_visuals: BTreeMap<String, BuiStateVisual>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<BuiNode>,
}

impl BuiNode {
    pub fn node_type(&self) -> BuiNodeType {
        kind_to_node_type(&self.kind).unwrap_or(BuiNodeType::Node)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiLayout {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    pub styles: BuiStyles,
}

impl BuiLayout {
    fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiStyle {
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    pub visuals: BuiVisuals,
}

impl BuiStyle {
    fn is_empty(&self) -> bool {
        self.visuals.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<BuiTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<BuiImageConfig>,
}

impl BuiContent {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiSemantics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub progress_fill: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slider: Option<BuiSliderSemantics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_view: Option<BuiScrollViewSemantics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropdown_group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropdown_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropdown_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropdown_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_fit: Option<BuiStageFitSemantics>,
}

impl BuiSemantics {
    fn is_empty(&self) -> bool {
        self.tab_group_name.is_none()
            && self.tab_binding_source.is_none()
            && self.tab_value.is_none()
            && self.progress_binding_source.is_none()
            && !self.progress_fill
            && self.list_binding_source.is_none()
            && self.slider.is_none()
            && self.scroll_view.is_none()
            && self.dropdown_group_name.is_none()
            && self.dropdown_binding_source.is_none()
            && self.dropdown_value.is_none()
            && self.dropdown_label.is_none()
            && self.stage_fit.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiSliderSemantics {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiScrollViewSemantics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuiStageFitSemantics {
    pub design_width: f32,
    pub design_height: f32,
    pub mode: String,
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
        state_visuals: BTreeMap::new(),
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
