//! A JSON-driven UI parser plugin for Bevy.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use bevy_app::{App, Plugin, Startup, Update};
use bevy_asset::{io::AssetSourceId, AssetPath, AssetServer, Assets};
use bevy_camera::visibility::Visibility;
use bevy_color::{Color, Srgba};
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::*;
use bevy_image::{TextureAtlas, TextureAtlasLayout};
use bevy_input_focus::{
    tab_navigation::{TabGroup, TabIndex},
    AutoFocus, FocusCause, InputFocus,
};
use bevy_log::{error, info, warn};
use bevy_math::{Rot2, UVec2, Vec2};
use bevy_text::{
    EditableText, FontSize, FontSource, Justify, LetterSpacing, LineBreak, LineHeight, TextBounds,
    TextColor, TextCursorStyle, TextFont, TextLayout,
};
use bevy_ui::{
    prelude::*, widget::TextShadow, Checkable, Checked, FocusPolicy, RelativeCursorPosition,
};
use serde::{Deserialize, Serialize};

const EXPECTED_VERSION: &str = "2.0";
const OPENDESIGN_DEFAULT_VIEWPORT_WIDTH: f32 = 1280.0;
const OPENDESIGN_DEFAULT_VIEWPORT_HEIGHT: f32 = 720.0;

/// Plugin that parses BUI JSON and spawns a native Bevy UI tree.
pub struct AiUiPlugin {
    source: BuiSource,
}

impl AiUiPlugin {
    /// Load BUI JSON from a file path.
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::Path(path.into()),
        }
    }

    /// Load BUI JSON from an in-memory string.
    pub fn from_json(json: impl Into<String>) -> Self {
        Self {
            source: BuiSource::Inline(json.into()),
        }
    }

    /// Load an OpenDesign HTML artifact from a file path and compile it into BUI IR.
    pub fn from_html_path(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::HtmlPath(path.into()),
        }
    }

    /// Load an OpenDesign HTML artifact from an in-memory string and compile it into BUI IR.
    pub fn from_html(html: impl Into<String>) -> Self {
        Self {
            source: BuiSource::HtmlInline(html.into()),
        }
    }
}

/// Validate a BUI JSON string against the parser contract without spawning UI.
pub fn validate_bui_json_str(json: &str) -> Result<(), String> {
    parse_bui_document(json).map(|_| ())
}

/// Validate a BUI JSON file against the parser contract without spawning UI.
pub fn validate_bui_json_file(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read BUI JSON '{}': {error}", path.display()))?;

    validate_bui_json_str(&raw).map_err(|error| format!("{}: {error}", path.display()))
}

/// Validate a BUI 3.0-ir JSON string against the parser contract without spawning UI.
pub fn validate_bui_ir_json_str(json: &str) -> Result<(), String> {
    parse_bui_ir_document(json).and_then(|document| {
        let compat = document.into_compat_document()?;
        validate_bui_document(&compat)
    })
}

/// Validate a BUI 3.0-ir JSON file against the parser contract without spawning UI.
pub fn validate_bui_ir_json_file(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read BUI IR JSON '{}': {error}", path.display()))?;

    validate_bui_ir_json_str(&raw).map_err(|error| format!("{}: {error}", path.display()))
}

/// Compile an OpenDesign HTML artifact into formatted BUI JSON.
pub fn opendesign_html_to_bui_json_str(html: &str) -> Result<String, String> {
    let document = opendesign_html_to_bui_document(html)?;
    serde_json::to_string_pretty(&document)
        .map_err(|error| format!("Failed to serialize generated BUI JSON: {error}"))
}

/// Compile an OpenDesign HTML artifact into formatted BUI IR JSON.
pub fn opendesign_html_to_bui_ir_json_str(html: &str) -> Result<String, String> {
    let document = opendesign_html_to_bui_document(html)?;
    let ir_document = BuiIrDocument::from_compat_document(&document);
    serde_json::to_string_pretty(&ir_document)
        .map_err(|error| format!("Failed to serialize generated BUI IR JSON: {error}"))
}

/// Compile an OpenDesign HTML artifact file into formatted BUI JSON.
pub fn opendesign_html_file_to_bui_json(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "Failed to read OpenDesign HTML '{}': {error}",
            path.display()
        )
    })?;

    opendesign_html_to_bui_json_str(&raw).map_err(|error| format!("{}: {error}", path.display()))
}

/// Compile an OpenDesign HTML artifact file into formatted BUI IR JSON.
pub fn opendesign_html_file_to_bui_ir_json(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "Failed to read OpenDesign HTML '{}': {error}",
            path.display()
        )
    })?;

    opendesign_html_to_bui_ir_json_str(&raw)
        .map_err(|error| format!("{}: {error}", path.display()))
}

impl Plugin for AiUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AiUiSource(self.source.clone()))
            .init_resource::<BuiStateStore>()
            .add_message::<BuiActionTriggered>()
            .add_message::<BuiBindingUpdate>()
            .add_message::<BuiStateSet>()
            .add_systems(Startup, spawn_bui_scene)
            .add_systems(
                Update,
                (
                    material_shader_notice_system,
                    text_input_proxy_focus_system,
                    sync_text_input_mirror_system,
                    dispatch_bui_tab_selection_system,
                    sync_bui_tab_selected_state_system,
                    sync_bui_progress_groups_system,
                    sync_bui_list_groups_system,
                    toggle_interaction_system,
                    update_toggle_visual_system,
                    apply_bui_visual_states_system,
                    dispatch_bui_actions_system,
                    apply_bui_state_updates_system,
                    apply_bui_binding_updates_system,
                    resolve_ui_target_camera_system,
                ),
            );
    }
}

#[derive(Resource, Clone)]
struct AiUiSource(BuiSource);

#[derive(Clone)]
enum BuiSource {
    Path(PathBuf),
    Inline(String),
    HtmlPath(PathBuf),
    HtmlInline(String),
}

/// Stable id copied from the BUI node's `id` field.
#[derive(Component, Debug, Clone)]
pub struct BuiId(pub String);

/// Logic tags copied from the BUI node's `custom_tags` field.
#[derive(Component, Debug, Clone)]
pub struct BuiLogicTags(pub Vec<String>);

/// Declarative action bindings copied from the BUI node's `actions` field.
#[derive(Component, Debug, Clone)]
pub struct BuiActions(pub Vec<BuiActionBinding>);

/// Declarative data bindings copied from the BUI node's `bindings` field.
#[derive(Component, Debug, Clone)]
pub struct BuiBindings(pub Vec<BuiBinding>);

#[derive(Component, Debug, Clone)]
struct BuiTabGroupDefinition {
    group: String,
    source: String,
}

#[derive(Component, Debug, Clone)]
struct BuiTabItem {
    group: String,
    value: String,
}

#[derive(Component, Debug, Clone)]
struct BuiProgressGroup {
    source: String,
}

#[derive(Component, Debug, Clone, Copy)]
struct BuiProgressFill;

#[derive(Component, Debug, Clone)]
struct BuiListDefinition {
    source: String,
    item_template: BuiNode,
}

/// Runtime message emitted when a BUI action is triggered by interaction.
#[derive(Message, Debug, Clone)]
pub struct BuiActionTriggered {
    /// The entity whose interaction triggered the action.
    pub entity: Entity,
    /// Stable BUI id copied from the source node.
    pub id: String,
    /// The emitted action name declared in JSON.
    pub action: String,
    /// The interaction trigger that fired this action.
    pub trigger: BuiActionTrigger,
}

/// Runtime message emitted by game systems to update declarative BUI bindings.
#[derive(Message, Debug, Clone)]
pub struct BuiBindingUpdate {
    /// The binding source key, for example `hero.power_value`.
    pub source: String,
    /// The value to push into matching bindings.
    pub value: BuiBindingValue,
}

/// Supported runtime values for declarative BUI bindings.
#[derive(Debug, Clone, PartialEq)]
pub enum BuiBindingValue {
    /// String-like content, currently applied to `text.content`.
    Text(String),
    /// String list content, currently applied to semantic list nodes.
    StringList(Vec<String>),
    /// Object list content, currently applied to semantic list nodes with keyed placeholders.
    ObjectList(Vec<HashMap<String, String>>),
    /// Numeric-like content, currently applied to size-like targets.
    Number(f32),
    /// Boolean-like content, currently applied to `visibility`.
    Bool(bool),
    /// Color-like content, currently applied to visual or text color targets.
    Color(String),
}

/// Scene-level runtime state store used to resolve declarative BUI bindings.
#[derive(Resource, Debug, Default)]
pub struct BuiStateStore(pub HashMap<String, BuiBindingValue>);

/// Runtime message emitted by game systems to update the scene-level BUI state store.
#[derive(Message, Debug, Clone)]
pub struct BuiStateSet {
    /// The state key to update, for example `hero.power_value`.
    pub key: String,
    /// The new value to store under `key`.
    pub value: BuiBindingValue,
}

/// Supported declarative interaction triggers for BUI actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiActionTrigger {
    /// Trigger when the node is pressed.
    Press,
    /// Trigger when the node enters the hovered state.
    HoverEnter,
    /// Trigger when the node exits the hovered state.
    HoverExit,
}

/// Marker data for nodes that request a custom UI material shader.
#[derive(Component, Debug, Clone)]
pub struct BuiMaterialShader {
    /// Shader asset path copied from `visuals.material_shader`.
    pub path: String,
}

/// Marker for JSON nodes parsed as toggle widgets.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuiToggle;

/// Marker for JSON nodes parsed as text input widgets.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuiTextInput;

#[derive(Component, Debug, Clone, Copy)]
struct BuiTextInputProxy {
    target: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
struct BuiTextInputMirror {
    target: Entity,
}

#[derive(Component, Debug, Clone)]
struct PendingUiTargetCamera {
    target_name: String,
}

/// Entity id of the spawned BUI root.
#[derive(Resource, Debug, Clone)]
pub struct BuiRootEntity(pub Entity);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiDocument {
    version: String,
    scene_name: String,
    root: BuiNode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiIrDocument {
    version: String,
    scene_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    imports: Vec<String>,
    #[serde(default, skip_serializing_if = "BuiIrStateModel::is_empty")]
    state_model: BuiIrStateModel,
    #[serde(default, skip_serializing_if = "BuiIrResources::is_empty")]
    resources: BuiIrResources,
    root: BuiIrNode,
}

impl BuiIrDocument {
    fn from_compat_document(document: &BuiDocument) -> Self {
        Self {
            version: "3.0-ir".to_string(),
            scene_name: document.scene_name.clone(),
            imports: Vec::new(),
            state_model: BuiIrStateModel::default(),
            resources: BuiIrResources::default(),
            root: BuiIrNode::from_compat_node(&document.root),
        }
    }

    fn into_compat_document(self) -> Result<BuiDocument, String> {
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
struct BuiIrNode {
    id: String,
    kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    markers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    actions: Vec<BuiActionBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bindings: Vec<BuiBinding>,
    #[serde(default, skip_serializing_if = "BuiIrLayout::is_empty")]
    layout: BuiIrLayout,
    #[serde(default, skip_serializing_if = "BuiIrStyle::is_empty")]
    style: BuiIrStyle,
    #[serde(default, skip_serializing_if = "BuiIrContent::is_empty")]
    content: BuiIrContent,
    #[serde(default, skip_serializing_if = "BuiIrSemantics::is_empty")]
    semantics: BuiIrSemantics,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    state_visuals: HashMap<String, BuiStateVisual>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<BuiIrNode>,
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
struct BuiIrLayout {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    styles: BuiStyles,
}

impl BuiIrLayout {
    fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiIrStyle {
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    visuals: BuiVisuals,
}

impl BuiIrStyle {
    fn is_empty(&self) -> bool {
        self.visuals.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiIrContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text: Option<BuiTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image: Option<BuiImageConfig>,
}

impl BuiIrContent {
    fn from_compat_node(node: &BuiNode) -> Self {
        Self {
            text: node.text_config.clone(),
            image: node.image_config.clone(),
        }
    }

    fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiIrSemantics {
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiNode {
    id: String,
    #[serde(rename = "type")]
    node_type: BuiNodeType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    custom_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    actions: Vec<BuiActionBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bindings: Vec<BuiBinding>,
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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    state_visuals: HashMap<String, BuiStateVisual>,
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    styles: BuiStyles,
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    visuals: BuiVisuals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_config: Option<BuiTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image_config: Option<BuiImageConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<BuiNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum BuiNodeType {
    Node,
    Text,
    TextInput,
    Toggle,
    Button,
    Image,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Declarative action binding parsed from a BUI node.
pub struct BuiActionBinding {
    event: String,
    emit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Declarative binding parsed from a BUI node.
pub struct BuiBinding {
    /// The target property path to update, for example `text.content`.
    pub target: String,
    /// The source key resolved by game-side binding systems.
    pub source: String,
}

#[derive(Component, Debug, Clone)]
struct BuiVisualStateDefinitions {
    states: HashMap<String, BuiStateVisual>,
}

/// Explicit visual state name applied to a BUI node at runtime.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct BuiVisualState(pub String);

/// Marks a BUI node as disabled for declarative state resolution and action dispatch.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuiDisabled;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiStateVisual {
    #[serde(default, skip_serializing_if = "BuiStyles::is_empty")]
    styles: BuiStyles,
    #[serde(default, skip_serializing_if = "BuiVisuals::is_empty")]
    visuals: BuiVisuals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_color: Option<String>,
}

impl BuiStateVisual {
    fn is_empty(&self) -> bool {
        self.styles.is_empty() && self.visuals.is_empty() && self.text_color.is_none()
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiStyles {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    visibility: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    overflow_clip_margin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    margin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    margin_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    margin_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    margin_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    margin_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flex_direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flex_wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flex_grow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flex_shrink: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flex_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    row_gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    column_gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    justify_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    justify_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    align_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    align_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    align_self: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    justify_self: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ui_translation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ui_scale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ui_rotation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tab_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tab_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auto_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relative_cursor_position: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ui_target_camera: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    position_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fixed_node: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    global_z_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grid_template_columns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grid_template_rows: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grid_column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grid_row: Option<String>,
}

impl BuiStyles {
    fn is_empty(&self) -> bool {
        self.display.is_none()
            && self.visibility.is_none()
            && self.width.is_none()
            && self.height.is_none()
            && self.aspect_ratio.is_none()
            && self.min_width.is_none()
            && self.min_height.is_none()
            && self.max_width.is_none()
            && self.max_height.is_none()
            && self.left.is_none()
            && self.right.is_none()
            && self.top.is_none()
            && self.bottom.is_none()
            && self.overflow.is_none()
            && self.overflow_clip_margin.is_none()
            && self.margin.is_none()
            && self.margin_left.is_none()
            && self.margin_right.is_none()
            && self.margin_top.is_none()
            && self.margin_bottom.is_none()
            && self.padding.is_none()
            && self.padding_left.is_none()
            && self.padding_right.is_none()
            && self.padding_top.is_none()
            && self.padding_bottom.is_none()
            && self.flex_direction.is_none()
            && self.flex_wrap.is_none()
            && self.flex_grow.is_none()
            && self.flex_shrink.is_none()
            && self.flex_basis.is_none()
            && self.row_gap.is_none()
            && self.column_gap.is_none()
            && self.justify_content.is_none()
            && self.justify_items.is_none()
            && self.align_content.is_none()
            && self.align_items.is_none()
            && self.align_self.is_none()
            && self.justify_self.is_none()
            && self.ui_translation.is_none()
            && self.ui_scale.is_none()
            && self.ui_rotation.is_none()
            && self.tab_group.is_none()
            && self.tab_index.is_none()
            && self.auto_focus.is_none()
            && self.relative_cursor_position.is_none()
            && self.ui_target_camera.is_none()
            && self.position_type.is_none()
            && self.fixed_node.is_none()
            && self.z_index.is_none()
            && self.global_z_index.is_none()
            && self.grid_template_columns.is_none()
            && self.grid_template_rows.is_none()
            && self.grid_column.is_none()
            && self.grid_row.is_none()
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiVisuals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    border_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    border_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    material_shader: Option<String>,
}

impl BuiVisuals {
    fn is_empty(&self) -> bool {
        self.background_color.is_none()
            && self.border_color.is_none()
            && self.border_width.is_none()
            && self.border_radius.is_none()
            && self.material_shader.is_none()
    }
}

#[derive(Component, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiTextConfig {
    content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    font_size: f32,
    font_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    font_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_shadow: Option<BuiTextShadowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    linebreak: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    visible_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    allow_newlines: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiTextShadowConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset_x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset_y: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiImageConfig {
    texture_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    atlas: Option<BuiTextureAtlasConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    slicer: Option<BuiTextureSlicerConfig>,
    #[serde(default, skip_serializing_if = "is_false")]
    flip_x: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    flip_y: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiTextureAtlasConfig {
    tile_width: u32,
    tile_height: u32,
    columns: u32,
    rows: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding_x: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    padding_y: Option<u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiTextureSlicerConfig {
    border: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    center_scale_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sides_scale_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stretch_value: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_corner_scale: Option<f32>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

fn spawn_bui_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    source: Res<AiUiSource>,
) {
    match load_bui_document(&source.0) {
        Ok(document) => {
            info!("Spawning BUI scene '{}'.", document.scene_name);
            match spawn_bui_tree(
                &mut commands,
                &asset_server,
                &mut texture_atlases,
                &document,
            ) {
                Ok(root) => {
                    commands.insert_resource(BuiRootEntity(root));
                }
                Err(error) => {
                    error!("{error}");
                    spawn_error_text(&mut commands, error);
                }
            }
        }
        Err(error) => {
            error!("{error}");
            spawn_error_text(&mut commands, error);
        }
    }
}

fn load_bui_document(source: &BuiSource) -> Result<BuiDocument, String> {
    match source {
        BuiSource::Path(path) => {
            let raw = fs::read_to_string(path).map_err(|error| {
                format!("Failed to read BUI JSON '{}': {error}", path.display())
            })?;
            parse_bui_document(&raw)
        }
        BuiSource::Inline(json) => parse_bui_document(json),
        BuiSource::HtmlPath(path) => {
            let raw = fs::read_to_string(path).map_err(|error| {
                format!(
                    "Failed to read OpenDesign HTML '{}': {error}",
                    path.display()
                )
            })?;
            opendesign_html_to_bui_document(&raw)
        }
        BuiSource::HtmlInline(html) => opendesign_html_to_bui_document(html),
    }
}

fn parse_bui_document(raw: &str) -> Result<BuiDocument, String> {
    let version = detect_bui_version(raw)?;
    let document = if version == "3.0-ir" {
        parse_bui_ir_document(raw)?.into_compat_document()?
    } else {
        serde_json::from_str(raw).map_err(|error| format!("Invalid BUI JSON: {error}"))?
    };

    validate_bui_document(&document)?;

    Ok(document)
}

fn parse_bui_ir_document(raw: &str) -> Result<BuiIrDocument, String> {
    serde_json::from_str(raw).map_err(|error| format!("Invalid BUI IR JSON: {error}"))
}

fn detect_bui_version(raw: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct VersionProbe {
        version: String,
    }

    let probe: VersionProbe =
        serde_json::from_str(raw).map_err(|error| format!("Invalid BUI JSON: {error}"))?;
    Ok(probe.version)
}

fn opendesign_html_to_bui_document(html: &str) -> Result<BuiDocument, String> {
    let stylesheet = OpenDesignStylesheet::parse(html);
    let fragment = extract_opendesign_fragment(html)?;
    let wrapped = format!("<bui_root>{fragment}</bui_root>");
    let parsed = roxmltree::Document::parse(&wrapped)
        .map_err(|error| format!("Failed to parse OpenDesign HTML fragment: {error}"))?;

    let overlay = parsed
        .descendants()
        .find(|node| has_class(*node, "overlay"));

    let root_node = overlay.or_else(|| {
        parsed
            .descendants()
            .find(|node| has_class(*node, "game-stage"))
    });

    let root_node = root_node
        .ok_or_else(|| "OpenDesign HTML is missing a recognized root container (.overlay or .game-stage).".to_string())?;

    if overlay.is_none() {
        return opendesign_html_to_generic_bui_document(&stylesheet, root_node);
    }

    let overlay = root_node;
    let panel_source = overlay
        .descendants()
        .find(|node| has_class(*node, "panel"))
        .ok_or_else(|| "OpenDesign HTML is missing a .panel node.".to_string())?;
    let panel_header_source = panel_source
        .descendants()
        .find(|node| has_class(*node, "panel-header"));
    let title_board_source = panel_header_source.and_then(|panel_header_source| {
        panel_header_source
            .descendants()
            .find(|node| has_class(*node, "title-board"))
    });
    let title_text_source = title_board_source.and_then(|title_board_source| {
        title_board_source
            .descendants()
            .find(|node| has_class(*node, "title-text"))
    });
    let close_button_source = panel_header_source.and_then(|panel_header_source| {
        panel_header_source
            .descendants()
            .find(|node| has_class(*node, "close-btn"))
    });
    let title_board_source = panel_header_source
        .and(title_board_source);
    let title_text_source = title_board_source.and(title_text_source);
    let shop_body_source = panel_source
        .descendants()
        .find(|node| has_class(*node, "shop-body"));
    let shop_scroll_source = shop_body_source.and_then(|shop_body_source| {
        shop_body_source
            .descendants()
            .find(|node| has_class(*node, "shop-scroll"))
    });
    let foot_hint_source = panel_source
        .descendants()
        .find(|node| has_class(*node, "foot-hint"));

    let (
        Some(panel_header_source),
        Some(title_board_source),
        Some(title_text_source),
        Some(close_button_source),
        Some(shop_body_source),
        Some(shop_scroll_source),
    ) = (
        panel_header_source,
        title_board_source,
        title_text_source,
        close_button_source,
        shop_body_source,
        shop_scroll_source,
    )
    else {
        return opendesign_html_to_generic_bui_document(&stylesheet, overlay);
    };

    let title = first_text_by_class(overlay, "title-text").unwrap_or_else(|| "UI".to_string());
    let footer = first_text_by_class(overlay, "foot-hint").unwrap_or_default();

    let mut root = bui_node("overlay_root", BuiNodeType::Node);
    apply_opendesign_preset(&mut root, OpenDesignPreset::OverlayRoot);
    apply_opendesign_styles(&stylesheet, &mut root, overlay);

    let mut panel = bui_node("panel", BuiNodeType::Node);
    apply_opendesign_preset(&mut panel, OpenDesignPreset::Panel);
    apply_opendesign_styles(&stylesheet, &mut panel, panel_source);

    let mut panel_header = bui_node("panel_header", BuiNodeType::Node);
    apply_opendesign_preset(&mut panel_header, OpenDesignPreset::PanelHeader);
    apply_opendesign_styles(&stylesheet, &mut panel_header, panel_header_source);

    let mut title_board = bui_node("title_board", BuiNodeType::Node);
    apply_opendesign_preset(&mut title_board, OpenDesignPreset::TitleBoard);
    apply_opendesign_styles(&stylesheet, &mut title_board, title_board_source);
    let mut title_text = text_node(
        "title_text",
        title,
        36.0,
        "#FFFFFF",
        Some("STHeiti Medium.ttc"),
    );
    apply_opendesign_styles(&stylesheet, &mut title_text, title_text_source);
    title_board.children.push(title_text);

    let mut close_btn = bui_node("close_btn", BuiNodeType::Button);
    close_btn.custom_tags.push("Action_Close_Shop".to_string());
    close_btn.actions.push(BuiActionBinding {
        event: "press".to_string(),
        emit: "close_shop_overlay".to_string(),
    });
    apply_opendesign_preset(&mut close_btn, OpenDesignPreset::CloseButton);
    apply_opendesign_styles(&stylesheet, &mut close_btn, close_button_source);
    close_btn.children.push(text_node(
        "close_btn_text",
        "X",
        22.0,
        "#FFFFFF",
        Some("STHeiti Medium.ttc"),
    ));

    panel_header.children.push(title_board);
    panel_header.children.push(close_btn);

    let mut shop_body = bui_node("shop_body", BuiNodeType::Node);
    apply_opendesign_preset(&mut shop_body, OpenDesignPreset::ShopBody);
    apply_opendesign_styles(&stylesheet, &mut shop_body, shop_body_source);

    let mut shop_scroll = bui_node("shop_scroll", BuiNodeType::Node);
    apply_opendesign_preset(&mut shop_scroll, OpenDesignPreset::ShopScroll);
    apply_opendesign_styles(&stylesheet, &mut shop_scroll, shop_scroll_source);

    for article in overlay
        .descendants()
        .filter(|node| has_class(*node, "shop-card"))
    {
        shop_scroll.children.push(shop_card_node(article, &stylesheet)?);
    }

    shop_body.children.push(shop_scroll);

    let mut foot_hint = bui_node("foot_hint", BuiNodeType::Node);
    apply_opendesign_preset(&mut foot_hint, OpenDesignPreset::FootHint);
    if let Some(foot_hint_source) = foot_hint_source {
        apply_opendesign_styles(&stylesheet, &mut foot_hint, foot_hint_source);
    }
    let mut foot_hint_text = text_node(
        "foot_hint_text",
        footer,
        12.0,
        "#79614B",
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(foot_hint_source) = foot_hint_source {
        apply_opendesign_styles(&stylesheet, &mut foot_hint_text, foot_hint_source);
    }
    foot_hint.children.push(foot_hint_text);

    panel.children.push(panel_header);
    panel.children.push(shop_body);
    panel.children.push(foot_hint);
    root.children.push(panel);

    let document = BuiDocument {
        version: EXPECTED_VERSION.to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}

fn opendesign_html_to_generic_bui_document(
    stylesheet: &OpenDesignStylesheet,
    overlay: roxmltree::Node<'_, '_>,
) -> Result<BuiDocument, String> {
    let mut id_counts = HashMap::new();
    let mut root = generic_element_node("overlay_root", BuiNodeType::Node, stylesheet, overlay);
    apply_opendesign_preset(&mut root, OpenDesignPreset::OverlayRoot);
    apply_opendesign_styles(stylesheet, &mut root, overlay);
    generic_append_children(&mut root, overlay, stylesheet, &mut id_counts);

    let document = BuiDocument {
        version: EXPECTED_VERSION.to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}

fn generic_append_children(
    parent: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
    id_counts: &mut HashMap<String, usize>,
) {
    let before_decls = stylesheet.matching_pseudo_declarations(dom_node, "before");
    if !before_decls.is_empty() {
        let mut pseudo_node = bui_node(&format!("{}_pseudo_before", parent.id), BuiNodeType::Node);
        pseudo_node.custom_tags.push("pseudo:before".to_string());
        for (name, value) in &before_decls {
            let value = stylesheet.resolve_value(value);
            apply_opendesign_declaration(&mut pseudo_node, name, &value);
        }
        parent.children.push(pseudo_node);
    }

    let mut direct_text_index = 0;

    for child in dom_node.children() {
        if child.is_element() {
            let id = generic_dom_id(child, id_counts);
            let node_type = generic_node_type(child);
            let mut child_node = generic_element_node(&id, node_type, stylesheet, child);
            generic_append_children(&mut child_node, child, stylesheet, id_counts);
            parent.children.push(child_node);
        } else if let Some(text) = child.text().map(str::trim).filter(|text| !text.is_empty()) {
            direct_text_index += 1;
            let mut text_child = text_node(
                &format!("{}_text_{}", parent.id, direct_text_index),
                text,
                16.0,
                "#3B2818",
                Some("Hiragino Sans GB.ttc"),
            );
            apply_opendesign_styles(stylesheet, &mut text_child, dom_node);
            parent.children.push(text_child);
        }
    }

    let after_decls = stylesheet.matching_pseudo_declarations(dom_node, "after");
    if !after_decls.is_empty() {
        let mut pseudo_node = bui_node(&format!("{}_pseudo_after", parent.id), BuiNodeType::Node);
        pseudo_node.custom_tags.push("pseudo:after".to_string());
        for (name, value) in &after_decls {
            let value = stylesheet.resolve_value(value);
            apply_opendesign_declaration(&mut pseudo_node, name, &value);
        }
        parent.children.push(pseudo_node);
    }
}

fn generic_element_node(
    id: &str,
    node_type: BuiNodeType,
    stylesheet: &OpenDesignStylesheet,
    dom_node: roxmltree::Node<'_, '_>,
) -> BuiNode {
    let mut node = bui_node(id, node_type);

    if let Some(classes) = dom_node.attribute("class") {
        node.custom_tags.extend(
            classes
                .split_whitespace()
                .filter(|class| !class.is_empty())
                .map(|class| format!("class:{class}")),
        );
    }

    if let Some(action) = dom_node.attribute("data-action") {
        node.actions.push(BuiActionBinding {
            event: "press".to_string(),
            emit: action.to_string(),
        });
    }

    apply_opendesign_styles(stylesheet, &mut node, dom_node);
    node
}

fn generic_node_type(dom_node: roxmltree::Node<'_, '_>) -> BuiNodeType {
    let tag = dom_node.tag_name().name();
    if tag == "button"
        || dom_node.attribute("role") == Some("button")
        || dom_node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(is_button_like_class))
    {
        return BuiNodeType::Button;
    }

    BuiNodeType::Node
}

fn is_button_like_class(class_name: &str) -> bool {
    class_name == "btn" || class_name.ends_with("-btn") || class_name.ends_with("-button")
}

fn generic_dom_id(
    dom_node: roxmltree::Node<'_, '_>,
    id_counts: &mut HashMap<String, usize>,
) -> String {
    let base = dom_node
        .attribute("id")
        .map(sanitize_id)
        .filter(|id| !id.is_empty())
        .or_else(|| {
            dom_node
                .attribute("class")
                .and_then(|classes| classes.split_whitespace().next())
                .map(sanitize_id)
                .filter(|id| !id.is_empty())
        })
        .unwrap_or_else(|| sanitize_id(dom_node.tag_name().name()));

    let count = id_counts.entry(base.clone()).or_default();
    *count += 1;

    if *count == 1 {
        base
    } else {
        format!("{base}_{}", *count)
    }
}

fn extract_opendesign_fragment(html: &str) -> Result<&str, String> {
    let overlay_start = html.find("<div class=\"overlay");
    let main_start = html.find("<main class=\"game-stage");

    let start = overlay_start
        .or(main_start)
        .ok_or_else(|| "OpenDesign HTML does not contain a recognized root container ('<div class=\"overlay' or '<main class=\"game-stage').".to_string())?;

    let visually_hidden_end = html[start..]
        .find("<p class=\"visually-hidden\"")
        .map(|offset| start + offset);

    let closing_main_end = html[start..]
        .find("</main>")
        .map(|offset| start + offset + "</main>".len());

    let end = visually_hidden_end
        .or(closing_main_end)
        .ok_or_else(|| {
            "OpenDesign HTML does not contain the expected closing marker after the root container."
                .to_string()
        })?;

    Ok(html[start..end].trim())
}

fn shop_card_node(
    article: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
) -> Result<BuiNode, String> {
    let item_id = article.attribute("data-item-id").unwrap_or("item");
    let id = sanitize_id(item_id);
    let asset_text = first_text_by_class(article, "asset-slot").unwrap_or_default();
    let item_name = first_text_by_class(article, "item-name").unwrap_or_default();
    let item_meta = first_text_by_class(article, "item-meta").unwrap_or_default();
    let item_bonus = first_text_by_class(article, "item-bonus").unwrap_or_default();
    let price = first_text_by_class(article, "price-tag")
        .or_else(|| article.attribute("data-price").map(format_price))
        .unwrap_or_default();

    let mut card = bui_node(&format!("shop_card_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut card, OpenDesignPreset::ShopCard);
    apply_opendesign_styles(stylesheet, &mut card, article);

    let mut item_main = bui_node(&format!("item_main_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut item_main, OpenDesignPreset::ItemMain);
    let item_main_source = article
        .descendants()
        .find(|node| has_class(*node, "item-main"));
    if let Some(source) = item_main_source {
        apply_opendesign_styles(stylesheet, &mut item_main, source);
    }

    let mut asset_stack = bui_node(&format!("asset_stack_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut asset_stack, OpenDesignPreset::AssetStack);
    let asset_stack_source = article
        .descendants()
        .find(|node| has_class(*node, "asset-stack"));
    if let Some(source) = asset_stack_source {
        apply_opendesign_styles(stylesheet, &mut asset_stack, source);
    }

    let mut asset_slot = bui_node(&format!("asset_slot_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut asset_slot, OpenDesignPreset::AssetSlot);
    let asset_slot_source = article
        .descendants()
        .find(|node| has_class(*node, "asset-slot"));
    if let Some(source) = asset_slot_source {
        apply_opendesign_styles(stylesheet, &mut asset_slot, source);
    }
    let mut asset_label = text_node(
        &format!("asset_slot_{id}_text"),
        asset_text,
        12.0,
        "#79614B",
        Some("Hiragino Sans GB.ttc"),
    );
    asset_label.styles.width = Some("72px".to_string());
    if let Some(text_config) = &mut asset_label.text_config {
        text_config.font_size = 11.0;
        text_config.linebreak = Some("word_or_character".to_string());
    }
    if let Some(source) = asset_slot_source {
        apply_opendesign_styles(stylesheet, &mut asset_label, source);
    }
    asset_slot.children.push(asset_label);

    let mut stars = bui_node(&format!("stars_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut stars, OpenDesignPreset::Stars);
    let stars_source = article.descendants().find(|node| has_class(*node, "stars"));
    if let Some(source) = stars_source {
        apply_opendesign_styles(stylesheet, &mut stars, source);
    }
    for index in 1..=4 {
        stars.children.push(text_node(
            &format!("star_{id}_{index}"),
            "★",
            18.0,
            if index == 1 { "#D89A1F" } else { "#79614BCC" },
            Some("Hiragino Sans GB.ttc"),
        ));
    }
    asset_stack.children.push(asset_slot);
    asset_stack.children.push(stars);

    let mut item_copy = bui_node(&format!("item_copy_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut item_copy, OpenDesignPreset::ItemCopy);
    let item_copy_source = article
        .descendants()
        .find(|node| has_class(*node, "item-copy"));
    if let Some(source) = item_copy_source {
        apply_opendesign_styles(stylesheet, &mut item_copy, source);
    }
    let item_name_source = article
        .descendants()
        .find(|node| has_class(*node, "item-name"));
    let mut item_name_node = text_node(
        &format!("item_name_{id}"),
        item_name,
        24.0,
        "#3B2818",
        Some("STHeiti Medium.ttc"),
    );
    if let Some(source) = item_name_source {
        apply_opendesign_styles(stylesheet, &mut item_name_node, source);
    }
    item_copy.children.push(item_name_node);
    let item_meta_source = article
        .descendants()
        .find(|node| has_class(*node, "item-meta"));
    let mut item_meta_node = text_node(
        &format!("item_meta_{id}"),
        item_meta,
        13.0,
        "#79614B",
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(source) = item_meta_source {
        apply_opendesign_styles(stylesheet, &mut item_meta_node, source);
    }
    item_copy.children.push(item_meta_node);
    let mut bonus = bui_node(&format!("item_bonus_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut bonus, OpenDesignPreset::ItemBonus);
    let item_bonus_source = article
        .descendants()
        .find(|node| has_class(*node, "item-bonus"));
    if let Some(source) = item_bonus_source {
        apply_opendesign_styles(stylesheet, &mut bonus, source);
    }
    let mut item_bonus_text = text_node(
        &format!("item_bonus_{id}_text"),
        item_bonus,
        12.0,
        "#8B5F3356",
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(source) = item_bonus_source {
        apply_opendesign_styles(stylesheet, &mut item_bonus_text, source);
    }
    bonus.children.push(item_bonus_text);
    item_copy.children.push(bonus);

    item_main.children.push(asset_stack);
    item_main.children.push(item_copy);

    let mut purchase = bui_node(&format!("purchase_node_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut purchase, OpenDesignPreset::Purchase);
    let purchase_source = article
        .descendants()
        .find(|node| has_class(*node, "purchase-node"));
    if let Some(source) = purchase_source {
        apply_opendesign_styles(stylesheet, &mut purchase, source);
    }

    let mut price_tag = bui_node(&format!("price_tag_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut price_tag, OpenDesignPreset::PriceTag);
    let price_tag_source = article
        .descendants()
        .find(|node| has_class(*node, "price-tag"));
    if let Some(source) = price_tag_source {
        apply_opendesign_styles(stylesheet, &mut price_tag, source);
    }

    let mut coin = bui_node(&format!("price_coin_{id}"), BuiNodeType::Node);
    apply_opendesign_preset(&mut coin, OpenDesignPreset::PriceCoin);
    if let Some(source) = article
        .descendants()
        .find(|node| has_class(*node, "price-coin"))
    {
        apply_opendesign_styles(stylesheet, &mut coin, source);
    }
    price_tag.children.push(coin);
    let mut price_text = text_node(
        &format!("price_{id}_text"),
        price,
        30.0,
        "#D89A1F",
        Some("STHeiti Medium.ttc"),
    );
    if let Some(source) = price_tag_source {
        apply_opendesign_styles(stylesheet, &mut price_text, source);
    }
    price_tag.children.push(price_text);

    let mut buy = bui_node(&format!("buy_btn_{id}"), BuiNodeType::Button);
    buy.custom_tags.push("Sound_Click".to_string());
    buy.custom_tags
        .push(format!("Action_Buy_{}", pascal_case(&id)));
    buy.actions.push(BuiActionBinding {
        event: "press".to_string(),
        emit: format!("buy_item_{id}"),
    });
    apply_opendesign_preset(&mut buy, OpenDesignPreset::BuyButton);
    let buy_source = article.descendants().find(|node| has_class(*node, "buy-btn"));
    if let Some(source) = buy_source {
        apply_opendesign_styles(stylesheet, &mut buy, source);
    }
    let mut buy_text = text_node(
        &format!("buy_btn_{id}_text"),
        "购买",
        20.0,
        "#FFFFFF",
        Some("STHeiti Medium.ttc"),
    );
    if let Some(source) = buy_source {
        apply_opendesign_styles(stylesheet, &mut buy_text, source);
    }
    buy.children.push(buy_text);

    purchase.children.push(price_tag);
    purchase.children.push(buy);
    card.children.push(item_main);
    card.children.push(purchase);

    Ok(card)
}

fn bui_node(id: &str, node_type: BuiNodeType) -> BuiNode {
    BuiNode {
        id: id.to_string(),
        node_type,
        custom_tags: Vec::new(),
        actions: Vec::new(),
        bindings: Vec::new(),
        tab_group_name: None,
        tab_binding_source: None,
        tab_value: None,
        progress_binding_source: None,
        progress_fill: false,
        list_binding_source: None,
        state_visuals: HashMap::new(),
        styles: BuiStyles::default(),
        visuals: BuiVisuals::default(),
        text_config: None,
        image_config: None,
        children: Vec::new(),
    }
}

fn text_node(
    id: &str,
    content: impl Into<String>,
    font_size: f32,
    font_color: &str,
    font_path: Option<&str>,
) -> BuiNode {
    let mut node = bui_node(id, BuiNodeType::Text);
    node.text_config = Some(BuiTextConfig {
        content: content.into(),
        placeholder: None,
        font_size,
        font_color: font_color.to_string(),
        font_path: font_path.map(str::to_string),
        text_shadow: None,
        linebreak: None,
        visible_width: None,
        allow_newlines: None,
    });
    node
}

fn node_type_to_kind(node_type: &BuiNodeType) -> &'static str {
    match node_type {
        BuiNodeType::Node => "node",
        BuiNodeType::Text => "text",
        BuiNodeType::TextInput => "text_input",
        BuiNodeType::Toggle => "toggle",
        BuiNodeType::Button => "button",
        BuiNodeType::Image => "image",
    }
}

fn kind_to_node_type(kind: &str) -> Result<BuiNodeType, String> {
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

#[derive(Debug, Clone, Copy)]
enum OpenDesignPreset {
    OverlayRoot,
    Panel,
    PanelHeader,
    TitleBoard,
    CloseButton,
    ShopBody,
    ShopScroll,
    ShopCard,
    ItemMain,
    AssetStack,
    AssetSlot,
    Stars,
    ItemCopy,
    ItemBonus,
    Purchase,
    PriceTag,
    PriceCoin,
    BuyButton,
    FootHint,
}

fn apply_opendesign_preset(node: &mut BuiNode, preset: OpenDesignPreset) {
    match preset {
        OpenDesignPreset::OverlayRoot => {
            node.styles.width = Some("100%".to_string());
            node.styles.height = Some("100%".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.align_items = Some("center".to_string());
            node.visuals.background_color = Some("#3B281862".to_string());
        }
        OpenDesignPreset::Panel => {
            node.styles.width = Some("92%".to_string());
            node.styles.height = Some("90%".to_string());
            node.styles.max_width = Some("720px".to_string());
            node.styles.max_height = Some("860px".to_string());
            node.styles.flex_direction = Some("column".to_string());
            node.styles.padding = Some("0px".to_string());
            node.visuals.background_color = Some("#F8ECD0".to_string());
            node.visuals.border_color = Some("#8B5F33".to_string());
            node.visuals.border_width = Some("4px".to_string());
            node.visuals.border_radius = Some("28px".to_string());
        }
        OpenDesignPreset::PanelHeader => {
            node.styles.flex_direction = Some("row".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.align_items = Some("center".to_string());
            node.styles.padding = Some("28px 64px 18px 64px".to_string());
        }
        OpenDesignPreset::TitleBoard => {
            node.styles.min_width = Some("220px".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.align_items = Some("center".to_string());
            node.styles.padding = Some("14px 32px 16px 32px".to_string());
            node.visuals.background_color = Some("#8B5F33".to_string());
            node.visuals.border_width = Some("3px".to_string());
            node.visuals.border_color = Some("#3B2818D8".to_string());
            node.visuals.border_radius = Some("18px".to_string());
        }
        OpenDesignPreset::CloseButton => {
            node.styles.width = Some("48px".to_string());
            node.styles.height = Some("48px".to_string());
            node.styles.position_type = Some("absolute".to_string());
            node.styles.top = Some("18px".to_string());
            node.styles.right = Some("18px".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.align_items = Some("center".to_string());
            node.visuals.background_color = Some("#CC4D3F".to_string());
            node.visuals.border_width = Some("0px".to_string());
            node.visuals.border_color = Some("transparent".to_string());
            node.visuals.border_radius = Some("48px".to_string());
        }
        OpenDesignPreset::ShopBody => {
            node.styles.flex_direction = Some("column".to_string());
            node.styles.padding = Some("0px 16px 18px 16px".to_string());
            node.styles.flex_grow = Some("1".to_string());
        }
        OpenDesignPreset::ShopScroll => {
            node.styles.flex_direction = Some("column".to_string());
            node.styles.overflow = Some("scroll_y".to_string());
            node.styles.padding = Some("8px 6px 8px 2px".to_string());
            node.styles.row_gap = Some("14px".to_string());
            node.styles.max_height = Some("560px".to_string());
        }
        OpenDesignPreset::ShopCard => {
            node.styles.display = Some("grid".to_string());
            node.styles.grid_template_columns = Some("flex(1) auto".to_string());
            node.styles.align_items = Some("stretch".to_string());
            node.styles.padding = Some("14px".to_string());
            node.visuals.background_color = Some("#F8ECD0".to_string());
            node.visuals.border_width = Some("2px".to_string());
            node.visuals.border_color = Some("#8B5F33E6".to_string());
            node.visuals.border_radius = Some("20px".to_string());
        }
        OpenDesignPreset::ItemMain => {
            node.styles.display = Some("grid".to_string());
            node.styles.grid_template_columns = Some("px(92) flex(1)".to_string());
            node.styles.flex_grow = Some("1".to_string());
        }
        OpenDesignPreset::AssetStack => {
            node.styles.flex_direction = Some("column".to_string());
            node.styles.row_gap = Some("10px".to_string());
            node.styles.align_items = Some("stretch".to_string());
            node.styles.width = Some("92px".to_string());
        }
        OpenDesignPreset::AssetSlot => {
            node.styles.min_height = Some("92px".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.align_items = Some("center".to_string());
            node.styles.padding = Some("10px".to_string());
            node.visuals.background_color = Some("#F8ECD0".to_string());
            node.visuals.border_width = Some("2px".to_string());
            node.visuals.border_color = Some("#8B5F33D2".to_string());
            node.visuals.border_radius = Some("18px".to_string());
        }
        OpenDesignPreset::Stars => {
            node.styles.flex_direction = Some("row".to_string());
            node.styles.column_gap = Some("6px".to_string());
            node.styles.justify_content = Some("space_evenly".to_string());
            node.styles.min_height = Some("24px".to_string());
            node.styles.padding = Some("0px 2px".to_string());
        }
        OpenDesignPreset::ItemCopy => {
            node.styles.flex_direction = Some("column".to_string());
            node.styles.row_gap = Some("6px".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.flex_grow = Some("1".to_string());
            node.styles.min_width = Some("0px".to_string());
        }
        OpenDesignPreset::ItemBonus => {
            node.styles.flex_direction = Some("row".to_string());
            node.styles.align_items = Some("center".to_string());
            node.styles.column_gap = Some("6px".to_string());
            node.styles.padding = Some("6px 10px".to_string());
            node.visuals.background_color = Some("#D89A1F2E".to_string());
            node.visuals.border_radius = Some("48px".to_string());
        }
        OpenDesignPreset::Purchase => {
            node.styles.flex_direction = Some("column".to_string());
            node.styles.justify_content = Some("space_between".to_string());
            node.styles.align_items = Some("flex_end".to_string());
            node.styles.min_width = Some("120px".to_string());
            node.styles.width = Some("140px".to_string());
            node.styles.row_gap = Some("12px".to_string());
        }
        OpenDesignPreset::PriceTag => {
            node.styles.flex_direction = Some("row".to_string());
            node.styles.align_items = Some("center".to_string());
            node.styles.column_gap = Some("6px".to_string());
            node.styles.padding = Some("8px 12px".to_string());
            node.visuals.background_color = Some("#FFFFFF2E".to_string());
            node.visuals.border_width = Some("2px".to_string());
            node.visuals.border_color = Some("#8B5F33DE".to_string());
            node.visuals.border_radius = Some("14px".to_string());
        }
        OpenDesignPreset::PriceCoin => {
            node.styles.width = Some("16px".to_string());
            node.styles.height = Some("16px".to_string());
            node.visuals.background_color = Some("#D89A1F".to_string());
            node.visuals.border_width = Some("1px".to_string());
            node.visuals.border_color = Some("#3B28189C".to_string());
            node.visuals.border_radius = Some("16px".to_string());
        }
        OpenDesignPreset::BuyButton => {
            node.styles.min_width = Some("112px".to_string());
            node.styles.min_height = Some("48px".to_string());
            node.styles.justify_content = Some("center".to_string());
            node.styles.align_items = Some("center".to_string());
            node.styles.padding = Some("0px 20px".to_string());
            node.visuals.background_color = Some("#3FB45A".to_string());
            node.visuals.border_width = Some("0px".to_string());
            node.visuals.border_color = Some("transparent".to_string());
            node.visuals.border_radius = Some("18px".to_string());
        }
        OpenDesignPreset::FootHint => {
            node.styles.justify_content = Some("center".to_string());
            node.styles.padding = Some("6px 18px 18px 18px".to_string());
        }
    }
}

#[derive(Debug, Default)]
struct OpenDesignStylesheet {
    variables: HashMap<String, String>,
    rules: Vec<OpenDesignCssRule>,
}

#[derive(Debug)]
struct OpenDesignCssRule {
    selector: OpenDesignSelector,
    declarations: Vec<(String, String)>,
    order: usize,
}

#[derive(Debug, Clone)]
struct OpenDesignSelector {
    parts: Vec<OpenDesignSelectorPart>,
    weight: i32,
}

#[derive(Debug, Clone)]
struct OpenDesignSelectorPart {
    combinator: OpenDesignCombinator,
    compound: OpenDesignSelectorCompound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenDesignCombinator {
    SelfNode,
    Descendant,
    DirectChild,
}

#[derive(Debug, Default, Clone)]
struct OpenDesignSelectorCompound {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    states: Vec<String>,
    pseudo_element: Option<String>,
}

impl OpenDesignStylesheet {
    fn parse(html: &str) -> Self {
        let mut stylesheet = Self::default();
        let mut order = 0;

        for block in style_blocks(html) {
            for (selector_group, declarations) in css_rules(block) {
                for (name, value) in &declarations {
                    if selector_group.trim() == ":root" && name.starts_with("--") {
                        stylesheet
                            .variables
                            .insert(name.to_string(), value.trim().to_string());
                    }
                }

                for selector in selector_group.split(',') {
                    let selector = selector.trim();
                    if selector.is_empty()
                        || selector.starts_with('@')
                    {
                        continue;
                    }
                    if selector.contains("::before") || selector.contains("::after") {
                        if let Some(selector) = OpenDesignSelector::parse_pseudo(selector) {
                            stylesheet.rules.push(OpenDesignCssRule {
                                selector,
                                declarations: declarations.clone(),
                                order,
                            });
                            order += 1;
                        }
                        continue;
                    }
                    if selector.contains("::") {
                        continue;
                    }
                    if selector.contains('[') {
                        continue;
                    }
                    if let Some(selector) = OpenDesignSelector::parse(selector) {
                        stylesheet.rules.push(OpenDesignCssRule {
                            selector,
                            declarations: declarations.clone(),
                            order,
                        });
                        order += 1;
                    }
                }
            }
        }

        stylesheet
    }

    fn matching_declarations(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> Vec<&(String, String)> {
        let mut rules = self
            .rules
            .iter()
            .filter(|rule| rule.selector.state_name().is_none() && rule.selector.pseudo_element_name().is_none() && rule.selector.matches(dom_node))
            .collect::<Vec<_>>();
        rules.sort_by_key(|rule| (rule.selector.weight, rule.order));
        rules
            .into_iter()
            .flat_map(|rule| rule.declarations.iter())
            .collect()
    }

    fn matching_pseudo_declarations(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
        pseudo_element: &str,
    ) -> Vec<&(String, String)> {
        let mut rules = self
            .rules
            .iter()
            .filter(|rule| {
                rule.selector.state_name().is_none()
                    && rule.selector.pseudo_element_name() == Some(pseudo_element)
                    && rule.selector.matches(dom_node)
            })
            .collect::<Vec<_>>();
        rules.sort_by_key(|rule| (rule.selector.weight, rule.order));
        rules
            .into_iter()
            .flat_map(|rule| rule.declarations.iter())
            .collect()
    }

    fn matching_state_declarations(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> Vec<(&str, &(String, String))> {
        let mut rules = self
            .rules
            .iter()
            .filter_map(|rule| {
                rule.selector
                    .state_name()
                    .filter(|_| rule.selector.matches(dom_node))
                    .map(|state| (state, rule))
            })
            .collect::<Vec<_>>();
        rules.sort_by_key(|(_, rule)| (rule.selector.weight, rule.order));
        rules
            .into_iter()
            .flat_map(|(state, rule)| {
                rule.declarations
                    .iter()
                    .map(move |declaration| (state, declaration))
            })
            .collect()
    }

    fn resolve_value(&self, value: &str) -> String {
        let mut resolved = value.trim().to_string();
        for _ in 0..4 {
            let Some(start) = resolved.find("var(") else {
                break;
            };
            let Some(end) = resolved[start..].find(')') else {
                break;
            };
            let end = start + end;
            let variable_name = resolved[start + 4..end].trim();
            let replacement = self
                .variables
                .get(variable_name)
                .cloned()
                .unwrap_or_default();
            resolved.replace_range(start..=end, &replacement);
        }
        resolved.trim().to_string()
    }
}

impl OpenDesignSelector {
    fn parse(selector: &str) -> Option<Self> {
        let mut parts = Vec::new();
        let mut token = String::new();
        let mut combinator = OpenDesignCombinator::SelfNode;
        let mut chars = selector.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '>' => {
                    push_selector_part(&mut parts, &mut token, combinator);
                    combinator = OpenDesignCombinator::DirectChild;
                    while chars.peek().is_some_and(|c| c.is_whitespace()) {
                        chars.next();
                    }
                }
                character if character.is_whitespace() => {
                    push_selector_part(&mut parts, &mut token, combinator);
                    if !parts.is_empty() {
                        combinator = OpenDesignCombinator::Descendant;
                    }
                    while chars.peek().is_some_and(|c| c.is_whitespace()) {
                        chars.next();
                    }
                }
                _ => token.push(character),
            }
        }
        push_selector_part(&mut parts, &mut token, combinator);

        if parts.is_empty() {
            return None;
        }

        let weight = parts
            .iter()
            .map(|part| part.compound.weight())
            .sum::<i32>();

        Some(Self { parts, weight })
    }

    fn matches(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
        self.matches_from(self.parts.len() - 1, dom_node)
    }

    fn state_name(&self) -> Option<&'static str> {
        self.parts
            .last()?
            .compound
            .states
            .iter()
            .rev()
            .find_map(|state| match state.as_str() {
                "hover" => Some("hovered"),
                "active" => Some("pressed"),
                "focus" | "focus-visible" => Some("focused"),
                _ => None,
            })
    }

    fn pseudo_element_name(&self) -> Option<&str> {
        self.parts.last()?.compound.pseudo_element.as_deref()
    }

    fn matches_from(&self, part_index: usize, dom_node: roxmltree::Node<'_, '_>) -> bool {
        let part = &self.parts[part_index];
        if !part.compound.matches(dom_node) {
            return false;
        }
        if part_index == 0 {
            return true;
        }

        match part.combinator {
            OpenDesignCombinator::DirectChild => dom_node
                .parent()
                .filter(|parent| parent.is_element())
                .is_some_and(|parent| self.matches_from(part_index - 1, parent)),
            OpenDesignCombinator::Descendant => dom_node
                .ancestors()
                .skip(1)
                .filter(|ancestor| ancestor.is_element())
                .any(|ancestor| self.matches_from(part_index - 1, ancestor)),
            OpenDesignCombinator::SelfNode => false,
        }
    }

    fn parse_pseudo(selector: &str) -> Option<Self> {
        let pseudo_element = if selector.contains("::before") {
            "before"
        } else if selector.contains("::after") {
            "after"
        } else {
            return None;
        };

        let cleaned = selector.replace("::before", "").replace("::after", "");
        let mut parsed = Self::parse(cleaned.trim())?;
        parsed.parts.last_mut()?.compound.pseudo_element = Some(pseudo_element.to_string());
        Some(parsed)
    }
}

impl OpenDesignSelectorCompound {
    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        if raw == "*" {
            return Some(Self::default());
        }

        let mut compound = Self::default();
        let mut chars = raw.chars().peekable();
        let mut tag = String::new();

        while let Some(character) = chars.peek().copied() {
            match character {
                '.' | '#' | ':' => break,
                _ => {
                    tag.push(read_selector_char(&mut chars));
                }
            }
        }
        if !tag.is_empty() {
            compound.tag = Some(unescape_css_ident(&tag).to_ascii_lowercase());
        }

        while let Some(prefix) = chars.next() {
            let mut value = String::new();
            while let Some(character) = chars.peek().copied() {
                if matches!(character, '.' | '#' | ':') {
                    break;
                }
                value.push(read_selector_char(&mut chars));
            }
            if value.is_empty() {
                continue;
            }
            let value = unescape_css_ident(&value);
            match prefix {
                '.' => compound.classes.push(value),
                '#' => compound.id = Some(value),
                ':' => compound.states.push(value),
                _ => {}
            }
        }

        Some(compound)
    }

    fn weight(&self) -> i32 {
        let mut weight = 0;
        if self.tag.is_some() {
            weight += 1;
        }
        weight += self.classes.len() as i32 * 10;
        weight += self.states.len() as i32 * 10;
        if self.id.is_some() {
            weight += 100;
        }
        weight
    }

    fn matches(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
        if !dom_node.is_element() {
            return false;
        }
        if let Some(tag) = &self.tag {
            if dom_node.tag_name().name().to_ascii_lowercase() != *tag {
                return false;
            }
        }
        if let Some(id) = &self.id {
            if dom_node.attribute("id") != Some(id.as_str()) {
                return false;
            }
        }
        if !self
            .classes
            .iter()
            .all(|class_name| has_class(dom_node, class_name))
        {
            return false;
        }

        true
    }
}

fn apply_opendesign_styles(
    stylesheet: &OpenDesignStylesheet,
    bui_node: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
) {
    for (name, value) in stylesheet.matching_declarations(dom_node) {
        let value = stylesheet.resolve_value(value);
        apply_opendesign_declaration(bui_node, name, &value);
    }

    for (state, (name, value)) in stylesheet.matching_state_declarations(dom_node) {
        let value = stylesheet.resolve_value(value);
        apply_opendesign_state_declaration(bui_node, state, name, &value);
    }

    if let Some(inline_style) = dom_node.attribute("style") {
        for (name, value) in css_declarations(inline_style) {
            let value = stylesheet.resolve_value(&value);
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }
}

fn apply_opendesign_state_declaration(
    bui_node: &mut BuiNode,
    state: &str,
    name: &str,
    value: &str,
) {
    let value = normalize_css_value(value);
    if value.is_empty() || value.contains("!important") {
        return;
    }
    if matches!(bui_node.node_type, BuiNodeType::Text) && name != "color" {
        return;
    }

    let mut needs_normal_scale_reset = false;
    let became_empty;
    {
        let state_visual = bui_node
            .state_visuals
            .entry(state.to_string())
            .or_insert_with(|| BuiStateVisual {
                styles: BuiStyles::default(),
                visuals: BuiVisuals::default(),
                text_color: None,
            });

        match name {
            "background" | "background-color" => {
                if let Some(color) = css_color(&value) {
                    state_visual.visuals.background_color = Some(color);
                }
            }
            "border" => {
                if let Some(color) = css_color(&value) {
                    state_visual.visuals.border_color = Some(color);
                }
                if let Some(width) = css_first_size(&value) {
                    state_visual.visuals.border_width = Some(width);
                }
            }
            "border-color" => {
                if let Some(color) = css_color(&value) {
                    state_visual.visuals.border_color = Some(color);
                }
            }
            "color" => {
                if let Some(color) = css_color(&value) {
                    state_visual.text_color = Some(color);
                }
            }
            "transform" => {
                if let Some(scale) = css_transform_scale(&value) {
                    state_visual.styles.ui_scale = Some(scale);
                    needs_normal_scale_reset = true;
                }
            }
            _ => {}
        }

        became_empty = state_visual.is_empty();
    }

    if needs_normal_scale_reset {
        ensure_opendesign_normal_state(bui_node).styles.ui_scale = Some("1 1".to_string());
    }
    if became_empty {
        bui_node.state_visuals.remove(state);
    }
}

fn ensure_opendesign_normal_state(bui_node: &mut BuiNode) -> &mut BuiStateVisual {
    bui_node
        .state_visuals
        .entry("normal".to_string())
        .or_insert_with(|| BuiStateVisual {
            styles: BuiStyles::default(),
            visuals: BuiVisuals::default(),
            text_color: None,
        })
}

fn apply_opendesign_declaration(bui_node: &mut BuiNode, name: &str, value: &str) {
    let value = normalize_css_value(value);
    if value.is_empty() || value.contains("!important") {
        return;
    }
    if matches!(bui_node.node_type, BuiNodeType::Text) && !matches!(name, "color" | "font-size") {
        return;
    }

    match name {
        "display" => {
            if let Some(display) = css_display(&value) {
                bui_node.styles.display = Some(display.to_string());
            }
        }
        "position" => {
            if matches!(value.as_str(), "absolute" | "relative") {
                bui_node.styles.position_type = Some(value);
            } else if value == "fixed" {
                bui_node.styles.position_type = Some("absolute".to_string());
                bui_node.styles.fixed_node = Some(true);
            }
        }
        "width" => set_simple_css_val(&mut bui_node.styles.width, &value),
        "height" => set_simple_css_val(&mut bui_node.styles.height, &value),
        "min-width" => set_simple_css_val(&mut bui_node.styles.min_width, &value),
        "min-height" => set_simple_css_val(&mut bui_node.styles.min_height, &value),
        "max-width" => set_simple_css_val(&mut bui_node.styles.max_width, &value),
        "max-height" => set_simple_css_val(&mut bui_node.styles.max_height, &value),
        "left" => set_simple_css_val(&mut bui_node.styles.left, &value),
        "right" => set_simple_css_val(&mut bui_node.styles.right, &value),
        "top" => set_simple_css_val(&mut bui_node.styles.top, &value),
        "bottom" => set_simple_css_val(&mut bui_node.styles.bottom, &value),
        "margin" => set_css_rect(&mut bui_node.styles.margin, &value),
        "padding" => set_css_rect(&mut bui_node.styles.padding, &value),
        "padding-inline" => {
            set_simple_css_val(&mut bui_node.styles.padding_left, &value);
            set_simple_css_val(&mut bui_node.styles.padding_right, &value);
        }
        "padding-block" => {
            set_simple_css_val(&mut bui_node.styles.padding_top, &value);
            set_simple_css_val(&mut bui_node.styles.padding_bottom, &value);
        }
        "gap" => {
            if is_simple_css_size(&value) {
                bui_node.styles.row_gap = Some(value.clone());
                bui_node.styles.column_gap = Some(value);
            }
        }
        "row-gap" => set_simple_css_val(&mut bui_node.styles.row_gap, &value),
        "column-gap" => set_simple_css_val(&mut bui_node.styles.column_gap, &value),
        "flex-direction" => bui_node.styles.flex_direction = Some(value),
        "flex-wrap" => bui_node.styles.flex_wrap = Some(value),
        "flex-grow" => bui_node.styles.flex_grow = Some(value),
        "flex-shrink" => bui_node.styles.flex_shrink = Some(value),
        "flex-basis" => set_simple_css_val(&mut bui_node.styles.flex_basis, &value),
        "align-items" => bui_node.styles.align_items = Some(value),
        "align-self" => bui_node.styles.align_self = Some(value),
        "align-content" => bui_node.styles.align_content = Some(value),
        "justify-content" => bui_node.styles.justify_content = Some(value),
        "justify-items" => bui_node.styles.justify_items = Some(value),
        "justify-self" => bui_node.styles.justify_self = Some(value),
        "place-items" => {
            if value == "center" {
                bui_node.styles.align_items = Some("center".to_string());
                bui_node.styles.justify_items = Some("center".to_string());
                bui_node.styles.justify_content = Some("center".to_string());
            }
        }
        "overflow" => {
            if let Some(overflow) = css_overflow(&value) {
                bui_node.styles.overflow = Some(overflow.to_string());
            }
        }
        "overflow-x" => {
            if value == "auto" || value == "scroll" {
                bui_node.styles.overflow = Some("scroll_x".to_string());
            }
        }
        "overflow-y" => {
            if value == "auto" || value == "scroll" {
                bui_node.styles.overflow = Some("scroll_y".to_string());
            }
        }
        "grid-template-columns" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.styles.grid_template_columns = Some(tracks);
            }
        }
        "grid-template-rows" => {
            if let Some(tracks) = css_grid_tracks(&value) {
                bui_node.styles.grid_template_rows = Some(tracks);
            }
        }
        "border-radius" => {
            if let Some(radius) = css_first_size(&value) {
                bui_node.visuals.border_radius = Some(radius);
            }
        }
        "border-width" => set_css_rect(&mut bui_node.visuals.border_width, &value),
        "border" => apply_css_border(bui_node, &value),
        "border-color" => {
            if let Some(color) = css_color(&value) {
                bui_node.visuals.border_color = Some(color);
            }
        }
        "background" | "background-color" => {
            if let Some(color) = css_color(&value) {
                bui_node.visuals.background_color = Some(color);
            }
        }
        "color" => {
            if let Some(color) = css_color(&value) {
                if let Some(text_config) = &mut bui_node.text_config {
                    text_config.font_color = color;
                }
            }
        }
        "font-size" => {
            if let Some(font_size) = css_font_size(&value) {
                if let Some(text_config) = &mut bui_node.text_config {
                    text_config.font_size = font_size;
                }
            }
        }
        "font-family" => {
            if let Some(text_config) = &mut bui_node.text_config {
                text_config.font_path = Some(css_font_family_to_path(&value));
            }
        }
        "opacity" => {
            if let Some(opacity) = value.parse::<f32>().ok() {
                if let Some(color) = &mut bui_node.visuals.background_color {
                    if let Some(hex) = append_hex_alpha(color, opacity * 100.0) {
                        *color = hex;
                    }
                }
                if let Some(color) = &mut bui_node.visuals.border_color {
                    if let Some(hex) = append_hex_alpha(color, opacity * 100.0) {
                        *color = hex;
                    }
                }
                if let Some(text_config) = &mut bui_node.text_config {
                    if let Some(hex) = append_hex_alpha(&text_config.font_color, opacity * 100.0) {
                        text_config.font_color = hex;
                    }
                }
            }
        }
        "cursor" | "pointer-events" | "mix-blend-mode" | "filter" | "transition" | "clip-path" | "mask-image" | "content" | "isolation" | "z-index" | "-webkit-tap-highlight-color" | "text-shadow" | "font-weight" | "letter-spacing" | "aspect-ratio" | "text-align" | "box-shadow" => {}
        _ => {}
    }
}

fn style_blocks(html: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        rest = &rest[start..];
        let Some(tag_end) = rest.find('>') else {
            break;
        };
        rest = &rest[tag_end + 1..];
        let Some(end) = rest.find("</style>") else {
            break;
        };
        blocks.push(&rest[..end]);
        rest = &rest[end + "</style>".len()..];
    }
    blocks
}

fn css_rules(css: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut rules = Vec::new();
    let bytes = css.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let selector_start = index;
        while index < bytes.len() && bytes[index] != b'{' {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let selector = css[selector_start..index].trim();
        index += 1;

        let body_start = index;
        let mut depth = 1;
        while index < bytes.len() && depth > 0 {
            match bytes[index] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        if depth != 0 {
            break;
        }
        let body = &css[body_start..index - 1];
        if selector.starts_with("@media") {
            if media_query_matches(selector) {
                rules.extend(css_rules(body));
            }
            continue;
        }
        if selector.starts_with('@') {
            continue;
        }
        let declarations = css_declarations(body);
        if !selector.is_empty() && !declarations.is_empty() {
            rules.push((selector.to_string(), declarations));
        }
    }

    rules
}

fn media_query_matches(selector: &str) -> bool {
    let query = selector.trim_start_matches("@media").trim();
    let mut matched_any_width_condition = false;

    for condition in query.split("and") {
        let condition = condition
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();

        if let Some(value) = condition.strip_prefix("min-width:") {
            matched_any_width_condition = true;
            let Some(width) = css_first_size(value).and_then(|size| css_size_to_px(&size)) else {
                return false;
            };
            if OPENDESIGN_DEFAULT_VIEWPORT_WIDTH < width {
                return false;
            }
        } else if let Some(value) = condition.strip_prefix("max-width:") {
            matched_any_width_condition = true;
            let Some(width) = css_first_size(value).and_then(|size| css_size_to_px(&size)) else {
                return false;
            };
            if OPENDESIGN_DEFAULT_VIEWPORT_WIDTH > width {
                return false;
            }
        }
    }

    matched_any_width_condition
}

fn css_declarations(body: &str) -> Vec<(String, String)> {
    body.split(';')
        .filter_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                None
            } else {
                Some((name.to_string(), value.to_string()))
            }
        })
        .collect()
}

fn push_selector_part(
    parts: &mut Vec<OpenDesignSelectorPart>,
    token: &mut String,
    combinator: OpenDesignCombinator,
) {
    if let Some(compound) = OpenDesignSelectorCompound::parse(token) {
        parts.push(OpenDesignSelectorPart {
            combinator: if parts.is_empty() {
                OpenDesignCombinator::SelfNode
            } else {
                combinator
            },
            compound,
        });
    }
    token.clear();
}

fn read_selector_char(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> char {
    let character = chars.next().unwrap_or_default();
    if character == '\\' {
        chars.next().unwrap_or(character)
    } else {
        character
    }
}

fn unescape_css_ident(value: &str) -> String {
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    output
}

fn normalize_css_value(value: &str) -> String {
    value
        .trim()
        .trim_end_matches("!important")
        .trim()
        .trim_matches('"')
        .replace("  ", " ")
        .replace("solid ", "")
}

fn set_simple_css_val(target: &mut Option<String>, value: &str) {
    if let Some(size) = css_eval_length_function(value) {
        *target = Some(size);
    } else if is_simple_css_size(value) {
        *target = Some(value.to_string());
    } else if let Some(size) = css_first_size(value) {
        *target = Some(size);
    }
}

fn set_css_rect(target: &mut Option<String>, value: &str) {
    let normalized = value
        .split_whitespace()
        .filter(|part| is_simple_css_size(part))
        .collect::<Vec<_>>()
        .join(" ");
    if !normalized.is_empty() {
        *target = Some(normalized);
    }
}

fn is_simple_css_size(value: &str) -> bool {
    let value = value.trim();
    if value == "auto" || value == "0" {
        return true;
    }
    value
        .strip_suffix("px")
        .or_else(|| value.strip_suffix('%'))
        .or_else(|| value.strip_suffix("vw"))
        .or_else(|| value.strip_suffix("vh"))
        .is_some_and(|number| number.parse::<f32>().is_ok())
}

fn css_first_size(value: &str) -> Option<String> {
    if let Some(size) = css_eval_length_function(value) {
        return Some(size);
    }
    value
        .split(|character: char| character.is_whitespace() || matches!(character, ',' | '(' | ')'))
        .find(|part| is_simple_css_size(part))
        .and_then(css_length_to_bui_val)
}

fn css_size_to_px(value: &str) -> Option<f32> {
    let value = value.trim();
    if let Some(px) = value.strip_suffix("px") {
        px.parse::<f32>().ok()
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|percent| OPENDESIGN_DEFAULT_VIEWPORT_WIDTH * percent / 100.0)
    } else if let Some(vw) = value.strip_suffix("vw") {
        vw.parse::<f32>()
            .ok()
            .map(|vw| OPENDESIGN_DEFAULT_VIEWPORT_WIDTH * vw / 100.0)
    } else if let Some(vh) = value.strip_suffix("vh") {
        vh.parse::<f32>()
            .ok()
            .map(|vh| OPENDESIGN_DEFAULT_VIEWPORT_HEIGHT * vh / 100.0)
    } else {
        value.parse::<f32>().ok()
    }
}

fn css_length_to_bui_val(value: &str) -> Option<String> {
    let value = value.trim();
    if value == "auto" || value == "0" || value.ends_with("px") || value.ends_with('%') {
        return Some(value.to_string());
    }
    if value.ends_with("vw") || value.ends_with("vh") {
        return css_size_to_px(value).map(|px| format_css_px(px));
    }
    None
}

fn css_eval_length_function(value: &str) -> Option<String> {
    let value = value.trim();
    let (name, args) = css_function_call(value)?;
    let args = split_css_function_args(args);
    match name {
        "min" => args
            .iter()
            .filter_map(|arg| css_size_to_px(arg))
            .reduce(f32::min)
            .map(format_css_px),
        "max" => args
            .iter()
            .filter_map(|arg| css_size_to_px(arg))
            .reduce(f32::max)
            .map(format_css_px),
        "clamp" if args.len() == 3 => {
            let min = css_size_to_px(args[0])?;
            let preferred = css_size_to_px(args[1])?;
            let max = css_size_to_px(args[2])?;
            Some(format_css_px(preferred.clamp(min, max)))
        }
        _ => None,
    }
}

fn css_function_call(value: &str) -> Option<(&str, &str)> {
    let (name, rest) = value.split_once('(')?;
    let args = rest.strip_suffix(')')?;
    Some((name.trim(), args.trim()))
}

fn split_css_function_args(value: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(value[start..].trim());
    args.into_iter().filter(|arg| !arg.is_empty()).collect()
}

fn format_css_px(value: f32) -> String {
    if (value.fract()).abs() < f32::EPSILON {
        format!("{}px", value as i32)
    } else {
        let number = format!("{value:.2}");
        let number = number.trim_end_matches('0').trim_end_matches('.');
        format!("{number}px")
    }
}

fn css_display(value: &str) -> Option<&'static str> {
    match value {
        "flex" | "inline-flex" => Some("flex"),
        "grid" | "inline-grid" => Some("grid"),
        "none" => Some("none"),
        _ => None,
    }
}

fn css_overflow(value: &str) -> Option<&'static str> {
    match value {
        "visible" => Some("visible"),
        "hidden" | "clip" => Some("clip"),
        "auto" | "scroll" => Some("scroll_y"),
        _ => None,
    }
}

fn css_grid_tracks(value: &str) -> Option<String> {
    let value = value.trim();
    match value {
        "minmax(0, 1fr) auto" => Some("flex(1) auto".to_string()),
        "minmax(0, 1fr) 140px" => Some("flex(1) px(140)".to_string()),
        "92px minmax(0, 1fr)" => Some("px(92) flex(1)".to_string()),
        "104px minmax(0, 1fr)" => Some("px(104) flex(1)".to_string()),
        "84px minmax(0, 1fr)" => Some("px(84) flex(1)".to_string()),
        "repeat(4, minmax(0, 1fr))" => Some("flex(4, 1)".to_string()),
        "1fr" => Some("flex(1)".to_string()),
        _ if !value.contains('(') => Some(
            value
                .split_whitespace()
                .map(|part| {
                    if let Some(px) = part.strip_suffix("px") {
                        format!("px({px})")
                    } else if part == "1fr" {
                        "flex(1)".to_string()
                    } else {
                        part.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
        ),
        _ => None,
    }
}

fn css_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(color) = css_color_mix_with_transparency(value) {
        return Some(color);
    }
    if let Some(color) = oklch_to_hex(value) {
        return Some(color);
    }
    if let Some(color) = css_gradient_first_color(value) {
        return Some(color);
    }
    if value == "transparent" {
        return Some("transparent".to_string());
    }
    if is_hex_color(value) {
        return Some(value.to_string());
    }
    for token in value.split(|character: char| {
        character.is_whitespace() || matches!(character, ',' | '(' | ')')
    }) {
        if let Some(color) = oklch_to_hex(token) {
            return Some(color);
        }
        if is_hex_color(token) {
            return Some(token.to_string());
        }
    }
    css_named_color(value).map(ToString::to_string)
}

fn is_hex_color(value: &str) -> bool {
    let value = value.trim();
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn css_color_mix_with_transparency(value: &str) -> Option<String> {
    if !value.trim_start().starts_with("color-mix(") || !value.contains("transparent") {
        return None;
    }

    let tokens = css_function_tokens(value);
    let transparent_index = tokens.iter().position(|token| *token == "transparent")?;
    let transparent_percent = tokens
        .get(transparent_index + 1)
        .and_then(|token| token.strip_suffix('%'))
        .and_then(|percent| percent.parse::<f32>().ok())
        .unwrap_or(50.0)
        .clamp(0.0, 100.0);

    let base_color = tokens
        .iter()
        .take(transparent_index)
        .find_map(|token| {
            if is_hex_color(token) {
                Some((*token).to_string())
            } else {
                css_named_color(token).map(ToString::to_string)
            }
        })
        .or_else(|| {
            tokens.iter().skip(transparent_index + 1).find_map(|token| {
                if is_hex_color(token) {
                    Some((*token).to_string())
                } else {
                    css_named_color(token).map(ToString::to_string)
                }
            })
        })?;

    append_hex_alpha(&base_color, 100.0 - transparent_percent)
}

fn css_function_tokens(value: &str) -> Vec<&str> {
    value
        .split(|character: char| character.is_whitespace() || matches!(character, ',' | '(' | ')'))
        .filter(|token| !token.is_empty() && *token != "in" && *token != "oklab")
        .collect()
}

fn css_named_color(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "black" => Some("#000000"),
        "white" => Some("#FFFFFF"),
        "red" => Some("#FF0000"),
        _ => None,
    }
}

fn oklch_to_hex(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.starts_with("oklch(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.strip_prefix("oklch(")?.strip_suffix(")")?;

    let parts: Vec<&str> = inner
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();

    if parts.len() < 3 {
        return None;
    }

    let l_raw = parts[0];
    let l = l_raw.strip_suffix('%')
        .and_then(|s| s.parse::<f32>().ok())
        .map(|v| v / 100.0)
        .or_else(|| l_raw.parse::<f32>().ok())?;
    let c: f32 = parts[1].parse::<f32>().ok()?;
    let h: f32 = parts[2].parse::<f32>().ok()?;

    let alpha = if parts.len() >= 4 && parts[3].starts_with('/') {
        parts[3].strip_prefix('/')?.parse::<f32>().ok()
    } else if parts.len() >= 5 {
        parts[4].parse::<f32>().ok()
    } else {
        Some(1.0)
    };

    let alpha = alpha?;

    let h_rad = h * std::f32::consts::PI / 180.0;
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l_c = l_ * l_ * l_;
    let m_c = m_ * m_ * m_;
    let s_c = s_ * s_ * s_;

    let x = 1.2268798737 * l_c - 0.5556238332 * m_c + 0.2811894837 * s_c;
    let y = -0.0405757626 * l_c + 1.1971573648 * m_c - 0.1560437476 * s_c;
    let z = -0.0753452638 * l_c + 0.2413318055 * m_c + 1.8340138286 * s_c;

    let r_lin = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g_lin = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let b_lin = 0.0556432 * x - 0.2040259 * y + 1.0572252 * z;

    let r = srgb_gamma(r_lin);
    let g = srgb_gamma(g_lin);
    let b = srgb_gamma(b_lin);

    let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;

    if a == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }
}

fn srgb_gamma(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn css_gradient_first_color(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.starts_with("radial-gradient(") && !value.starts_with("linear-gradient(") && !value.starts_with("conic-gradient(") {
        return None;
    }

    let inner = if value.starts_with("radial-gradient(") {
        value.strip_prefix("radial-gradient(")?.strip_suffix(")")
    } else if value.starts_with("linear-gradient(") {
        value.strip_prefix("linear-gradient(")?.strip_suffix(")")
    } else {
        value.strip_prefix("conic-gradient(")?.strip_suffix(")")
    };
    let Some(inner) = inner else {
        return None;
    };

    let mut depth = 0;
    let mut token_start = 0;
    let mut tokens: Vec<&str> = Vec::new();
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
            }
            b',' if depth == 0 => {
                if i > token_start {
                    tokens.push(inner[token_start..i].trim());
                }
                token_start = i + 1;
            }
            _ => {}
        }
    }
    if token_start < inner.len() {
        tokens.push(inner[token_start..].trim());
    }

    for token in &tokens {
        let stripped = token.trim();
        if stripped.starts_with("oklch(") || stripped.starts_with("#") || stripped.starts_with("rgb(") || stripped.starts_with("rgba(") {
            return css_color(stripped);
        }
        if let Some(hex) = css_named_color(stripped) {
            return Some(hex.to_string());
        }
        if stripped.contains("oklch(") || stripped.contains("#") || stripped.contains("rgb(") {
            for sub in stripped.split(|c: char| c.is_whitespace()) {
                if let Some(color) = css_color(sub) {
                    return Some(color);
                }
            }
        }
    }

    None
}

fn append_hex_alpha(color: &str, alpha_percent: f32) -> Option<String> {
    let hex = color.trim().strip_prefix('#')?;
    let rgb = match hex.len() {
        3 | 4 => hex
            .chars()
            .take(3)
            .flat_map(|character| [character, character])
            .collect::<String>(),
        6 | 8 => hex.chars().take(6).collect::<String>(),
        _ => return None,
    };
    let alpha = ((alpha_percent.clamp(0.0, 100.0) / 100.0) * 255.0).round() as u8;
    Some(format!("#{}{alpha:02X}", rgb.to_ascii_uppercase()))
}

fn css_font_size(value: &str) -> Option<f32> {
    value
        .split(|character: char| character.is_whitespace() || matches!(character, ',' | '(' | ')'))
        .filter_map(|part| part.strip_suffix("px"))
        .filter_map(|number| number.parse::<f32>().ok())
        .last()
}

fn css_font_family_to_path(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("palatino") || lower.contains("iowan") || lower.contains("georgia") || lower.contains("serif") {
        "Hiragino Sans GB.ttc".to_string()
    } else if lower.contains("sfmono") || lower.contains("menlo") || lower.contains("monospace") || lower.contains("consolas") {
        "Hiragino Sans GB.ttc".to_string()
    } else {
        "Hiragino Sans GB.ttc".to_string()
    }
}

fn apply_css_border(bui_node: &mut BuiNode, value: &str) {
    if let Some(width) = css_first_size(value) {
        bui_node.visuals.border_width = Some(width);
    }
    if let Some(color) = css_color(value) {
        bui_node.visuals.border_color = Some(color);
    }
}

fn css_transform_scale(value: &str) -> Option<String> {
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

fn has_class(node: roxmltree::Node<'_, '_>, class_name: &str) -> bool {
    node.is_element()
        && node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(|class| class == class_name))
}

fn first_text_by_class(node: roxmltree::Node<'_, '_>, class_name: &str) -> Option<String> {
    node.descendants()
        .find(|candidate| has_class(*candidate, class_name))
        .map(node_text)
        .filter(|text| !text.is_empty())
}

fn node_text(node: roxmltree::Node<'_, '_>) -> String {
    node.children()
        .filter_map(|candidate| candidate.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_id(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    sanitized.trim_matches('_').to_string()
}

fn pascal_case(value: &str) -> String {
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

fn format_price(value: &str) -> String {
    let mut reversed = String::new();
    for (index, character) in value.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(character);
    }
    reversed.chars().rev().collect()
}

fn validate_bui_document(document: &BuiDocument) -> Result<(), String> {
    if document.version != EXPECTED_VERSION {
        return Err(format!(
            "Unsupported BUI version '{}'. This parser expects version {EXPECTED_VERSION}.",
            document.version
        ));
    }

    if document.scene_name.trim().is_empty() {
        return Err("BUI scene_name must not be empty.".to_string());
    }

    if !matches!(document.root.node_type, BuiNodeType::Node) {
        return Err("BUI root must be a Node.".to_string());
    }

    let mut ids = HashSet::new();
    validate_bui_node(&document.root, "root", &mut ids)
}

fn validate_bui_node(node: &BuiNode, path: &str, ids: &mut HashSet<String>) -> Result<(), String> {
    if node.id.trim().is_empty() {
        return Err(format!("{path}: id must not be empty."));
    }

    if !ids.insert(node.id.clone()) {
        return Err(format!("{path}: duplicate id '{}'.", node.id));
    }

    build_node(&node.styles, &node.visuals).map_err(|error| format!("{path}: {error}"))?;
    validate_styles(&node.styles).map_err(|error| format!("{path}: {error}"))?;
    if let Some(value) = &node.styles.visibility {
        parse_visibility(value).map_err(|error| format!("{path}: {error}"))?;
    }
    if let Some(value) = &node.styles.z_index {
        parse_integer(value).map_err(|error| format!("{path}: {error}"))?;
    }
    if let Some(value) = &node.styles.global_z_index {
        parse_integer(value).map_err(|error| format!("{path}: {error}"))?;
    }
    validate_visuals(&node.visuals).map_err(|error| format!("{path}: {error}"))?;
    validate_actions(&node.actions).map_err(|error| format!("{path}: {error}"))?;
    validate_bindings(&node.bindings).map_err(|error| format!("{path}: {error}"))?;
    validate_state_visuals(&node.state_visuals).map_err(|error| format!("{path}: {error}"))?;
    validate_tab_semantics(node).map_err(|error| format!("{path}: {error}"))?;
    validate_progress_semantics(node).map_err(|error| format!("{path}: {error}"))?;
    validate_list_semantics(node).map_err(|error| format!("{path}: {error}"))?;

    match node.node_type {
        BuiNodeType::Node | BuiNodeType::Button | BuiNodeType::Toggle => {
            reject_config(node.text_config.is_some(), path, "text_config")?;
            if !matches!(node.node_type, BuiNodeType::Button) {
                reject_config(node.image_config.is_some(), path, "image_config")?;
            }
            if let Some(image_config) = &node.image_config {
                validate_image_config(image_config).map_err(|error| format!("{path}: {error}"))?;
            }
        }
        BuiNodeType::Text => {
            let text_config = node
                .text_config
                .as_ref()
                .ok_or_else(|| format!("{path}: Text requires text_config."))?;
            validate_text_config(text_config).map_err(|error| format!("{path}: {error}"))?;
            reject_config(
                text_config.placeholder.is_some(),
                path,
                "text_config.placeholder",
            )?;
            reject_config(node.image_config.is_some(), path, "image_config")?;
            reject_children(node, path)?;
        }
        BuiNodeType::TextInput => {
            let text_config = node
                .text_config
                .as_ref()
                .ok_or_else(|| format!("{path}: TextInput requires text_config."))?;
            validate_text_config(text_config).map_err(|error| format!("{path}: {error}"))?;
            reject_config(node.image_config.is_some(), path, "image_config")?;
            reject_children(node, path)?;
        }
        BuiNodeType::Image => {
            let image_config = node
                .image_config
                .as_ref()
                .ok_or_else(|| format!("{path}: Image requires image_config."))?;
            validate_image_config(image_config).map_err(|error| format!("{path}: {error}"))?;
            reject_config(node.text_config.is_some(), path, "text_config")?;
            reject_children(node, path)?;
        }
    }

    for (index, child) in node.children.iter().enumerate() {
        validate_bui_node(child, &format!("{path}.children[{index}]"), ids)?;
    }

    Ok(())
}

fn validate_visuals(visuals: &BuiVisuals) -> Result<(), String> {
    if let Some(color) = &visuals.background_color {
        parse_color(color)?;
    }
    if let Some(color) = &visuals.border_color {
        parse_color(color)?;
    }
    if let Some(shader) = &visuals.material_shader {
        if shader.trim().is_empty() {
            return Err("visuals.material_shader must not be empty.".to_string());
        }
    }

    Ok(())
}

fn validate_text_config(text_config: &BuiTextConfig) -> Result<(), String> {
    if text_config.font_size <= 0.0 {
        return Err("text_config.font_size must be greater than 0.".to_string());
    }

    parse_color(&text_config.font_color)?;

    if let Some(font_path) = &text_config.font_path {
        if font_path.trim().is_empty() {
            return Err("text_config.font_path must not be empty when present.".to_string());
        }
    }

    if let Some(placeholder) = &text_config.placeholder
        && placeholder.trim().is_empty()
    {
        return Err("text_config.placeholder must not be empty when present.".to_string());
    }

    if let Some(text_shadow) = &text_config.text_shadow {
        if let Some(color) = &text_shadow.color {
            parse_color(color)?;
        }
    }
    if let Some(linebreak) = &text_config.linebreak {
        parse_linebreak(linebreak)?;
    }
    if let Some(visible_width) = text_config.visible_width
        && visible_width <= 0.0
    {
        return Err("text_config.visible_width must be greater than 0 when present.".to_string());
    }

    Ok(())
}

fn validate_actions(actions: &[BuiActionBinding]) -> Result<(), String> {
    for action in actions {
        parse_action_trigger(&action.event)?;

        if action.emit.trim().is_empty() {
            return Err("actions.emit must not be empty.".to_string());
        }
    }

    Ok(())
}

fn validate_bindings(bindings: &[BuiBinding]) -> Result<(), String> {
    for binding in bindings {
        let target = binding.target.trim();
        let source = binding.source.trim();

        if target.is_empty() {
            return Err("bindings.target must not be empty.".to_string());
        }
        if source.is_empty() {
            return Err("bindings.source must not be empty.".to_string());
        }
    }

    Ok(())
}

fn validate_state_visuals(states: &HashMap<String, BuiStateVisual>) -> Result<(), String> {
    for (name, state) in states {
        if name.trim().is_empty() {
            return Err("state_visuals keys must not be empty.".to_string());
        }

        validate_styles(&state.styles).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        if let Some(value) = &state.styles.visibility {
            parse_visibility(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        if let Some(value) = &state.styles.ui_translation {
            parse_val2(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        if let Some(value) = &state.styles.ui_scale {
            parse_vec2(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        if let Some(value) = &state.styles.ui_rotation {
            parse_rotation(value).map_err(|error| format!("state_visuals.{name}: {error}"))?;
        }
        validate_visuals(&state.visuals)?;

        if let Some(text_color) = &state.text_color {
            parse_color(text_color)?;
        }
    }

    Ok(())
}

fn validate_tab_semantics(node: &BuiNode) -> Result<(), String> {
    if let Some(group) = &node.tab_group_name
        && group.trim().is_empty()
    {
        return Err("tab_group_name must not be empty when present.".to_string());
    }

    if let Some(source) = &node.tab_binding_source {
        if source.trim().is_empty() {
            return Err("tab_binding_source must not be empty when present.".to_string());
        }
        if node.tab_group_name.is_none() {
            return Err("tab_binding_source requires tab_group_name.".to_string());
        }
    }

    if let Some(value) = &node.tab_value {
        if value.trim().is_empty() {
            return Err("tab_value must not be empty when present.".to_string());
        }
        if node.tab_group_name.is_none() {
            return Err("tab_value requires tab_group_name.".to_string());
        }
    }

    Ok(())
}

fn validate_progress_semantics(node: &BuiNode) -> Result<(), String> {
    if let Some(source) = &node.progress_binding_source
        && source.trim().is_empty()
    {
        return Err("progress_binding_source must not be empty when present.".to_string());
    }

    if node.progress_fill && !matches!(node.node_type, BuiNodeType::Node) {
        return Err("progress_fill is only supported on Node nodes.".to_string());
    }

    Ok(())
}

fn validate_list_semantics(node: &BuiNode) -> Result<(), String> {
    if let Some(source) = &node.list_binding_source {
        if source.trim().is_empty() {
            return Err("list_binding_source must not be empty when present.".to_string());
        }
        if !matches!(node.node_type, BuiNodeType::Node) {
            return Err("list_binding_source is only supported on Node nodes.".to_string());
        }
        if node.children.is_empty() {
            return Err("list_binding_source requires at least one child template.".to_string());
        }
    }

    Ok(())
}

fn validate_styles(styles: &BuiStyles) -> Result<(), String> {
    if let Some(target_camera) = &styles.ui_target_camera
        && target_camera.trim().is_empty()
    {
        return Err("styles.ui_target_camera must not be empty when present.".to_string());
    }
    if let Some(tab_group) = &styles.tab_group {
        parse_tab_group(tab_group)?;
    }
    if let Some(tab_index) = &styles.tab_index {
        parse_integer(tab_index)?;
    }
    if let Some(auto_focus) = styles.auto_focus
        && !auto_focus
    {
        // `false` is allowed but carries no behavior; keep it explicit and valid.
    }

    Ok(())
}

fn validate_image_config(image_config: &BuiImageConfig) -> Result<(), String> {
    if image_config.texture_path.trim().is_empty() {
        return Err("image_config.texture_path must not be empty.".to_string());
    }

    if let Some(image_mode) = &image_config.image_mode {
        parse_node_image_mode(image_mode, image_config.slicer.as_ref())?;
    }

    if image_config.slicer.is_some()
        && !matches!(image_config.image_mode.as_deref(), Some("sliced"))
    {
        return Err("image_config.slicer requires image_mode 'sliced'.".to_string());
    }

    if let Some(atlas) = &image_config.atlas {
        validate_texture_atlas_config(atlas)?;
    }

    if let Some(slicer) = &image_config.slicer {
        validate_texture_slicer_config(slicer)?;
    }

    Ok(())
}

fn validate_texture_atlas_config(atlas: &BuiTextureAtlasConfig) -> Result<(), String> {
    if atlas.tile_width == 0 || atlas.tile_height == 0 {
        return Err("image_config.atlas tile size must be greater than 0.".to_string());
    }
    if atlas.columns == 0 || atlas.rows == 0 {
        return Err("image_config.atlas grid size must be greater than 0.".to_string());
    }

    let cell_count = atlas.columns as usize * atlas.rows as usize;
    if atlas.index >= cell_count {
        return Err(format!(
            "image_config.atlas.index '{}' is out of range for a {}x{} atlas.",
            atlas.index, atlas.columns, atlas.rows
        ));
    }

    Ok(())
}

fn validate_texture_slicer_config(slicer: &BuiTextureSlicerConfig) -> Result<(), String> {
    if slicer.border < 0.0 {
        return Err("image_config.slicer.border must be non-negative.".to_string());
    }
    if slicer.max_corner_scale.is_some_and(|value| value < 0.0) {
        return Err("image_config.slicer.max_corner_scale must be non-negative.".to_string());
    }
    if slicer.stretch_value.is_some_and(|value| value < 0.0) {
        return Err("image_config.slicer.stretch_value must be non-negative.".to_string());
    }
    if let Some(mode) = &slicer.center_scale_mode {
        parse_slice_scale_mode(mode, slicer.stretch_value)?;
    }
    if let Some(mode) = &slicer.sides_scale_mode {
        parse_slice_scale_mode(mode, slicer.stretch_value)?;
    }

    Ok(())
}

fn reject_config(has_config: bool, path: &str, field: &str) -> Result<(), String> {
    if has_config {
        return Err(format!(
            "{path}: field '{field}' is not valid for this node type."
        ));
    }

    Ok(())
}

fn reject_children(node: &BuiNode, path: &str) -> Result<(), String> {
    if !node.children.is_empty() {
        return Err(format!(
            "{path}: {:?} nodes cannot have children.",
            node.node_type
        ));
    }

    Ok(())
}

fn build_image_node(
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    image_config: &BuiImageConfig,
) -> Result<ImageNode, String> {
    let image_mode = image_config
        .image_mode
        .as_deref()
        .map(|value| parse_node_image_mode(value, image_config.slicer.as_ref()))
        .transpose()?
        .unwrap_or_default();

    let image = asset_server.load(&image_config.texture_path);
    let mut image_node = if let Some(atlas) = &image_config.atlas {
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(atlas.tile_width, atlas.tile_height),
            atlas.columns,
            atlas.rows,
            atlas
                .padding_x
                .zip(atlas.padding_y)
                .map(|(x, y)| UVec2::new(x, y)),
            None,
        );
        let layout = texture_atlases.add(layout);
        ImageNode::from_atlas_image(
            image,
            TextureAtlas {
                layout,
                index: atlas.index,
            },
        )
    } else {
        ImageNode::new(image)
    };

    image_node.image_mode = image_mode;
    image_node.flip_x = image_config.flip_x;
    image_node.flip_y = image_config.flip_y;
    Ok(image_node)
}

fn spawn_bui_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    document: &BuiDocument,
) -> Result<Entity, String> {
    spawn_bui_node(commands, asset_server, texture_atlases, &document.root)
}

fn spawn_bui_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    node: &BuiNode,
) -> Result<Entity, String> {
    let mut entity_commands = commands.spawn_empty();
    let entity = entity_commands.id();

    insert_identity_components(&mut entity_commands, node);
    insert_visual_components(&mut entity_commands, node)?;
    insert_style_components(&mut entity_commands, node)?;

    match node.node_type {
        BuiNodeType::Node => {
            entity_commands.insert((build_node(&node.styles, &node.visuals)?, FocusPolicy::Pass));
        }
        BuiNodeType::Button => {
            entity_commands.insert((Button, build_node(&node.styles, &node.visuals)?));
            if let Some(image_config) = &node.image_config {
                entity_commands.insert(build_image_node(
                    asset_server,
                    texture_atlases,
                    image_config,
                )?);
            }
        }
        BuiNodeType::Toggle => {
            let bundle = (
                Button,
                Checkable,
                BuiToggle,
                build_node(&node.styles, &node.visuals)?,
            );
            entity_commands.insert(bundle);
            if node.custom_tags.iter().any(|tag| tag == "State_Checked") {
                entity_commands.insert(Checked);
            }
        }
        BuiNodeType::Text => {
            let text_config = node
                .text_config
                .as_ref()
                .ok_or_else(|| format!("Text node '{}' is missing text_config.", node.id))?;
            if text_config.placeholder.is_some() {
                return Err(format!(
                    "Text node '{}' cannot use text_config.placeholder.",
                    node.id
                ));
            }
            entity_commands.insert((
                build_node(&node.styles, &node.visuals)?,
                Text::new(text_config.content.clone()),
                TextFont {
                    font: load_font(asset_server, text_config.font_path.as_deref()),
                    font_size: FontSize::Px(text_config.font_size),
                    ..Default::default()
                },
                TextColor(parse_color(&text_config.font_color)?),
                text_layout(text_config)?,
                FocusPolicy::Pass,
            ));
            if let Some(text_shadow) = text_shadow(text_config)? {
                entity_commands.insert(text_shadow);
            }
        }
        BuiNodeType::TextInput => {
            let text_config = text_input_config(node)?;
            let text_font = TextFont {
                font: load_font(asset_server, text_config.font_path.as_deref()),
                font_size: FontSize::Px(text_config.font_size),
                ..Default::default()
            };
            let text_color = TextColor(parse_color(&text_config.font_color)?);
            entity_commands.insert((
                build_node(&node.styles, &node.visuals)?,
                EditableText {
                    visible_width: text_config.visible_width.or(Some(24.0)),
                    allow_newlines: text_config.allow_newlines.unwrap_or(false),
                    ..EditableText::new(&text_config.content)
                },
                text_layout(text_config)?,
                text_font.clone(),
                text_color,
                TextCursorStyle::default(),
                FocusPolicy::Block,
                BuiTextInput,
                text_config.clone(),
            ));

            let mirror = commands
                .spawn((
                    Text::new(initial_text_input_display(text_config, false)),
                    text_font,
                    text_color,
                    text_layout(text_config)?,
                    FocusPolicy::Pass,
                    BuiTextInputMirror { target: entity },
                ))
                .id();
            if let Some(text_shadow) = text_shadow(text_config)? {
                commands.entity(mirror).insert(text_shadow);
            }
            commands.entity(entity).add_child(mirror);
        }
        BuiNodeType::Image => {
            let image_config = node
                .image_config
                .as_ref()
                .ok_or_else(|| format!("Image node '{}' is missing image_config.", node.id))?;
            let image_node = build_image_node(asset_server, texture_atlases, image_config)?;
            entity_commands.insert((
                build_node(&node.styles, &node.visuals)?,
                image_node,
                FocusPolicy::Pass,
            ));
        }
    }

    if !matches!(node.node_type, BuiNodeType::TextInput) && node.list_binding_source.is_none() {
        let mut first_text_input_child = None;

        for child in &node.children {
            let child_entity = spawn_bui_node(commands, asset_server, texture_atlases, child)?;
            if first_text_input_child.is_none() && matches!(child.node_type, BuiNodeType::TextInput)
            {
                first_text_input_child = Some(child_entity);
            }
            commands.entity(entity).add_child(child_entity);
        }

        if let Some(target) = first_text_input_child {
            commands.entity(entity).insert((
                Interaction::default(),
                FocusPolicy::Block,
                BuiTextInputProxy { target },
            ));
        }
    }

    Ok(entity)
}

fn text_input_config(node: &BuiNode) -> Result<&BuiTextConfig, String> {
    if let Some(text_config) = &node.text_config {
        return Ok(text_config);
    }

    node.children
        .iter()
        .find_map(|child| {
            matches!(child.node_type, BuiNodeType::Text)
                .then_some(child.text_config.as_ref())
                .flatten()
        })
        .ok_or_else(|| {
            format!(
                "TextInput node '{}' is missing text_config and has no Text child fallback.",
                node.id
            )
        })
}

fn text_shadow(text_config: &BuiTextConfig) -> Result<Option<TextShadow>, String> {
    let Some(shadow) = &text_config.text_shadow else {
        return Ok(None);
    };

    let mut text_shadow = TextShadow::default();
    if let Some(offset_x) = shadow.offset_x {
        text_shadow.offset.x = offset_x;
    }
    if let Some(offset_y) = shadow.offset_y {
        text_shadow.offset.y = offset_y;
    }
    if let Some(color) = &shadow.color {
        text_shadow.color = parse_color(color)?;
    }

    Ok(Some(text_shadow))
}

fn text_layout(text_config: &BuiTextConfig) -> Result<TextLayout, String> {
    if let Some(linebreak) = &text_config.linebreak {
        return Ok(TextLayout::linebreak(parse_linebreak(linebreak)?));
    }

    if text_config.allow_newlines.unwrap_or(false) {
        return Ok(TextLayout::default());
    }

    Ok(TextLayout::no_wrap())
}

fn insert_identity_components(entity_commands: &mut EntityCommands, node: &BuiNode) {
    entity_commands.insert((Name::new(node.id.clone()), BuiId(node.id.clone())));

    if !node.custom_tags.is_empty() {
        entity_commands.insert(BuiLogicTags(node.custom_tags.clone()));
    }
    if !node.actions.is_empty() {
        entity_commands.insert(BuiActions(node.actions.clone()));
    }
    if !node.bindings.is_empty() {
        entity_commands.insert(BuiBindings(node.bindings.clone()));
    }
    if !node.state_visuals.is_empty() {
        entity_commands.insert(BuiVisualStateDefinitions {
            states: node.state_visuals.clone(),
        });
    }
    if node.custom_tags.iter().any(|tag| tag == "State_Disabled") {
        entity_commands.insert(BuiDisabled);
    }
    if let (Some(group), Some(source)) = (&node.tab_group_name, &node.tab_binding_source) {
        entity_commands.insert(BuiTabGroupDefinition {
            group: group.clone(),
            source: source.clone(),
        });
    }
    if let (Some(group), Some(value)) = (&node.tab_group_name, &node.tab_value) {
        entity_commands.insert(BuiTabItem {
            group: group.clone(),
            value: value.clone(),
        });
    }
    if let Some(source) = &node.progress_binding_source {
        entity_commands.insert(BuiProgressGroup {
            source: source.clone(),
        });
    }
    if node.progress_fill {
        entity_commands.insert(BuiProgressFill);
    }
    if let Some(source) = &node.list_binding_source
        && let Some(template) = node.children.first()
    {
        entity_commands.insert(BuiListDefinition {
            source: source.clone(),
            item_template: template.clone(),
        });
    }
}

fn insert_visual_components(
    entity_commands: &mut EntityCommands,
    node: &BuiNode,
) -> Result<(), String> {
    if let Some(color) = &node.visuals.background_color {
        entity_commands.insert(BackgroundColor(parse_color(color)?));
    }

    if let Some(color) = &node.visuals.border_color {
        entity_commands.insert(BorderColor::all(parse_color(color)?));
    }

    if let Some(shader_path) = &node.visuals.material_shader {
        entity_commands.insert(BuiMaterialShader {
            path: shader_path.clone(),
        });
    }

    Ok(())
}

fn insert_style_components(
    entity_commands: &mut EntityCommands,
    node: &BuiNode,
) -> Result<(), String> {
    if let Some(value) = &node.styles.visibility {
        entity_commands.insert(parse_visibility(value)?);
    }
    insert_ui_transform(entity_commands, &node.styles)?;
    if !has_ui_transform_styles(&node.styles)
        && node
            .state_visuals
            .values()
            .any(|state| has_ui_transform_styles(&state.styles))
    {
        entity_commands.insert(UiTransform::default());
    }
    if node.styles.relative_cursor_position.unwrap_or(false) {
        entity_commands.insert(RelativeCursorPosition::default());
    }
    if let Some(target_name) = &node.styles.ui_target_camera {
        entity_commands.insert(PendingUiTargetCamera {
            target_name: target_name.clone(),
        });
    }
    if let Some(value) = &node.styles.tab_group {
        entity_commands.insert(parse_tab_group(value)?);
    }
    if let Some(value) = &node.styles.tab_index {
        entity_commands.insert(TabIndex(parse_integer(value)?));
    }
    if node.styles.auto_focus.unwrap_or(false) {
        entity_commands.insert(AutoFocus);
    }
    if node.styles.fixed_node.unwrap_or(false) {
        entity_commands.insert(FixedNode);
    }
    if let Some(value) = &node.styles.z_index {
        entity_commands.insert(ZIndex(parse_integer(value)?));
    }
    if let Some(value) = &node.styles.global_z_index {
        entity_commands.insert(GlobalZIndex(parse_integer(value)?));
    }

    Ok(())
}

fn has_ui_transform_styles(styles: &BuiStyles) -> bool {
    styles.ui_translation.is_some() || styles.ui_scale.is_some() || styles.ui_rotation.is_some()
}

fn insert_ui_transform(
    entity_commands: &mut EntityCommands,
    styles: &BuiStyles,
) -> Result<(), String> {
    let mut ui_transform = UiTransform::default();
    let mut has_ui_transform = false;

    if let Some(value) = &styles.ui_translation {
        ui_transform.translation = parse_val2(value)?;
        has_ui_transform = true;
    }
    if let Some(value) = &styles.ui_scale {
        ui_transform.scale = parse_vec2(value)?;
        has_ui_transform = true;
    }
    if let Some(value) = &styles.ui_rotation {
        ui_transform.rotation = parse_rotation(value)?;
        has_ui_transform = true;
    }

    if has_ui_transform {
        entity_commands.insert(ui_transform);
    }

    Ok(())
}

fn build_node(styles: &BuiStyles, visuals: &BuiVisuals) -> Result<Node, String> {
    let mut node = Node::default();

    if let Some(value) = &styles.display {
        node.display = parse_display(value)?;
    }

    set_val(&mut node.width, &styles.width)?;
    set_val(&mut node.height, &styles.height)?;
    if let Some(value) = &styles.aspect_ratio {
        node.aspect_ratio = Some(parse_number(value)?);
    }
    set_val(&mut node.min_width, &styles.min_width)?;
    set_val(&mut node.min_height, &styles.min_height)?;
    set_val(&mut node.max_width, &styles.max_width)?;
    set_val(&mut node.max_height, &styles.max_height)?;
    set_val(&mut node.left, &styles.left)?;
    set_val(&mut node.right, &styles.right)?;
    set_val(&mut node.top, &styles.top)?;
    set_val(&mut node.bottom, &styles.bottom)?;
    if let Some(value) = &styles.overflow {
        node.overflow = parse_overflow(value)?;
    }
    if let Some(value) = &styles.overflow_clip_margin {
        node.overflow_clip_margin = parse_overflow_clip_margin(value)?;
    }

    if let Some(margin) = &styles.margin {
        node.margin = parse_ui_rect(margin)?;
    }
    set_val(&mut node.margin.left, &styles.margin_left)?;
    set_val(&mut node.margin.right, &styles.margin_right)?;
    set_val(&mut node.margin.top, &styles.margin_top)?;
    set_val(&mut node.margin.bottom, &styles.margin_bottom)?;

    if let Some(padding) = &styles.padding {
        node.padding = parse_ui_rect(padding)?;
    }
    set_val(&mut node.padding.left, &styles.padding_left)?;
    set_val(&mut node.padding.right, &styles.padding_right)?;
    set_val(&mut node.padding.top, &styles.padding_top)?;
    set_val(&mut node.padding.bottom, &styles.padding_bottom)?;

    if let Some(border_width) = &visuals.border_width {
        node.border = parse_ui_rect(border_width)?;
    }
    if let Some(border_radius) = &visuals.border_radius {
        node.border_radius = parse_border_radius(border_radius)?;
    }

    if let Some(value) = &styles.flex_direction {
        node.flex_direction = parse_flex_direction(value)?;
    }
    if let Some(value) = &styles.flex_wrap {
        node.flex_wrap = parse_flex_wrap(value)?;
    }
    if let Some(value) = &styles.flex_grow {
        node.flex_grow = parse_number(value)?;
    }
    if let Some(value) = &styles.flex_shrink {
        node.flex_shrink = parse_number(value)?;
    }
    if let Some(value) = &styles.flex_basis {
        node.flex_basis = parse_val(value)?;
    }
    set_val(&mut node.row_gap, &styles.row_gap)?;
    set_val(&mut node.column_gap, &styles.column_gap)?;
    if let Some(value) = &styles.justify_content {
        node.justify_content = parse_justify_content(value)?;
    }
    if let Some(value) = &styles.justify_items {
        node.justify_items = parse_justify_items(value)?;
    }
    if let Some(value) = &styles.align_content {
        node.align_content = parse_align_content(value)?;
    }
    if let Some(value) = &styles.align_items {
        node.align_items = parse_align_items(value)?;
    }
    if let Some(value) = &styles.align_self {
        node.align_self = parse_align_self(value)?;
    }
    if let Some(value) = &styles.justify_self {
        node.justify_self = parse_justify_self(value)?;
    }
    if let Some(value) = &styles.position_type {
        node.position_type = parse_position_type(value)?;
    }
    if let Some(value) = &styles.grid_template_columns {
        node.grid_template_columns = parse_grid_tracks(value)?;
    }
    if let Some(value) = &styles.grid_template_rows {
        node.grid_template_rows = parse_grid_tracks(value)?;
    }
    if let Some(value) = &styles.grid_column {
        node.grid_column = parse_grid_placement(value)?;
    }
    if let Some(value) = &styles.grid_row {
        node.grid_row = parse_grid_placement(value)?;
    }

    Ok(node)
}

fn set_val(target: &mut Val, source: &Option<String>) -> Result<(), String> {
    if let Some(value) = source {
        *target = parse_val(value)?;
    }
    Ok(())
}

fn load_font(asset_server: &AssetServer, font_path: Option<&str>) -> FontSource {
    let Some(font_path) = font_path else {
        return FontSource::default();
    };

    if asset_path_exists(font_path) {
        return FontSource::from(asset_server.load(font_path.to_owned()));
    }

    if let Some(windows_font) = windows_font_asset_path(font_path) {
        warn!(
            "BUI font path '{}' does not exist under the Bevy asset root. Loading '{}' from the optional windows_fonts asset source.",
            font_path, windows_font
        );
        return FontSource::from(asset_server.load(windows_font));
    }

    if let Some(macos_font) = macos_font_asset_path(font_path) {
        info!(
            "BUI font path '{}' does not exist under the Bevy asset root. Loading '{}' from an optional macOS font asset source.",
            font_path, macos_font
        );
        return FontSource::from(asset_server.load(macos_font));
    }

    warn!(
        "BUI font path '{}' does not exist under the Bevy asset root. Falling back to the default font.",
        font_path
    );
    FontSource::default()
}

fn asset_path_exists(asset_path: &str) -> bool {
    Path::new("assets").join(asset_path).exists()
}

fn windows_font_asset_path(asset_path: &str) -> Option<AssetPath<'static>> {
    let file_name = Path::new(asset_path).file_name()?.to_owned();
    let windows_font = Path::new("/mnt/c/Windows/Fonts").join(&file_name);

    windows_font.exists().then(|| {
        AssetPath::from_path_buf(PathBuf::from(file_name))
            .with_source(AssetSourceId::from("windows_fonts"))
    })
}

fn macos_font_asset_path(asset_path: &str) -> Option<AssetPath<'static>> {
    let file_name = Path::new(asset_path).file_name()?.to_owned();

    let macos_font = Path::new("/System/Library/Fonts").join(&file_name);
    if macos_font.exists() {
        return Some(
            AssetPath::from_path_buf(PathBuf::from(file_name))
                .with_source(AssetSourceId::from("macos_fonts")),
        );
    }

    let file_name = Path::new(asset_path).file_name()?.to_owned();
    let supplemental_font = Path::new("/System/Library/Fonts/Supplemental").join(&file_name);
    supplemental_font.exists().then(|| {
        AssetPath::from_path_buf(PathBuf::from(file_name))
            .with_source(AssetSourceId::from("macos_supplemental_fonts"))
    })
}

fn parse_val(value: &str) -> Result<Val, String> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("auto") {
        return Ok(Val::Auto);
    }

    if let Some(number) = value.strip_suffix("px") {
        return parse_number(number).map(Val::Px);
    }

    if let Some(number) = value.strip_suffix('%') {
        return parse_number(number).map(Val::Percent);
    }

    parse_number(value).map(Val::Px)
}

fn parse_ui_rect(value: &str) -> Result<UiRect, String> {
    let values = value
        .split_whitespace()
        .map(parse_val)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [all] => Ok(UiRect::all(*all)),
        [vertical, horizontal] => Ok(UiRect::axes(*horizontal, *vertical)),
        [top, horizontal, bottom] => Ok(UiRect {
            left: *horizontal,
            right: *horizontal,
            top: *top,
            bottom: *bottom,
        }),
        [top, right, bottom, left] => Ok(UiRect {
            left: *left,
            right: *right,
            top: *top,
            bottom: *bottom,
        }),
        _ => Err(format!("Invalid UiRect shorthand '{value}'.")),
    }
}

fn parse_border_radius(value: &str) -> Result<BorderRadius, String> {
    Ok(BorderRadius::all(parse_val(value)?))
}

fn parse_val2(value: &str) -> Result<Val2, String> {
    let values = value
        .split_whitespace()
        .map(parse_val)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [x, y] => Ok(Val2::new(*x, *y)),
        _ => Err(format!(
            "Invalid UiTransform translation '{value}'. Expected two values."
        )),
    }
}

fn parse_vec2(value: &str) -> Result<Vec2, String> {
    let values = value
        .split_whitespace()
        .map(parse_number)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [all] => Ok(Vec2::splat(*all)),
        [x, y] => Ok(Vec2::new(*x, *y)),
        _ => Err(format!(
            "Invalid UiTransform scale '{value}'. Expected one or two numbers."
        )),
    }
}

fn parse_rotation(value: &str) -> Result<Rot2, String> {
    let value = value.trim();

    if let Some(degrees) = value.strip_suffix("deg") {
        return parse_number(degrees).map(Rot2::degrees);
    }

    if let Some(radians) = value.strip_suffix("rad") {
        return parse_number(radians).map(Rot2::radians);
    }

    parse_number(value).map(Rot2::radians)
}

fn parse_color(value: &str) -> Result<Color, String> {
    if value.eq_ignore_ascii_case("transparent") {
        return Ok(Color::NONE);
    }

    Srgba::hex(value)
        .map(Color::from)
        .map_err(|error| format!("Invalid color '{value}': {error}"))
}

fn parse_display(value: &str) -> Result<Display, String> {
    match normalize_token(value).as_str() {
        "flex" => Ok(Display::Flex),
        "grid" => Ok(Display::Grid),
        "block" => Ok(Display::Block),
        "none" => Ok(Display::None),
        _ => Err(format!("Invalid display '{value}'.")),
    }
}

fn parse_visibility(value: &str) -> Result<Visibility, String> {
    match normalize_token(value).as_str() {
        "inherited" => Ok(Visibility::Inherited),
        "visible" => Ok(Visibility::Visible),
        "hidden" => Ok(Visibility::Hidden),
        _ => Err(format!("Invalid visibility '{value}'.")),
    }
}

fn parse_flex_direction(value: &str) -> Result<FlexDirection, String> {
    match normalize_token(value).as_str() {
        "row" => Ok(FlexDirection::Row),
        "row_reverse" => Ok(FlexDirection::RowReverse),
        "column" => Ok(FlexDirection::Column),
        "column_reverse" => Ok(FlexDirection::ColumnReverse),
        _ => Err(format!("Invalid flex_direction '{value}'.")),
    }
}

fn parse_flex_wrap(value: &str) -> Result<FlexWrap, String> {
    match normalize_token(value).as_str() {
        "no_wrap" | "nowrap" => Ok(FlexWrap::NoWrap),
        "wrap" => Ok(FlexWrap::Wrap),
        "wrap_reverse" => Ok(FlexWrap::WrapReverse),
        _ => Err(format!("Invalid flex_wrap '{value}'.")),
    }
}

fn parse_justify_content(value: &str) -> Result<JustifyContent, String> {
    match normalize_token(value).as_str() {
        "flex_start" | "start" => Ok(JustifyContent::FlexStart),
        "flex_end" | "end" => Ok(JustifyContent::FlexEnd),
        "center" => Ok(JustifyContent::Center),
        "space_between" => Ok(JustifyContent::SpaceBetween),
        "space_around" => Ok(JustifyContent::SpaceAround),
        "space_evenly" => Ok(JustifyContent::SpaceEvenly),
        _ => Err(format!("Invalid justify_content '{value}'.")),
    }
}

fn parse_justify_items(value: &str) -> Result<JustifyItems, String> {
    match normalize_token(value).as_str() {
        "default" => Ok(JustifyItems::Default),
        "start" => Ok(JustifyItems::Start),
        "end" => Ok(JustifyItems::End),
        "center" => Ok(JustifyItems::Center),
        "baseline" => Ok(JustifyItems::Baseline),
        "stretch" => Ok(JustifyItems::Stretch),
        _ => Err(format!("Invalid justify_items '{value}'.")),
    }
}

fn parse_align_content(value: &str) -> Result<AlignContent, String> {
    match normalize_token(value).as_str() {
        "default" => Ok(AlignContent::Default),
        "flex_start" | "start" => Ok(AlignContent::FlexStart),
        "flex_end" | "end" => Ok(AlignContent::FlexEnd),
        "center" => Ok(AlignContent::Center),
        "stretch" => Ok(AlignContent::Stretch),
        "space_between" => Ok(AlignContent::SpaceBetween),
        "space_around" => Ok(AlignContent::SpaceAround),
        "space_evenly" => Ok(AlignContent::SpaceEvenly),
        _ => Err(format!("Invalid align_content '{value}'.")),
    }
}

fn parse_align_items(value: &str) -> Result<AlignItems, String> {
    match normalize_token(value).as_str() {
        "default" => Ok(AlignItems::Default),
        "flex_start" | "start" => Ok(AlignItems::FlexStart),
        "flex_end" | "end" => Ok(AlignItems::FlexEnd),
        "center" => Ok(AlignItems::Center),
        "baseline" => Ok(AlignItems::Baseline),
        "stretch" => Ok(AlignItems::Stretch),
        _ => Err(format!("Invalid align_items '{value}'.")),
    }
}

fn parse_align_self(value: &str) -> Result<AlignSelf, String> {
    match normalize_token(value).as_str() {
        "auto" => Ok(AlignSelf::Auto),
        "flex_start" | "start" => Ok(AlignSelf::FlexStart),
        "flex_end" | "end" => Ok(AlignSelf::FlexEnd),
        "center" => Ok(AlignSelf::Center),
        "baseline" => Ok(AlignSelf::Baseline),
        "stretch" => Ok(AlignSelf::Stretch),
        _ => Err(format!("Invalid align_self '{value}'.")),
    }
}

fn parse_justify_self(value: &str) -> Result<JustifySelf, String> {
    match normalize_token(value).as_str() {
        "auto" => Ok(JustifySelf::Auto),
        "start" => Ok(JustifySelf::Start),
        "end" => Ok(JustifySelf::End),
        "center" => Ok(JustifySelf::Center),
        "baseline" => Ok(JustifySelf::Baseline),
        "stretch" => Ok(JustifySelf::Stretch),
        _ => Err(format!("Invalid justify_self '{value}'.")),
    }
}

fn parse_position_type(value: &str) -> Result<PositionType, String> {
    match normalize_token(value).as_str() {
        "relative" => Ok(PositionType::Relative),
        "absolute" => Ok(PositionType::Absolute),
        _ => Err(format!("Invalid position_type '{value}'.")),
    }
}

fn parse_overflow(value: &str) -> Result<Overflow, String> {
    match normalize_token(value).as_str() {
        "visible" => Ok(Overflow::visible()),
        "clip" => Ok(Overflow::clip()),
        "clip_x" => Ok(Overflow::clip_x()),
        "clip_y" => Ok(Overflow::clip_y()),
        "hidden" => Ok(Overflow::hidden()),
        "hidden_x" => Ok(Overflow::hidden_x()),
        "hidden_y" => Ok(Overflow::hidden_y()),
        "scroll" => Ok(Overflow::scroll()),
        "scroll_x" => Ok(Overflow::scroll_x()),
        "scroll_y" => Ok(Overflow::scroll_y()),
        _ => Err(format!("Invalid overflow '{value}'.")),
    }
}

fn parse_tab_group(value: &str) -> Result<TabGroup, String> {
    let normalized = normalize_token(value);
    if normalized == "modal" {
        return Ok(TabGroup::modal());
    }

    parse_integer(value).map(TabGroup::new)
}

fn parse_linebreak(value: &str) -> Result<LineBreak, String> {
    match normalize_token(value).as_str() {
        "word_boundary" => Ok(LineBreak::WordBoundary),
        "any_character" => Ok(LineBreak::AnyCharacter),
        "word_or_character" => Ok(LineBreak::WordOrCharacter),
        "no_wrap" => Ok(LineBreak::NoWrap),
        _ => Err(format!("Invalid text_config.linebreak '{value}'.")),
    }
}

fn parse_overflow_clip_margin(value: &str) -> Result<OverflowClipMargin, String> {
    let normalized = value.trim().replace(' ', "");

    if normalized.eq_ignore_ascii_case("border_box") {
        return Ok(OverflowClipMargin::border_box());
    }
    if normalized.eq_ignore_ascii_case("padding_box") {
        return Ok(OverflowClipMargin::padding_box());
    }
    if normalized.eq_ignore_ascii_case("content_box") {
        return Ok(OverflowClipMargin::content_box());
    }

    if let Some(argument) = normalized
        .strip_prefix("border_box(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return parse_number(argument)
            .map(|margin| OverflowClipMargin::border_box().with_margin(margin));
    }
    if let Some(argument) = normalized
        .strip_prefix("padding_box(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return parse_number(argument)
            .map(|margin| OverflowClipMargin::padding_box().with_margin(margin));
    }
    if let Some(argument) = normalized
        .strip_prefix("content_box(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return parse_number(argument)
            .map(|margin| OverflowClipMargin::content_box().with_margin(margin));
    }

    Err(format!("Invalid overflow_clip_margin '{value}'."))
}

fn parse_slice_scale_mode(
    value: &str,
    stretch_value: Option<f32>,
) -> Result<SliceScaleMode, String> {
    match normalize_token(value).as_str() {
        "stretch" => Ok(SliceScaleMode::Stretch),
        "tile" => Ok(SliceScaleMode::Tile {
            stretch_value: stretch_value.unwrap_or(1.0),
        }),
        _ => Err(format!("Invalid image_config.slicer scale mode '{value}'.")),
    }
}

fn parse_node_image_mode(
    value: &str,
    slicer: Option<&BuiTextureSlicerConfig>,
) -> Result<NodeImageMode, String> {
    match normalize_token(value).as_str() {
        "auto" => Ok(NodeImageMode::Auto),
        "stretch" => Ok(NodeImageMode::Stretch),
        "sliced" => {
            let Some(slicer) = slicer else {
                return Err("image_mode 'sliced' requires image_config.slicer.".to_string());
            };

            Ok(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(slicer.border),
                center_scale_mode: parse_slice_scale_mode(
                    slicer.center_scale_mode.as_deref().unwrap_or("stretch"),
                    slicer.stretch_value,
                )?,
                sides_scale_mode: parse_slice_scale_mode(
                    slicer.sides_scale_mode.as_deref().unwrap_or("stretch"),
                    slicer.stretch_value,
                )?,
                max_corner_scale: slicer.max_corner_scale.unwrap_or(1.0),
            }))
        }
        _ => Err(format!(
            "Invalid image_config.image_mode '{value}'. Supported values are 'auto', 'stretch', and 'sliced'."
        )),
    }
}

fn parse_grid_tracks(value: &str) -> Result<Vec<RepeatedGridTrack>, String> {
    let declarations = split_grid_track_declarations(value)?;

    declarations
        .into_iter()
        .map(|declaration| parse_single_grid_track(&declaration))
        .collect()
}

fn split_grid_track_declarations(value: &str) -> Result<Vec<String>, String> {
    let mut declarations = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for character in value.chars() {
        match character {
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                if depth == 0 {
                    return Err(format!("Invalid grid track declaration '{value}'."));
                }
                depth -= 1;
                current.push(character);
            }
            ',' | ' ' | '\n' | '\t' if depth == 0 => {
                if !current.trim().is_empty() {
                    declarations.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if depth != 0 {
        return Err(format!("Invalid grid track declaration '{value}'."));
    }

    if !current.trim().is_empty() {
        declarations.push(current.trim().to_string());
    }

    if declarations.is_empty() {
        return Err(format!("Invalid grid track declaration '{value}'."));
    }

    Ok(declarations)
}

fn parse_single_grid_track(value: &str) -> Result<RepeatedGridTrack, String> {
    let normalized = value.trim().replace(' ', "");

    if normalized.eq_ignore_ascii_case("auto") {
        return Ok(RepeatedGridTrack::auto(1));
    }

    if normalized.eq_ignore_ascii_case("min_content") {
        return Ok(RepeatedGridTrack::min_content(1));
    }

    if normalized.eq_ignore_ascii_case("max_content") {
        return Ok(RepeatedGridTrack::max_content(1));
    }

    if let Some(argument) = normalized
        .strip_prefix("px(")
        .and_then(|value| value.strip_suffix(')'))
    {
        if let Some((repetition, px_value)) = argument.split_once(',') {
            let repetition = repetition
                .parse::<u16>()
                .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
            let px_value = parse_number(px_value)?;
            return Ok(RepeatedGridTrack::px(repetition, px_value));
        }

        return parse_number(argument).map(|px_value| RepeatedGridTrack::px(1, px_value));
    }

    if let Some(argument) = normalized
        .strip_prefix("auto(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let repetition = argument
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
        return Ok(RepeatedGridTrack::auto(repetition));
    }

    if let Some(argument) = normalized
        .strip_prefix("fr(")
        .and_then(|value| value.strip_suffix(')'))
    {
        if argument.contains(',') {
            let (repetition, fraction) = parse_two_grid_args(argument, value)?;
            return Ok(RepeatedGridTrack::fr(repetition, fraction));
        }

        return parse_number(argument).map(|fraction| RepeatedGridTrack::fr(1, fraction));
    }

    if let Some(args) = normalized
        .strip_prefix("flex(")
        .and_then(|value| value.strip_suffix(')'))
    {
        if args.contains(',') {
            let (repetition, fraction) = parse_two_grid_args(args, value)?;
            return Ok(RepeatedGridTrack::flex(repetition, fraction));
        }

        return parse_number(args).map(|fraction| RepeatedGridTrack::flex(1, fraction));
    }

    if let Some(args) = normalized
        .strip_prefix("min_content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let repetition = args
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
        return Ok(RepeatedGridTrack::min_content(repetition));
    }

    if let Some(args) = normalized
        .strip_prefix("max_content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let repetition = args
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid track repetition in '{value}': {error}"))?;
        return Ok(RepeatedGridTrack::max_content(repetition));
    }

    Err(format!("Invalid grid track declaration '{value}'."))
}

fn parse_two_grid_args(args: &str, original: &str) -> Result<(u16, f32), String> {
    let Some((repetition, value)) = args.split_once(',') else {
        return Err(format!("Invalid grid track declaration '{original}'."));
    };

    let repetition = repetition
        .parse::<u16>()
        .map_err(|error| format!("Invalid grid track repetition in '{original}': {error}"))?;
    let value = parse_number(value)?;

    Ok((repetition, value))
}

fn parse_grid_placement(value: &str) -> Result<GridPlacement, String> {
    let normalized = value.trim().replace(' ', "");

    if normalized.eq_ignore_ascii_case("auto") {
        return Ok(GridPlacement::auto());
    }

    if let Some(span) = normalized
        .strip_prefix("span(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let span = span
            .parse::<u16>()
            .map_err(|error| format!("Invalid grid placement '{value}': {error}"))?;
        return Ok(GridPlacement::span(span));
    }

    Err(format!("Invalid grid placement '{value}'."))
}

fn parse_number(value: &str) -> Result<f32, String> {
    value
        .trim()
        .parse::<f32>()
        .map_err(|error| format!("Invalid number '{}': {error}", value.trim()))
}

fn parse_integer(value: &str) -> Result<i32, String> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|error| format!("Invalid integer '{}': {error}", value.trim()))
}

fn parse_action_trigger(value: &str) -> Result<BuiActionTrigger, String> {
    match normalize_token(value).as_str() {
        "press" | "pressed" => Ok(BuiActionTrigger::Press),
        "hover_enter" | "hovered" => Ok(BuiActionTrigger::HoverEnter),
        "hover_exit" | "unhovered" => Ok(BuiActionTrigger::HoverExit),
        _ => Err(format!(
            "Unsupported action event '{}'. Expected one of: press, hover_enter, hover_exit.",
            value
        )),
    }
}

fn parse_text_justify(value: &str) -> Result<Justify, String> {
    match normalize_token(value).as_str() {
        "left" => Ok(Justify::Left),
        "center" => Ok(Justify::Center),
        "right" => Ok(Justify::Right),
        "justified" | "justify" => Ok(Justify::Justified),
        "start" => Ok(Justify::Start),
        "end" => Ok(Justify::End),
        _ => Err(format!(
            "Invalid text justify '{value}'. Supported values are left, center, right, justified, start, end."
        )),
    }
}

fn normalize_token(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

fn spawn_error_text(commands: &mut Commands, error: String) {
    let root = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: px(24).all(),
            ..Default::default()
        })
        .id();

    let text = commands
        .spawn((
            Text::new(error),
            TextFont::from_font_size(22.0),
            TextColor(Color::srgb(1.0, 0.2, 0.2)),
        ))
        .id();

    commands.entity(root).add_child(text);
}

fn material_shader_notice_system(
    shaders: Query<(&BuiId, &BuiMaterialShader), Added<BuiMaterialShader>>,
) {
    for (id, shader) in &shaders {
        info!(
            "BUI node '{}' requested custom UI material shader '{}'.",
            id.0, shader.path
        );
    }
}

fn text_input_proxy_focus_system(
    mut input_focus: ResMut<InputFocus>,
    proxies: Query<(&Interaction, &BuiTextInputProxy), Changed<Interaction>>,
) {
    for (interaction, proxy) in &proxies {
        if *interaction == Interaction::Pressed {
            input_focus.set(proxy.target, FocusCause::Pressed);
        }
    }
}

fn dispatch_bui_tab_selection_system(
    tab_groups: Query<&BuiTabGroupDefinition>,
    tab_items: Query<(&Interaction, &BuiTabItem, Has<BuiDisabled>), Changed<Interaction>>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (interaction, tab_item, disabled) in &tab_items {
        if disabled || *interaction != Interaction::Pressed {
            continue;
        }

        let Some(group) = tab_groups
            .iter()
            .find(|group| group.group == tab_item.group)
        else {
            continue;
        };

        state_writer.write(BuiStateSet {
            key: group.source.clone(),
            value: BuiBindingValue::Text(tab_item.value.clone()),
        });
    }
}

fn sync_bui_tab_selected_state_system(
    tab_groups: Query<&BuiTabGroupDefinition>,
    tab_items: Query<(Entity, &BuiTabItem)>,
    state_store: Res<BuiStateStore>,
    mut commands: Commands,
) {
    if !state_store.is_changed() {
        return;
    }

    for (entity, tab_item) in &tab_items {
        let Some(group) = tab_groups
            .iter()
            .find(|group| group.group == tab_item.group)
        else {
            continue;
        };

        let selected = matches!(
            state_store.0.get(&group.source),
            Some(BuiBindingValue::Text(value)) if value == &tab_item.value
        );

        if selected {
            commands
                .entity(entity)
                .insert(BuiVisualState("selected".to_string()));
        } else {
            commands.entity(entity).remove::<BuiVisualState>();
        }
    }
}

fn sync_bui_progress_groups_system(
    state_store: Res<BuiStateStore>,
    groups: Query<(&BuiProgressGroup, &Children)>,
    fills: Query<(), With<BuiProgressFill>>,
    mut nodes: Query<&mut Node>,
) {
    if !state_store.is_changed() {
        return;
    }

    for (group, children) in &groups {
        let Some(BuiBindingValue::Number(value)) = state_store.0.get(&group.source) else {
            continue;
        };

        let ratio = value.clamp(0.0, 1.0) * 100.0;

        for child in children.iter() {
            if fills.get(child).is_err() {
                continue;
            }

            let Ok(mut node) = nodes.get_mut(child) else {
                continue;
            };

            node.width = Val::Percent(ratio);
        }
    }
}

fn sync_bui_list_groups_system(
    state_store: Res<BuiStateStore>,
    list_groups: Query<(Entity, &BuiListDefinition, Option<&Children>)>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    if !state_store.is_changed() {
        return;
    }

    for (entity, list, children) in &list_groups {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn_related::<Children>();
                commands.entity(child).despawn();
            }
        }

        match state_store.0.get(&list.source) {
            Some(BuiBindingValue::StringList(items)) => {
                for (index, item) in items.iter().enumerate() {
                    let template =
                        instantiate_list_item_template_text(&list.item_template, index, item);
                    let Ok(child_entity) = spawn_bui_node(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlases,
                        &template,
                    ) else {
                        continue;
                    };
                    commands.entity(entity).add_child(child_entity);
                }
            }
            Some(BuiBindingValue::ObjectList(items)) => {
                for (index, item) in items.iter().enumerate() {
                    let template =
                        instantiate_list_item_template_object(&list.item_template, index, item);
                    let Ok(child_entity) = spawn_bui_node(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlases,
                        &template,
                    ) else {
                        continue;
                    };
                    commands.entity(entity).add_child(child_entity);
                }
            }
            _ => {}
        }
    }
}

fn instantiate_list_item_template_text(template: &BuiNode, index: usize, item: &str) -> BuiNode {
    let mut node = template.clone();
    node.id = format!("{}__item_{index}", node.id);

    if let Some(text_config) = &mut node.text_config {
        text_config.content = text_config.content.replace("{{item}}", item);
    }

    node.children = node
        .children
        .iter()
        .map(|child| instantiate_list_item_template_text(child, index, item))
        .collect();

    node
}

fn instantiate_list_item_template_object(
    template: &BuiNode,
    index: usize,
    item: &HashMap<String, String>,
) -> BuiNode {
    let mut node = template.clone();
    node.id = format!("{}__item_{index}", node.id);

    if let Some(text_config) = &mut node.text_config {
        text_config.content = replace_template_tokens(&text_config.content, item);
    }

    node.children = node
        .children
        .iter()
        .map(|child| instantiate_list_item_template_object(child, index, item))
        .collect();

    node
}

fn replace_template_tokens(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (key, value) in values {
        let token = format!("{{{{{key}}}}}");
        result = result.replace(&token, value);
    }

    result
}

fn dispatch_bui_actions_system(
    interactions: Query<
        (Entity, &Interaction, &BuiId, &BuiActions, Has<BuiDisabled>),
        Changed<Interaction>,
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
) {
    for (entity, interaction, id, actions, disabled) in &interactions {
        if disabled {
            continue;
        }

        for action in &actions.0 {
            let Ok(trigger) = parse_action_trigger(&action.event) else {
                continue;
            };

            let matched = match trigger {
                BuiActionTrigger::Press => *interaction == Interaction::Pressed,
                BuiActionTrigger::HoverEnter => *interaction == Interaction::Hovered,
                BuiActionTrigger::HoverExit => *interaction == Interaction::None,
            };

            if !matched {
                continue;
            }

            action_writer.write(BuiActionTriggered {
                entity,
                id: id.0.clone(),
                action: action.emit.clone(),
                trigger,
            });
        }
    }
}

fn apply_bui_state_updates_system(
    mut updates: MessageReader<BuiStateSet>,
    mut state_store: ResMut<BuiStateStore>,
    mut binding_writer: MessageWriter<BuiBindingUpdate>,
) {
    for update in updates.read() {
        let key = update.key.clone();
        let value = update.value.clone();

        let changed = state_store.0.get(&key) != Some(&value);
        if !changed {
            continue;
        }

        state_store.0.insert(key.clone(), value.clone());
        binding_writer.write(BuiBindingUpdate { source: key, value });
    }
}

fn apply_bui_binding_updates_system(
    mut updates: MessageReader<BuiBindingUpdate>,
    mut nodes: Query<(&BuiBindings, &mut Node)>,
    mut ui_transforms: Query<(&BuiBindings, &mut UiTransform)>,
    mut texts: Query<(&BuiBindings, &mut Text)>,
    mut text_layouts: Query<(&BuiBindings, &mut TextLayout)>,
    mut text_bounds: Query<(&BuiBindings, &mut TextBounds)>,
    mut text_fonts: Query<(&BuiBindings, &mut TextFont)>,
    mut line_heights: Query<(&BuiBindings, &mut LineHeight)>,
    mut letter_spacings: Query<(&BuiBindings, &mut LetterSpacing)>,
    mut text_shadows: Query<(&BuiBindings, &mut TextShadow)>,
    mut text_colors: Query<(&BuiBindings, &mut TextColor)>,
    mut images: Query<(&BuiBindings, &mut ImageNode)>,
    mut backgrounds: Query<(&BuiBindings, &mut BackgroundColor)>,
    mut borders: Query<(&BuiBindings, &mut BorderColor)>,
    mut visibilities: Query<(&BuiBindings, &mut Visibility)>,
) {
    for update in updates.read() {
        for (bindings, mut node) in &mut nodes {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("display", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_display(value) {
                            node.display = parsed;
                        }
                    }
                    ("border_width", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_ui_rect(value) {
                            node.border = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut ui_transform) in &mut ui_transforms {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("ui_rotation", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_rotation(value) {
                            ui_transform.rotation = parsed;
                        }
                    }
                    ("ui_scale", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_vec2(value) {
                            ui_transform.scale = parsed;
                        }
                    }
                    ("ui_translation", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_val2(value) {
                            ui_transform.translation = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text) in &mut texts {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("text.content", BuiBindingValue::Text(value)) => {
                        text.0 = value.clone();
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text_layout) in &mut text_layouts {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("justify", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_text_justify(value) {
                            text_layout.justify = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut bounds) in &mut text_bounds {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("text_bounds.width", BuiBindingValue::Number(value)) => {
                        bounds.width = Some(*value);
                    }
                    ("text_bounds.height", BuiBindingValue::Number(value)) => {
                        bounds.height = Some(*value);
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text_font) in &mut text_fonts {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("font_size", BuiBindingValue::Number(value)) => {
                        text_font.font_size = FontSize::Px(*value);
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut line_height) in &mut line_heights {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("line_height", BuiBindingValue::Number(value)) => {
                        *line_height = LineHeight::RelativeToFont(*value);
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut letter_spacing) in &mut letter_spacings {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("letter_spacing", BuiBindingValue::Number(value)) => {
                        *letter_spacing = LetterSpacing::Px(*value);
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text_shadow) in &mut text_shadows {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("text_shadow.offset_x", BuiBindingValue::Number(value)) => {
                        text_shadow.offset.x = *value;
                    }
                    ("text_shadow.offset_y", BuiBindingValue::Number(value)) => {
                        text_shadow.offset.y = *value;
                    }
                    ("text_shadow.color", BuiBindingValue::Color(value)) => {
                        if let Ok(parsed) = parse_color(value) {
                            text_shadow.color = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text_color) in &mut text_colors {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("text.color", BuiBindingValue::Color(value)) => {
                        if let Ok(parsed) = parse_color(value) {
                            text_color.0 = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut image) in &mut images {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("image.tint", BuiBindingValue::Color(value)) => {
                        if let Ok(parsed) = parse_color(value) {
                            image.color = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut background) in &mut backgrounds {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("background_color", BuiBindingValue::Color(value)) => {
                        if let Ok(parsed) = parse_color(value) {
                            background.0 = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut border) in &mut borders {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("border_color", BuiBindingValue::Color(value)) => {
                        if let Ok(parsed) = parse_color(value) {
                            *border = BorderColor::all(parsed);
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut visibility) in &mut visibilities {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("visibility", BuiBindingValue::Bool(value)) => {
                        *visibility = if *value {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}

fn apply_bui_visual_states_system(
    input_focus: Res<InputFocus>,
    child_of_query: Query<&ChildOf>,
    explicit_states: Query<&BuiVisualState>,
    visual_states: Query<(
        Entity,
        &BuiVisualStateDefinitions,
        Option<&BuiVisualState>,
        Option<&Interaction>,
        Has<Checked>,
        Has<BuiDisabled>,
    )>,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut borders: Query<&mut BorderColor>,
    mut text_colors: Query<&mut TextColor>,
    mut ui_transforms: Query<&mut UiTransform>,
    mut visibilities: Query<&mut Visibility>,
) {
    for (entity, definitions, explicit_state, interaction, checked, disabled) in &visual_states {
        let inherited_state =
            inherited_visual_state(entity, &child_of_query, &explicit_states, explicit_state);
        let base_state = explicit_state
            .map(|state| state.0.clone())
            .or(inherited_state);
        let auto_state = disabled
            .then_some("disabled")
            .or_else(|| (input_focus.get() == Some(entity)).then_some("focused"))
            .or_else(|| checked.then_some("checked"))
            .or_else(|| {
                interaction.and_then(|interaction| match *interaction {
                    Interaction::Pressed => Some("pressed"),
                    Interaction::Hovered => Some("hovered"),
                    Interaction::None => Some("normal"),
                })
            })
            .or_else(|| {
                definitions
                    .states
                    .contains_key("normal")
                    .then_some("normal")
            });

        let Some(state_name) =
            resolve_visual_state_name(definitions, base_state.as_deref(), auto_state)
        else {
            continue;
        };

        let Some(state_visual) = definitions.states.get(&state_name) else {
            continue;
        };

        if let Some(color) = &state_visual.visuals.background_color
            && let Ok(mut background) = backgrounds.get_mut(entity)
            && let Ok(parsed) = parse_color(color)
        {
            background.0 = parsed;
        }

        if let Some(color) = &state_visual.visuals.border_color
            && let Ok(mut border) = borders.get_mut(entity)
            && let Ok(parsed) = parse_color(color)
        {
            *border = BorderColor::all(parsed);
        }

        if let Some(color) = &state_visual.text_color
            && let Ok(mut text_color) = text_colors.get_mut(entity)
            && let Ok(parsed) = parse_color(color)
        {
            text_color.0 = parsed;
        }

        if let Ok(mut ui_transform) = ui_transforms.get_mut(entity) {
            if let Some(value) = &state_visual.styles.ui_translation
                && let Ok(parsed) = parse_val2(value)
            {
                ui_transform.translation = parsed;
            }
            if let Some(value) = &state_visual.styles.ui_scale
                && let Ok(parsed) = parse_vec2(value)
            {
                ui_transform.scale = parsed;
            }
            if let Some(value) = &state_visual.styles.ui_rotation
                && let Ok(parsed) = parse_rotation(value)
            {
                ui_transform.rotation = parsed;
            }
        }

        if let Some(value) = &state_visual.styles.visibility
            && let Ok(mut visibility) = visibilities.get_mut(entity)
            && let Ok(parsed) = parse_visibility(value)
        {
            *visibility = parsed;
        }
    }
}

fn inherited_visual_state(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    explicit_states: &Query<&BuiVisualState>,
    self_state: Option<&BuiVisualState>,
) -> Option<String> {
    if self_state.is_some() {
        return None;
    }

    let mut current = entity;

    while let Ok(child_of) = child_of_query.get(current) {
        let parent = child_of.parent();
        if let Ok(state) = explicit_states.get(parent) {
            return Some(state.0.clone());
        }
        current = parent;
    }

    None
}

fn resolve_visual_state_name(
    definitions: &BuiVisualStateDefinitions,
    base_state: Option<&str>,
    auto_state: Option<&str>,
) -> Option<String> {
    if let (Some(base), Some(auto)) = (base_state, auto_state) {
        let combined = format!("{base}_{auto}");
        if definitions.states.contains_key(&combined) {
            return Some(combined);
        }

        let reversed = format!("{auto}_{base}");
        if definitions.states.contains_key(&reversed) {
            return Some(reversed);
        }
    }

    if let Some(base) = base_state
        && definitions.states.contains_key(base)
    {
        return Some(base.to_string());
    }

    if let Some(auto) = auto_state
        && definitions.states.contains_key(auto)
    {
        return Some(auto.to_string());
    }

    None
}

fn sync_text_input_mirror_system(
    input_focus: Res<InputFocus>,
    inputs: Query<(Entity, &EditableText, &BuiTextConfig), With<BuiTextInput>>,
    mut mirrors: Query<(&BuiTextInputMirror, &mut Text)>,
) {
    for (mirror, mut text) in &mut mirrors {
        let Ok((input_entity, editable_text, text_config)) = inputs.get(mirror.target) else {
            continue;
        };

        let is_focused = input_focus.get() == Some(input_entity);
        let display = current_text_input_display(editable_text, text_config, is_focused);

        if text.0 != display {
            text.0 = display;
        }
    }
}

fn initial_text_input_display(text_config: &BuiTextConfig, is_focused: bool) -> String {
    if text_config.content.is_empty() && !is_focused {
        return text_config.placeholder.clone().unwrap_or_default();
    }

    text_config.content.clone()
}

fn current_text_input_display(
    editable_text: &EditableText,
    text_config: &BuiTextConfig,
    is_focused: bool,
) -> String {
    let value = editable_text.value().to_string();

    if value.is_empty() && !is_focused {
        return text_config.placeholder.clone().unwrap_or_default();
    }

    value
}

fn toggle_interaction_system(
    mut commands: Commands,
    toggles: Query<(Entity, &Interaction, Has<Checked>), (Changed<Interaction>, With<BuiToggle>)>,
) {
    for (entity, interaction, checked) in &toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if checked {
            commands.entity(entity).remove::<Checked>();
        } else {
            commands.entity(entity).insert(Checked);
        }
    }
}

fn update_toggle_visual_system(
    toggles: Query<(&Children, Has<Checked>), With<BuiToggle>>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    for (children, checked) in &toggles {
        set_toggle_box_color(children, checked, &mut backgrounds);
    }
}

fn resolve_ui_target_camera_system(
    mut commands: Commands,
    pending_nodes: Query<(Entity, &PendingUiTargetCamera)>,
    named_entities: Query<(Entity, &Name)>,
) {
    for (entity, pending) in &pending_nodes {
        let Some((camera_entity, _)) = named_entities
            .iter()
            .find(|(_, name)| name.as_str() == pending.target_name)
        else {
            continue;
        };

        commands
            .entity(entity)
            .insert(UiTargetCamera(camera_entity))
            .remove::<PendingUiTargetCamera>();
    }
}

fn set_toggle_box_color(
    children: &Children,
    checked: bool,
    backgrounds: &mut Query<&mut BackgroundColor>,
) {
    let Some(first_child) = children.first() else {
        return;
    };

    let Ok(mut color) = backgrounds.get_mut(*first_child) else {
        return;
    };

    color.0 = if checked {
        Color::srgb(0.35, 0.75, 0.35)
    } else {
        Color::srgb(0.2, 0.2, 0.2)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    const VILLAGE_SHOP_HTML: &str = include_str!(
        "../../../examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.html"
    );
    const VILLAGE_SHOP_IR: &str = include_str!(
        "../../../examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.ir.json"
    );
    const QUEST_NOTICE_HTML: &str = include_str!(
        "../../../examples/UiParserTest/opendesignTest/quest_notice_overlay/quest-notice-overlay.html"
    );

    #[test]
    fn opendesign_media_query_rules_resolve_into_bui_styles() {
        let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
            .expect("OpenDesign HTML should compile");

        let panel = find_bui_node(&document.root, "panel");
        assert_eq!(panel.styles.width.as_deref(), Some("720px"));

        let card = find_bui_node(&document.root, "shop_card_hut");
        assert_eq!(
            card.styles.grid_template_columns.as_deref(),
            Some("flex(1) px(140)")
        );

        let item_main = find_bui_node(&document.root, "item_main_hut");
        assert_eq!(
            item_main.styles.grid_template_columns.as_deref(),
            Some("px(104) flex(1)")
        );
    }

    #[test]
    fn opendesign_active_transform_compiles_to_pressed_state_scale() {
        let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
            .expect("OpenDesign HTML should compile");

        let buy_button = find_bui_node(&document.root, "buy_btn_hut");
        assert_eq!(
            buy_button
                .state_visuals
                .get("pressed")
                .and_then(|state| state.styles.ui_scale.as_deref()),
            Some("0.95 0.95")
        );
        assert_eq!(
            buy_button
                .state_visuals
                .get("normal")
                .and_then(|state| state.styles.ui_scale.as_deref()),
            Some("1 1")
        );

        let close_button = find_bui_node(&document.root, "close_btn");
        assert_eq!(
            close_button
                .state_visuals
                .get("pressed")
                .and_then(|state| state.styles.ui_scale.as_deref()),
            Some("0.95 0.95")
        );
    }

    #[test]
    fn opendesign_text_nodes_do_not_inherit_button_transform_styles() {
        let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
            .expect("OpenDesign HTML should compile");

        let buy_text = find_bui_node(&document.root, "buy_btn_hut_text");
        assert!(buy_text.state_visuals.is_empty());
        assert_eq!(
            buy_text
                .text_config
                .as_ref()
                .map(|config| config.font_color.as_str()),
            Some("#ffffff")
        );
    }

    #[test]
    fn opendesign_ir_export_uses_3_0_shape() {
        let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
            .expect("OpenDesign HTML should compile");
        let ir = BuiIrDocument::from_compat_document(&document);

        assert_eq!(ir.version, "3.0-ir");
        assert_eq!(ir.root.kind, "node");

        let panel = ir
            .root
            .children
            .iter()
            .find(|child| child.id == "panel")
            .expect("panel should exist");
        assert_eq!(panel.layout.styles.max_width.as_deref(), Some("720px"));

        let buy_button = find_ir_node(&ir.root, "buy_btn_hut");
        assert_eq!(buy_button.kind, "button");
        assert!(buy_button.content.is_empty());
        assert!(
            buy_button
                .state_visuals
                .get("pressed")
                .and_then(|state| state.styles.ui_scale.as_deref())
                .is_some()
        );
    }

    #[test]
    fn opendesign_ir_snapshot_can_load_through_runtime_parser() {
        let ir_json = opendesign_html_to_bui_ir_json_str(VILLAGE_SHOP_HTML)
            .expect("OpenDesign HTML should compile to IR");
        let document = parse_bui_document(&ir_json).expect("BUI IR should parse for runtime");

        assert_eq!(document.version, EXPECTED_VERSION);

        let panel = find_bui_node(&document.root, "panel");
        assert_eq!(panel.styles.max_height.as_deref(), Some("648px"));

        let buy_button = find_bui_node(&document.root, "buy_btn_hut");
        assert!(matches!(buy_button.node_type, BuiNodeType::Button));
        assert_eq!(
            buy_button
                .state_visuals
                .get("pressed")
                .and_then(|state| state.styles.ui_scale.as_deref()),
            Some("0.95 0.95")
        );

        validate_bui_ir_json_str(&ir_json).expect("BUI IR validator should accept generated IR");
    }

    #[test]
    fn checked_in_ir_fixture_loads_through_runtime_parser() {
        let document = parse_bui_document(VILLAGE_SHOP_IR).expect("checked-in IR should parse");

        let root = find_bui_node(&document.root, "overlay_root");
        assert_eq!(root.styles.height.as_deref(), Some("100%"));

        let close_button = find_bui_node(&document.root, "close_btn");
        assert!(matches!(close_button.node_type, BuiNodeType::Button));
        assert_eq!(
            close_button
                .actions
                .first()
                .map(|action| (action.event.as_str(), action.emit.as_str())),
            Some(("press", "close_shop_overlay"))
        );
    }

    #[test]
    fn generic_opendesign_overlay_compiles_without_shop_structure() {
        let document = opendesign_html_to_bui_document(QUEST_NOTICE_HTML)
            .expect("generic OpenDesign overlay should compile");

        let title = find_bui_node(&document.root, "notice_title_text_1");
        assert_eq!(
            title
                .text_config
                .as_ref()
                .map(|text| text.content.as_str()),
            Some("新的委托")
        );

        let accept = find_bui_node(&document.root, "primary_btn");
        assert!(matches!(accept.node_type, BuiNodeType::Button));
        assert_eq!(
            accept
                .actions
                .first()
                .map(|action| (action.event.as_str(), action.emit.as_str())),
            Some(("press", "accept_quest"))
        );
        assert_eq!(
            accept
                .state_visuals
                .get("pressed")
                .and_then(|state| state.styles.ui_scale.as_deref()),
            Some("0.96 0.96")
        );

        let ir_json = opendesign_html_to_bui_ir_json_str(QUEST_NOTICE_HTML)
            .expect("generic OpenDesign overlay should compile to IR");
        validate_bui_ir_json_str(&ir_json).expect("generic IR should validate");
    }

    #[test]
    fn opendesign_css_important_values_are_normalized() {
        assert_eq!(normalize_css_value("  48px !important "), "48px");
        assert_eq!(normalize_css_value("\"#ffffff\" !important"), "#ffffff");
    }

    #[test]
    fn opendesign_color_mix_with_transparency_preserves_alpha() {
        assert_eq!(
            css_color("color-mix(in oklab, #3b2818, transparent 38%)").as_deref(),
            Some("#3B28189E")
        );
        assert_eq!(
            css_color("color-mix(in oklab, black, transparent 40%)").as_deref(),
            Some("#00000099")
        );
        assert_eq!(
            css_color("color-mix(in oklab, #fff, transparent)").as_deref(),
            Some("#FFFFFF80")
        );
    }

    #[test]
    fn unsupported_pseudo_element_selectors_do_not_leak_into_node_styles() {
        let document = opendesign_html_to_bui_document(VILLAGE_SHOP_HTML)
            .expect("OpenDesign HTML should compile");

        let scroll = find_bui_node(&document.root, "shop_scroll");
        assert_eq!(
            scroll.visuals.background_color, None,
            "::-webkit-scrollbar-thumb background should not leak into shop_scroll"
        );
        assert_eq!(
            scroll.visuals.border_radius, None,
            "::-webkit-scrollbar-thumb border-radius should not leak into shop_scroll"
        );
        assert_eq!(
            scroll.styles.width, None,
            "::-webkit-scrollbar width should not leak into shop_scroll"
        );
    }

    #[test]
    fn opendesign_length_functions_resolve_against_default_viewport() {
        assert_eq!(css_eval_length_function("min(100%, 460px)").as_deref(), Some("460px"));
        assert_eq!(css_eval_length_function("min(66vh, 620px)").as_deref(), Some("475.2px"));
        assert_eq!(css_eval_length_function("min(92vw, 720px)").as_deref(), Some("720px"));
        assert_eq!(css_eval_length_function("clamp(28px, 7vw, 36px)").as_deref(), Some("36px"));
        assert_eq!(css_eval_length_function("clamp(20px, 4vw, 24px)").as_deref(), Some("24px"));
        assert_eq!(css_eval_length_function("max(18px, env(safe-area-inset-top))").as_deref(), Some("18px"));
    }

    fn find_ir_node<'a>(node: &'a BuiIrNode, id: &str) -> &'a BuiIrNode {
        find_ir_node_optional(node, id).unwrap_or_else(|| panic!("IR node '{id}' should exist"))
    }

    fn find_ir_node_optional<'a>(node: &'a BuiIrNode, id: &str) -> Option<&'a BuiIrNode> {
        if node.id == id {
            return Some(node);
        }

        node.children
            .iter()
            .find_map(|child| find_ir_node_optional(child, id))
    }

    fn find_bui_node<'a>(node: &'a BuiNode, id: &str) -> &'a BuiNode {
        if node.id == id {
            return node;
        }

        node.children
            .iter()
            .find_map(|child| find_bui_node_optional(child, id))
            .unwrap_or_else(|| panic!("BUI node '{id}' should exist"))
    }

    fn find_bui_node_optional<'a>(node: &'a BuiNode, id: &str) -> Option<&'a BuiNode> {
        if node.id == id {
            return Some(node);
        }

        node.children
            .iter()
            .find_map(|child| find_bui_node_optional(child, id))
    }
}
