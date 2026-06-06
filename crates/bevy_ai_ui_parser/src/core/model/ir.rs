use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    kind_to_node_type,
    node_type_to_kind,
    BuiActionBinding,
    BuiBinding,
    BuiDocument,
    BuiImageConfig,
    BuiNode,
    BuiStateVisual,
    BuiStyles,
    BuiTextConfig,
    BuiVisuals,
};

const EXPECTED_VERSION: &str = "2.0";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiIrDocument {
    pub(crate) version: String,
    pub(crate) scene_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) imports: Vec<String>,
    #[serde(default, skip_serializing_if = "BuiIrStateModel::is_empty")]
    state_model: BuiIrStateModel,
    #[serde(default, skip_serializing_if = "BuiIrResources::is_empty")]
    resources: BuiIrResources,
    pub(crate) root: BuiIrNode,
}

impl BuiIrDocument {
    pub(crate) fn from_compat_document(document: &BuiDocument) -> Self {
        Self {
            version: "3.0-ir".to_string(),
            scene_name: document.scene_name.clone(),
            imports: Vec::new(),
            state_model: BuiIrStateModel::default(),
            resources: BuiIrResources::default(),
            root: BuiIrNode::from_compat_node(&document.root),
        }
    }

    pub(crate) fn into_compat_document(self) -> Result<BuiDocument, String> {
        if self.version != "3.0-ir" {
            return Err(format!(
                "Unsupported BUI IR version '{}'. This parser expects version 3.0-ir.",
                self.version
            ));
        }

        Ok(BuiDocument {
            version: EXPECTED_VERSION.to_string(),
            scene_name: self.scene_name,
            root: self.root.into_compat_node()?,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiIrStateModel {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    values: HashMap<String, String>,
}

impl BuiIrStateModel {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiIrResources {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    values: HashMap<String, String>,
}

impl BuiIrResources {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiIrNode {
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
    #[serde(default, skip_serializing_if = "BuiIrLayout::is_empty")]
    pub(crate) layout: BuiIrLayout,
    #[serde(default, skip_serializing_if = "BuiIrStyle::is_empty")]
    pub(crate) style: BuiIrStyle,
    #[serde(default, skip_serializing_if = "BuiIrContent::is_empty")]
    pub(crate) content: BuiIrContent,
    #[serde(default, skip_serializing_if = "BuiIrSemantics::is_empty")]
    pub(crate) semantics: BuiIrSemantics,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) state_visuals: HashMap<String, BuiStateVisual>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) children: Vec<BuiIrNode>,
}

impl BuiIrNode {
    fn from_compat_node(node: &BuiNode) -> Self {
        Self {
            id: node.id.clone(),
            kind: node_type_to_kind(&node.node_type).to_string(),
            markers: node.custom_tags.clone(),
            classes: Vec::new(),
            actions: node.actions.clone(),
            bindings: node.bindings.clone(),
            layout: BuiIrLayout {
                styles: node.styles.clone(),
            },
            style: BuiIrStyle {
                visuals: node.visuals.clone(),
            },
            content: BuiIrContent::from_compat_node(node),
            semantics: BuiIrSemantics::from_compat_node(node),
            state_visuals: node.state_visuals.clone(),
            children: node
                .children
                .iter()
                .map(BuiIrNode::from_compat_node)
                .collect(),
        }
    }

    fn into_compat_node(self) -> Result<BuiNode, String> {
        let mut custom_tags = self.markers;
        custom_tags.extend(self.classes.into_iter().map(|class| format!("class:{class}")));

        Ok(BuiNode {
            id: self.id,
            node_type: kind_to_node_type(&self.kind)?,
            custom_tags,
            actions: self.actions,
            bindings: self.bindings,
            tab_group_name: self.semantics.tab_group_name,
            tab_binding_source: self.semantics.tab_binding_source,
            tab_value: self.semantics.tab_value,
            progress_binding_source: self.semantics.progress_binding_source,
            progress_fill: self.semantics.progress_fill,
            list_binding_source: self.semantics.list_binding_source,
            state_visuals: self.state_visuals,
            styles: self.layout.styles,
            visuals: self.style.visuals,
            text_config: self.content.text,
            image_config: self.content.image,
            children: self
                .children
                .into_iter()
                .map(BuiIrNode::into_compat_node)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiIrLayout {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    pub(crate) styles: BuiStyles,
}

impl BuiIrLayout {
    fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiIrStyle {
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    pub(crate) visuals: BuiVisuals,
}

impl BuiIrStyle {
    fn is_empty(&self) -> bool {
        self.visuals.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiIrContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<BuiTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<BuiImageConfig>,
}

impl BuiIrContent {
    fn from_compat_node(node: &BuiNode) -> Self {
        Self {
            text: node.text_config.clone(),
            image: node.image_config.clone(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiIrSemantics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tab_group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tab_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tab_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress_binding_source: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    progress_fill: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    list_binding_source: Option<String>,
}

impl BuiIrSemantics {
    fn from_compat_node(node: &BuiNode) -> Self {
        Self {
            tab_group_name: node.tab_group_name.clone(),
            tab_binding_source: node.tab_binding_source.clone(),
            tab_value: node.tab_value.clone(),
            progress_binding_source: node.progress_binding_source.clone(),
            progress_fill: node.progress_fill,
            list_binding_source: node.list_binding_source.clone(),
        }
    }

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
