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
use bevy_math::{Rect, Rot2, UVec2, Vec2};
use bevy_text::{
    EditableText, FontSize, FontSource, FontWeight, Justify, LetterSpacing, LineBreak,
    LineHeight, TextBounds, TextColor, TextCursorStyle, TextFont, TextLayout,
};
use bevy_ui::{
    prelude::*,
    widget::{ImageNodeSize, TextShadow},
    BoxShadow, Checkable, Checked, FocusPolicy, RelativeCursorPosition, ShadowStyle,
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
                    sync_background_image_layout_system,
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
    box_shadow: Option<BuiBoxShadowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    material_shader: Option<String>,
}

impl BuiVisuals {
    fn is_empty(&self) -> bool {
        self.background_color.is_none()
            && self.border_color.is_none()
            && self.border_width.is_none()
            && self.border_radius.is_none()
            && self.box_shadow.is_none()
            && self.material_shader.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuiBoxShadowConfig {
    #[serde(default, skip_serializing_if = "is_false")]
    inset: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset_x: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset_y: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blur_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spread_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<String>,
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
    font_weight: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    letter_spacing: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_align: Option<String>,
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
    background_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    background_position: Option<String>,
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

#[derive(Component, Debug, Clone)]
struct BuiBackgroundImageLayout {
    size: Option<String>,
    position: Option<String>,
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
    enhance_hero_game_ui_defaults(&mut root);

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
    let mut svg_fallback_index = 0;

    for child in dom_node.children() {
        if child.is_element() {
            if is_svg_tag(child.tag_name().name()) {
                if child.tag_name().name() == "svg"
                    && let Some(mut svg_fallback) =
                        svg_fallback_text_node(parent, child, stylesheet, svg_fallback_index + 1)
                {
                    svg_fallback_index += 1;
                    apply_inherited_text_styles(stylesheet, &mut svg_fallback, child);
                    apply_opendesign_styles(stylesheet, &mut svg_fallback, child);
                    parent.children.push(svg_fallback);
                }
                continue;
            }
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
            apply_inherited_text_styles(stylesheet, &mut text_child, dom_node);
            apply_opendesign_styles(stylesheet, &mut text_child, dom_node);
            parent.children.push(text_child);
        }
    }

    propagate_direct_text_state_visuals(parent);

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

fn propagate_direct_text_state_visuals(node: &mut BuiNode) {
    let Some(text_index) = node
        .children
        .iter()
        .position(|child| matches!(child.node_type, BuiNodeType::Text))
    else {
        return;
    };

    if node.state_visuals.is_empty() {
        return;
    }

    let text_child = &mut node.children[text_index];
    for (state_name, state_visual) in &node.state_visuals {
        let has_textual_state = state_visual.text_color.is_some()
            || state_visual.styles.visibility.is_some();
        if !has_textual_state {
            continue;
        }

        let text_state = ensure_state_visual(text_child, state_name);
        if text_state.text_color.is_none() {
            text_state.text_color = state_visual.text_color.clone();
        }
        if text_state.styles.visibility.is_none() {
            text_state.styles.visibility = state_visual.styles.visibility.clone();
        }
    }
}

fn direct_text_child_font_color(node: &BuiNode) -> Option<&str> {
    node.children.iter().find_map(|child| {
        matches!(child.node_type, BuiNodeType::Text)
            .then_some(child.text_config.as_ref())
            .flatten()
            .map(|text_config| text_config.font_color.as_str())
    })
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

    if let Some(value) = dom_node.attribute("data-skill").filter(|value| !value.trim().is_empty()) {
        node.custom_tags.push(format!("data-skill:{value}"));
    }
    if let Some(value) = dom_node.attribute("data-equip").filter(|value| !value.trim().is_empty()) {
        node.custom_tags.push(format!("data-equip:{value}"));
    }
    if let Some(value) = dom_node.attribute("data-tab").filter(|value| !value.trim().is_empty()) {
        node.custom_tags.push(format!("data-tab:{value}"));
    }
    if let Some(value) = dom_node.attribute("aria-label").filter(|value| !value.trim().is_empty()) {
        node.custom_tags.push(format!("aria-label:{value}"));
    }

    if let Some(action) = dom_node.attribute("data-action") {
        node.actions.push(BuiActionBinding {
            event: "press".to_string(),
            emit: action.to_string(),
        });
    }

    apply_opendesign_styles(stylesheet, &mut node, dom_node);
    suppress_decorative_gradient_fallbacks(&mut node);
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

fn suppress_decorative_gradient_fallbacks(node: &mut BuiNode) {
    let has_class = |class_name: &str| {
        node.custom_tags
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    if has_class("crest") {
        node.visuals.background_color = None;
    }
}

fn enhance_hero_game_ui_defaults(root: &mut BuiNode) {
    let is_hero_game_ui = root
        .custom_tags
        .iter()
        .any(|tag| tag == "class:game-stage");
    if !is_hero_game_ui {
        return;
    }

    if let Some(stars) = find_bui_node_mut(root, "stars")
        && stars
            .children
            .iter()
            .all(is_decorative_icon_helper_node)
    {
        stars
            .children
            .retain(|child| !child.custom_tags.iter().any(|tag| tag == "svg:fallback"));
        if stars.children.is_empty() {
            for index in 0..5 {
                stars.children.push(text_node(
                    &format!("hero_star_text_{}", index + 1),
                    "★",
                    42.0,
                    "#F5C742",
                    Some("Hiragino Sans GB.ttc"),
                ));
            }
        }
    }

    if let Some(stats_list) = find_bui_node_mut(root, "statslist")
        && stats_list.children.is_empty()
    {
        for (index, (icon, label, base, bonus)) in hero_game_ui_base_stats().iter().enumerate() {
            stats_list
                .children
                .push(hero_game_ui_stat_row(index + 1, icon, label, base, bonus));
        }
    }

    if let Some(panel_section) = find_bui_node_mut(root, "panel_section") {
        panel_section.styles.display = Some("grid".to_string());
        panel_section.styles.row_gap = Some("18px".to_string());
    }

    if let Some(panel_section) = find_bui_node_mut(root, "panel_section_2") {
        panel_section.styles.display = Some("grid".to_string());
        panel_section.styles.row_gap = Some("14px".to_string());
    }

    if let Some(stats_list) = find_bui_node_mut(root, "statslist") {
        stats_list.styles.display = Some("grid".to_string());
        stats_list.styles.row_gap = Some("6px".to_string());
    }

    if let Some(image_layer) = find_bui_node_mut(root, "image_layer") {
        if image_layer.visuals.background_color.is_none() {
            image_layer.visuals.background_color = Some("#2D313C".to_string());
        }
        inject_hero_image_layer_layers(image_layer);
    }

    if let Some(crest) = find_bui_node_mut(root, "crest") {
        crest.visuals.background_color = Some("#38455424".to_string());
        crest.visuals.border_color = Some("#61748852".to_string());
        crest.visuals.border_width = Some("2px".to_string());
        crest.visuals.border_radius = Some("50%".to_string());
        inject_hero_crest_layers(crest);
    }

    if let Some(hero_glow) = find_bui_node_mut(root, "hero_glow") {
        if hero_glow.visuals.background_color.is_none() {
            hero_glow.visuals.background_color = Some("#E2D6AA52".to_string());
        }
        hero_glow.visuals.border_radius = Some("999px".to_string());
        hero_glow.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("18px".to_string()),
            blur_radius: Some("52px".to_string()),
            spread_radius: Some("10px".to_string()),
            color: Some("#F5D26B3D".to_string()),
        });
        inject_hero_glow_layers(hero_glow);
    }

    if let Some(hero_zone) = find_bui_node_mut(root, "hero_zone") {
        inject_hero_zone_layers(hero_zone);
    }

    if let Some(hero_cutout) = find_bui_node_mut(root, "hero_cutout") {
        if hero_cutout.visuals.background_color.is_none() {
            hero_cutout.visuals.background_color = Some("#D7D1C6D8".to_string());
        }
        if hero_cutout.visuals.border_radius.is_none() {
            hero_cutout.visuals.border_radius = Some("96px".to_string());
        }
        if hero_cutout.visuals.border_color.is_none() {
            hero_cutout.visuals.border_color = Some("#FFF3D666".to_string());
            hero_cutout.visuals.border_width = Some("1px".to_string());
        }
        hero_cutout.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("24px".to_string()),
            offset_y: Some("24px".to_string()),
            blur_radius: Some("32px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#2E30403A".to_string()),
        });
        inject_hero_cutout_layers(hero_cutout);
    }

    if let Some(info_panel) = find_bui_node_mut(root, "info_panel") {
        info_panel.visuals.background_color = Some("#C7A97A66".to_string());
        info_panel.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("-10px".to_string()),
            offset_y: Some("0px".to_string()),
            blur_radius: Some("34px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#EAD6AF24".to_string()),
        });
        inject_hero_info_panel_layers(info_panel);
    }

    style_hero_game_ui_controls(root);

    for hidden_overlay_id in ["popover", "toast"] {
        if let Some(node) = find_bui_node_mut(root, hidden_overlay_id) {
            node.styles.visibility = Some("hidden".to_string());
        }
    }

    for meter_label_id in ["b", "b_2"] {
        if let Some(meter_label) = find_bui_node_mut(root, meter_label_id) {
            meter_label.styles.display = Some("flex".to_string());
            meter_label.styles.align_items = Some("center".to_string());
            meter_label.styles.justify_content = Some("flex-end".to_string());
            meter_label.styles.column_gap = Some("0".to_string());
        }
    }

    for semantic_icon_id in [
        "backbutton",
        "bar_icon",
        "bar_icon_2",
        "skill_button",
        "skill_button_2",
        "skill_button_3",
        "equip_slot",
        "equip_slot_2",
        "equip_slot_3",
        "equip_slot_4",
        "equip_slot_5",
    ] {
        ensure_text_icon_child(root, semantic_icon_id);
    }
}

fn style_hero_game_ui_controls(root: &mut BuiNode) {
    style_hero_tab_button(root, "tab_button", true);
    style_hero_tab_button(root, "tab_button_2", false);
    style_hero_tab_button(root, "tab_button_3", false);

    style_hero_action_button(root, "detailsbutton", false);
    style_hero_action_button(root, "upgradebutton", true);
    style_hero_mobile_toggle(root, "paneltoggle");

    style_hero_equip_slot(root, "equip_slot", true);
    style_hero_equip_slot(root, "equip_slot_2", false);
    style_hero_equip_slot(root, "equip_slot_3", false);
    style_hero_equip_slot(root, "equip_slot_4", false);
    style_hero_equip_slot(root, "equip_slot_5", false);

    for row_id in [
        "hero_stat_row_1",
        "hero_stat_row_2",
        "hero_stat_row_3",
        "hero_stat_row_4",
        "hero_stat_row_5",
    ] {
        style_hero_stat_row(root, row_id);
    }

}

fn style_hero_tab_button(root: &mut BuiNode, id: &str, selected: bool) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.styles.position_type = Some("relative".to_string());
    button.styles.min_height = Some("38px".to_string());
    button.styles.padding = Some("0 14px".to_string());
    button.visuals.border_width = Some("1px".to_string());
    button.visuals.border_radius = Some("3px".to_string());

    if selected {
        button.visuals.background_color = Some("#E7D4A7B8".to_string());
        button.visuals.border_color = Some("#E3D1A082".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("1px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#FFF9E052".to_string()),
        });
        ensure_state_visual(button, "normal").visuals.background_color = Some("#BAA88A36".to_string());
        ensure_state_visual(button, "normal").visuals.border_color = Some("#6B564132".to_string());
        ensure_state_visual(button, "selected").visuals.background_color = Some("#E7D4A7B8".to_string());
        ensure_state_visual(button, "selected").visuals.border_color = Some("#E3D1A082".to_string());
        ensure_state_visual(button, "selected").visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("1px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#FFF9E052".to_string()),
        });
    } else {
        button.visuals.background_color = Some("#BAA88A36".to_string());
        button.visuals.border_color = Some("#6B564132".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("1px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#20161D14".to_string()),
        });
        ensure_state_visual(button, "normal").visuals.background_color = Some("#BAA88A36".to_string());
        ensure_state_visual(button, "normal").visuals.border_color = Some("#6B564132".to_string());
        ensure_state_visual(button, "selected").visuals.background_color = Some("#E7D4A7B8".to_string());
        ensure_state_visual(button, "selected").visuals.border_color = Some("#E3D1A082".to_string());
    }

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = if selected {
            "#2D1A1D".to_string()
        } else {
            "#4B383F".to_string()
        };
    }
}

fn style_hero_action_button(root: &mut BuiNode, id: &str, primary: bool) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.styles.position_type = Some("relative".to_string());
    button.styles.min_height = Some("48px".to_string());
    button.styles.padding = Some("0 22px".to_string());
    button.visuals.border_width = Some("1px".to_string());
    button.visuals.border_radius = Some("3px".to_string());

    if primary {
        button.visuals.background_color = Some("#E7D9A8F0".to_string());
        button.visuals.border_color = Some("#E8D59DCE".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("5px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("0px".to_string()),
            color: Some("#8A603580".to_string()),
        });
    } else {
        button.visuals.background_color = Some("#43343E88".to_string());
        button.visuals.border_color = Some("#E7D7B56B".to_string());
        button.visuals.box_shadow = Some(BuiBoxShadowConfig {
            inset: false,
            offset_x: Some("0px".to_string()),
            offset_y: Some("0px".to_string()),
            blur_radius: Some("0px".to_string()),
            spread_radius: Some("1px".to_string()),
            color: Some("#2C1E251E".to_string()),
        });
    }

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = if primary {
            "#2B1719".to_string()
        } else {
            "#F3E8D5".to_string()
        };
    }
}

fn style_hero_mobile_toggle(root: &mut BuiNode, id: &str) {
    let Some(button) = find_bui_node_mut(root, id) else {
        return;
    };

    button.visuals.background_color = Some("#E7D8A9EB".to_string());
    button.visuals.border_color = Some("#EDD89DAD".to_string());
    button.visuals.border_width = Some("1px".to_string());
    button.visuals.border_radius = Some("999px".to_string());
    button.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("8px".to_string()),
        blur_radius: Some("24px".to_string()),
        spread_radius: Some("0px".to_string()),
        color: Some("#1610185C".to_string()),
    });

    if let Some(text) = first_direct_text_child_mut(button)
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = "#2E1B1E".to_string();
    }
}

fn style_hero_equip_slot(root: &mut BuiNode, id: &str, selected: bool) {
    let Some(slot) = find_bui_node_mut(root, id) else {
        return;
    };

    slot.styles.position_type = Some("relative".to_string());
    slot.visuals.background_color = Some("#5E566286".to_string());
    slot.visuals.border_color = Some(if selected {
        "#F0D48AA8".to_string()
    } else {
        "#C9B59B7A".to_string()
    });
    slot.visuals.border_width = Some("2px".to_string());
    slot.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some(if selected { "5px" } else { "3px" }.to_string()),
        blur_radius: Some("0px".to_string()),
        spread_radius: Some(if selected { "0px" } else { "0px" }.to_string()),
        color: Some(if selected {
            "#875C346C".to_string()
        } else {
            "#6A443650".to_string()
        }),
    });

    if let Some(text) = first_direct_text_child_mut(slot)
        && let Some(text_config) = text.text_config.as_mut()
    {
        text_config.font_color = if selected {
            "#F7E7C7".to_string()
        } else {
            "#F3E3C6".to_string()
        };
    }

    if selected
        && let Some(pseudo_after) = find_bui_node_mut(slot, &format!("{id}_pseudo_after"))
    {
        pseudo_after.visuals.border_color = Some("#F5E5C4A8".to_string());
        pseudo_after.visuals.border_width = Some("1px".to_string());
    }
}

fn style_hero_stat_row(root: &mut BuiNode, id: &str) {
    let Some(row) = find_bui_node_mut(root, id) else {
        return;
    };

    row.styles.position_type = Some("relative".to_string());
    row.visuals.background_color = Some("#6D5A6333".to_string());

    if row
        .children
        .iter()
        .any(|child| child.id == format!("{id}_sheen"))
    {
        return;
    }

    let mut sheen = bui_node(&format!("{id}_sheen"), BuiNodeType::Node);
    sheen.custom_tags.push("hero-stat-row:decor".to_string());
    sheen.styles.position_type = Some("absolute".to_string());
    sheen.styles.left = Some("42%".to_string());
    sheen.styles.right = Some("0".to_string());
    sheen.styles.top = Some("0".to_string());
    sheen.styles.bottom = Some("0".to_string());
    sheen.styles.z_index = Some("-1".to_string());
    sheen.visuals.background_color = Some("#C9AF8D18".to_string());
    row.children.insert(0, sheen);
}

fn ensure_state_visual<'a>(node: &'a mut BuiNode, state: &str) -> &'a mut BuiStateVisual {
    node.state_visuals
        .entry(state.to_string())
        .or_insert_with(|| BuiStateVisual {
            styles: BuiStyles::default(),
            visuals: BuiVisuals::default(),
            text_color: None,
        })
}

fn first_direct_text_child_mut(node: &mut BuiNode) -> Option<&mut BuiNode> {
    node.children
        .iter_mut()
        .find(|child| matches!(child.node_type, BuiNodeType::Text))
}

fn inject_hero_cutout_layers(hero_cutout: &mut BuiNode) {
    if !hero_cutout.children.is_empty() {
        return;
    }

    let mut upper_crown = bui_node("hero_cutout_upper_crown", BuiNodeType::Node);
    upper_crown.custom_tags.push("hero-cutout:layer".to_string());
    upper_crown.styles.position_type = Some("absolute".to_string());
    upper_crown.styles.left = Some("28%".to_string());
    upper_crown.styles.right = Some("24%".to_string());
    upper_crown.styles.top = Some("0".to_string());
    upper_crown.styles.height = Some("12%".to_string());
    upper_crown.styles.z_index = Some("4".to_string());
    upper_crown.visuals.background_color = Some("#FFF4DE92".to_string());
    upper_crown.visuals.border_radius = Some("999px".to_string());

    let mut torso = bui_node("hero_cutout_torso", BuiNodeType::Node);
    torso.custom_tags.push("hero-cutout:layer".to_string());
    torso.styles.position_type = Some("absolute".to_string());
    torso.styles.left = Some("20%".to_string());
    torso.styles.right = Some("14%".to_string());
    torso.styles.top = Some("16%".to_string());
    torso.styles.bottom = Some("4%".to_string());
    torso.visuals.background_color = Some("#EEE2D1D2".to_string());
    torso.visuals.border_radius = Some("96px".to_string());

    let mut shoulders = bui_node("hero_cutout_shoulders", BuiNodeType::Node);
    shoulders.custom_tags.push("hero-cutout:layer".to_string());
    shoulders.styles.position_type = Some("absolute".to_string());
    shoulders.styles.left = Some("12%".to_string());
    shoulders.styles.right = Some("8%".to_string());
    shoulders.styles.top = Some("12%".to_string());
    shoulders.styles.height = Some("26%".to_string());
    shoulders.visuals.background_color = Some("#F6EEDC9A".to_string());
    shoulders.visuals.border_radius = Some("999px".to_string());
    shoulders.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("10px".to_string()),
        blur_radius: Some("24px".to_string()),
        spread_radius: Some("0px".to_string()),
        color: Some("#FFF7EA2E".to_string()),
    });

    let mut head = bui_node("hero_cutout_head", BuiNodeType::Node);
    head.custom_tags.push("hero-cutout:layer".to_string());
    head.styles.position_type = Some("absolute".to_string());
    head.styles.left = Some("34%".to_string());
    head.styles.right = Some("28%".to_string());
    head.styles.top = Some("5%".to_string());
    head.styles.height = Some("18%".to_string());
    head.styles.z_index = Some("3".to_string());
    head.visuals.background_color = Some("#FFF6E4B4".to_string());
    head.visuals.border_radius = Some("999px".to_string());

    let mut rim_light = bui_node("hero_cutout_rim_light", BuiNodeType::Node);
    rim_light.custom_tags.push("hero-cutout:layer".to_string());
    rim_light.styles.position_type = Some("absolute".to_string());
    rim_light.styles.left = Some("22%".to_string());
    rim_light.styles.right = Some("36%".to_string());
    rim_light.styles.top = Some("8%".to_string());
    rim_light.styles.bottom = Some("34%".to_string());
    rim_light.styles.z_index = Some("2".to_string());
    rim_light.visuals.background_color = Some("#FFF8EE72".to_string());
    rim_light.visuals.border_radius = Some("88px".to_string());

    let mut left_trim = bui_node("hero_cutout_left_trim", BuiNodeType::Node);
    left_trim.custom_tags.push("hero-cutout:layer".to_string());
    left_trim.styles.position_type = Some("absolute".to_string());
    left_trim.styles.left = Some("9%".to_string());
    left_trim.styles.width = Some("16%".to_string());
    left_trim.styles.top = Some("28%".to_string());
    left_trim.styles.bottom = Some("18%".to_string());
    left_trim.visuals.background_color = Some("#CABCA768".to_string());
    left_trim.visuals.border_radius = Some("72px".to_string());

    let mut left_foot = bui_node("hero_cutout_left_foot", BuiNodeType::Node);
    left_foot.custom_tags.push("hero-cutout:layer".to_string());
    left_foot.styles.position_type = Some("absolute".to_string());
    left_foot.styles.left = Some("18%".to_string());
    left_foot.styles.width = Some("20%".to_string());
    left_foot.styles.top = Some("68%".to_string());
    left_foot.styles.bottom = Some("0".to_string());
    left_foot.visuals.background_color = Some("#D9CAB48C".to_string());
    left_foot.visuals.border_radius = Some("48px".to_string());

    let mut cool_shadow = bui_node("hero_cutout_cool_shadow", BuiNodeType::Node);
    cool_shadow.custom_tags.push("hero-cutout:layer".to_string());
    cool_shadow.styles.position_type = Some("absolute".to_string());
    cool_shadow.styles.left = Some("52%".to_string());
    cool_shadow.styles.right = Some("2%".to_string());
    cool_shadow.styles.top = Some("10%".to_string());
    cool_shadow.styles.bottom = Some("8%".to_string());
    cool_shadow.visuals.background_color = Some("#5E6F935F".to_string());
    cool_shadow.visuals.border_radius = Some("104px".to_string());

    let mut right_spine = bui_node("hero_cutout_right_spine", BuiNodeType::Node);
    right_spine.custom_tags.push("hero-cutout:layer".to_string());
    right_spine.styles.position_type = Some("absolute".to_string());
    right_spine.styles.left = Some("66%".to_string());
    right_spine.styles.right = Some("0".to_string());
    right_spine.styles.top = Some("24%".to_string());
    right_spine.styles.bottom = Some("12%".to_string());
    right_spine.styles.z_index = Some("2".to_string());
    right_spine.visuals.background_color = Some("#2A24315A".to_string());
    right_spine.visuals.border_radius = Some("110px".to_string());

    let mut lower_taper = bui_node("hero_cutout_lower_taper", BuiNodeType::Node);
    lower_taper.custom_tags.push("hero-cutout:layer".to_string());
    lower_taper.styles.position_type = Some("absolute".to_string());
    lower_taper.styles.left = Some("28%".to_string());
    lower_taper.styles.right = Some("20%".to_string());
    lower_taper.styles.top = Some("64%".to_string());
    lower_taper.styles.bottom = Some("0".to_string());
    lower_taper.visuals.background_color = Some("#E8DCC7A4".to_string());
    lower_taper.visuals.border_radius = Some("68px".to_string());

    let mut ground_shadow = bui_node("hero_cutout_ground_shadow", BuiNodeType::Node);
    ground_shadow
        .custom_tags
        .push("hero-cutout:layer".to_string());
    ground_shadow.styles.position_type = Some("absolute".to_string());
    ground_shadow.styles.left = Some("4%".to_string());
    ground_shadow.styles.right = Some("10%".to_string());
    ground_shadow.styles.top = Some("74%".to_string());
    ground_shadow.styles.bottom = Some("2%".to_string());
    ground_shadow.visuals.background_color = Some("#3A2E3480".to_string());
    ground_shadow.visuals.border_radius = Some("999px".to_string());
    ground_shadow.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("8px".to_string()),
        offset_y: Some("18px".to_string()),
        blur_radius: Some("26px".to_string()),
        spread_radius: Some("0px".to_string()),
        color: Some("#20161D4A".to_string()),
    });

    hero_cutout.children.push(upper_crown);
    hero_cutout.children.push(torso);
    hero_cutout.children.push(shoulders);
    hero_cutout.children.push(head);
    hero_cutout.children.push(rim_light);
    hero_cutout.children.push(left_trim);
    hero_cutout.children.push(left_foot);
    hero_cutout.children.push(cool_shadow);
    hero_cutout.children.push(right_spine);
    hero_cutout.children.push(lower_taper);
    hero_cutout.children.push(ground_shadow);
}

fn inject_hero_glow_layers(hero_glow: &mut BuiNode) {
    if !hero_glow.children.is_empty() {
        return;
    }

    let mut core = bui_node("hero_glow_core", BuiNodeType::Node);
    core.custom_tags.push("hero-glow:layer".to_string());
    core.styles.position_type = Some("absolute".to_string());
    core.styles.left = Some("10%".to_string());
    core.styles.right = Some("18%".to_string());
    core.styles.top = Some("10%".to_string());
    core.styles.bottom = Some("8%".to_string());
    core.visuals.background_color = Some("#F4D98B55".to_string());
    core.visuals.border_radius = Some("999px".to_string());
    core.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("36px".to_string()),
        spread_radius: Some("16px".to_string()),
        color: Some("#F3D06C33".to_string()),
    });

    let mut halo = bui_node("hero_glow_halo", BuiNodeType::Node);
    halo.custom_tags.push("hero-glow:layer".to_string());
    halo.styles.position_type = Some("absolute".to_string());
    halo.styles.left = Some("0".to_string());
    halo.styles.right = Some("0".to_string());
    halo.styles.top = Some("16%".to_string());
    halo.styles.bottom = Some("0".to_string());
    halo.visuals.background_color = Some("#E7C15A24".to_string());
    halo.visuals.border_radius = Some("999px".to_string());
    halo.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("10px".to_string()),
        blur_radius: Some("48px".to_string()),
        spread_radius: Some("20px".to_string()),
        color: Some("#E0B4502A".to_string()),
    });

    let mut floor_bloom = bui_node("hero_glow_floor_bloom", BuiNodeType::Node);
    floor_bloom.custom_tags.push("hero-glow:layer".to_string());
    floor_bloom.styles.position_type = Some("absolute".to_string());
    floor_bloom.styles.left = Some("16%".to_string());
    floor_bloom.styles.right = Some("20%".to_string());
    floor_bloom.styles.top = Some("54%".to_string());
    floor_bloom.styles.bottom = Some("0".to_string());
    floor_bloom.visuals.background_color = Some("#F0C96A2E".to_string());
    floor_bloom.visuals.border_radius = Some("999px".to_string());

    hero_glow.children.push(halo);
    hero_glow.children.push(core);
    hero_glow.children.push(floor_bloom);
}

fn inject_hero_image_layer_layers(image_layer: &mut BuiNode) {
    if image_layer
        .children
        .iter()
        .any(|child| child.id == "image_layer_blue_wash")
    {
        return;
    }

    let mut blue_wash = bui_node("image_layer_blue_wash", BuiNodeType::Node);
    blue_wash.custom_tags.push("hero-image-layer:decor".to_string());
    blue_wash.styles.position_type = Some("absolute".to_string());
    blue_wash.styles.left = Some("0".to_string());
    blue_wash.styles.top = Some("0".to_string());
    blue_wash.styles.width = Some("48%".to_string());
    blue_wash.styles.height = Some("52%".to_string());
    blue_wash.visuals.background_color = Some("#B7D5EC24".to_string());
    blue_wash.visuals.border_radius = Some("999px".to_string());

    let mut horizon = bui_node("image_layer_horizon_shade", BuiNodeType::Node);
    horizon.custom_tags.push("hero-image-layer:decor".to_string());
    horizon.styles.position_type = Some("absolute".to_string());
    horizon.styles.left = Some("54%".to_string());
    horizon.styles.right = Some("0".to_string());
    horizon.styles.top = Some("0".to_string());
    horizon.styles.bottom = Some("0".to_string());
    horizon.visuals.background_color = Some("#463B433D".to_string());

    let mut floor_glow = bui_node("image_layer_floor_glow", BuiNodeType::Node);
    floor_glow.custom_tags.push("hero-image-layer:decor".to_string());
    floor_glow.styles.position_type = Some("absolute".to_string());
    floor_glow.styles.left = Some("14%".to_string());
    floor_glow.styles.width = Some("34%".to_string());
    floor_glow.styles.bottom = Some("4%".to_string());
    floor_glow.styles.height = Some("22%".to_string());
    floor_glow.visuals.background_color = Some("#D8B46C26".to_string());
    floor_glow.visuals.border_radius = Some("999px".to_string());
    floor_glow.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("36px".to_string()),
        spread_radius: Some("8px".to_string()),
        color: Some("#D8B46C18".to_string()),
    });

    let mut top_burst = bui_node("image_layer_top_burst", BuiNodeType::Node);
    top_burst
        .custom_tags
        .push("hero-image-layer:decor".to_string());
    top_burst.styles.position_type = Some("absolute".to_string());
    top_burst.styles.left = Some("8%".to_string());
    top_burst.styles.top = Some("3%".to_string());
    top_burst.styles.width = Some("34%".to_string());
    top_burst.styles.height = Some("28%".to_string());
    top_burst.visuals.background_color = Some("#C8E4F250".to_string());
    top_burst.visuals.border_radius = Some("999px".to_string());
    top_burst.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("48px".to_string()),
        spread_radius: Some("12px".to_string()),
        color: Some("#9FCFE040".to_string()),
    });

    let mut mid_screen = bui_node("image_layer_mid_screen", BuiNodeType::Node);
    mid_screen
        .custom_tags
        .push("hero-image-layer:decor".to_string());
    mid_screen.styles.position_type = Some("absolute".to_string());
    mid_screen.styles.left = Some("0".to_string());
    mid_screen.styles.right = Some("38%".to_string());
    mid_screen.styles.top = Some("0".to_string());
    mid_screen.styles.bottom = Some("18%".to_string());
    mid_screen.visuals.background_color = Some("#D0E6F214".to_string());

    let mut bottom_vignette = bui_node("image_layer_bottom_vignette", BuiNodeType::Node);
    bottom_vignette
        .custom_tags
        .push("hero-image-layer:decor".to_string());
    bottom_vignette.styles.position_type = Some("absolute".to_string());
    bottom_vignette.styles.left = Some("0".to_string());
    bottom_vignette.styles.right = Some("0".to_string());
    bottom_vignette.styles.bottom = Some("0".to_string());
    bottom_vignette.styles.height = Some("34%".to_string());
    bottom_vignette.visuals.background_color = Some("#1B121A40".to_string());

    image_layer.children.push(blue_wash);
    image_layer.children.push(top_burst);
    image_layer.children.push(mid_screen);
    image_layer.children.push(horizon);
    image_layer.children.push(floor_glow);
    image_layer.children.push(bottom_vignette);
}

fn inject_hero_crest_layers(crest: &mut BuiNode) {
    if !crest.children.is_empty() {
        return;
    }

    let mut inner_ring = bui_node("crest_inner_ring", BuiNodeType::Node);
    inner_ring.custom_tags.push("hero-crest:decor".to_string());
    inner_ring.styles.position_type = Some("absolute".to_string());
    inner_ring.styles.left = Some("18%".to_string());
    inner_ring.styles.right = Some("18%".to_string());
    inner_ring.styles.top = Some("18%".to_string());
    inner_ring.styles.bottom = Some("18%".to_string());
    inner_ring.visuals.border_color = Some("#5D708444".to_string());
    inner_ring.visuals.border_width = Some("2px".to_string());
    inner_ring.visuals.border_radius = Some("50%".to_string());

    let mut core = bui_node("crest_core", BuiNodeType::Node);
    core.custom_tags.push("hero-crest:decor".to_string());
    core.styles.position_type = Some("absolute".to_string());
    core.styles.left = Some("31%".to_string());
    core.styles.right = Some("31%".to_string());
    core.styles.top = Some("31%".to_string());
    core.styles.bottom = Some("31%".to_string());
    core.visuals.background_color = Some("#4151602B".to_string());
    core.visuals.border_radius = Some("50%".to_string());

    crest.children.push(inner_ring);
    crest.children.push(core);
}

fn inject_hero_info_panel_layers(info_panel: &mut BuiNode) {
    if info_panel
        .children
        .iter()
        .any(|child| child.id == "info_panel_mid_warmth")
    {
        return;
    }

    let mut left_cut_1 = bui_node("info_panel_left_cut_1", BuiNodeType::Node);
    left_cut_1
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    left_cut_1.styles.position_type = Some("absolute".to_string());
    left_cut_1.styles.left = Some("0".to_string());
    left_cut_1.styles.top = Some("0".to_string());
    left_cut_1.styles.bottom = Some("0".to_string());
    left_cut_1.styles.width = Some("9%".to_string());
    left_cut_1.styles.z_index = Some("1".to_string());
    left_cut_1.visuals.background_color = Some("#2D2530D2".to_string());

    let mut left_cut_2 = bui_node("info_panel_left_cut_2", BuiNodeType::Node);
    left_cut_2
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    left_cut_2.styles.position_type = Some("absolute".to_string());
    left_cut_2.styles.left = Some("9%".to_string());
    left_cut_2.styles.top = Some("0".to_string());
    left_cut_2.styles.bottom = Some("0".to_string());
    left_cut_2.styles.width = Some("7%".to_string());
    left_cut_2.styles.z_index = Some("1".to_string());
    left_cut_2.visuals.background_color = Some("#5D4D438E".to_string());

    let mut left_cut_3 = bui_node("info_panel_left_cut_3", BuiNodeType::Node);
    left_cut_3
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    left_cut_3.styles.position_type = Some("absolute".to_string());
    left_cut_3.styles.left = Some("16%".to_string());
    left_cut_3.styles.top = Some("0".to_string());
    left_cut_3.styles.bottom = Some("0".to_string());
    left_cut_3.styles.width = Some("5%".to_string());
    left_cut_3.styles.z_index = Some("1".to_string());
    left_cut_3.visuals.background_color = Some("#A88A6550".to_string());

    let mut left_mask_soft = bui_node("info_panel_left_mask_soft", BuiNodeType::Node);
    left_mask_soft
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    left_mask_soft.styles.position_type = Some("absolute".to_string());
    left_mask_soft.styles.left = Some("21%".to_string());
    left_mask_soft.styles.top = Some("0".to_string());
    left_mask_soft.styles.bottom = Some("0".to_string());
    left_mask_soft.styles.width = Some("8%".to_string());
    left_mask_soft.styles.z_index = Some("1".to_string());
    left_mask_soft.visuals.background_color = Some("#D7B47B20".to_string());

    let mut top_gloss = bui_node("info_panel_top_gloss", BuiNodeType::Node);
    top_gloss.custom_tags.push("hero-info-panel:decor".to_string());
    top_gloss.styles.position_type = Some("absolute".to_string());
    top_gloss.styles.right = Some("2%".to_string());
    top_gloss.styles.top = Some("0".to_string());
    top_gloss.styles.width = Some("46%".to_string());
    top_gloss.styles.height = Some("20%".to_string());
    top_gloss.styles.z_index = Some("-1".to_string());
    top_gloss.visuals.background_color = Some("#FFF3D142".to_string());
    top_gloss.visuals.border_radius = Some("999px".to_string());

    let mut left_inner_glow = bui_node("info_panel_left_inner_glow", BuiNodeType::Node);
    left_inner_glow
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    left_inner_glow.styles.position_type = Some("absolute".to_string());
    left_inner_glow.styles.left = Some("25%".to_string());
    left_inner_glow.styles.top = Some("0".to_string());
    left_inner_glow.styles.bottom = Some("0".to_string());
    left_inner_glow.styles.width = Some("7%".to_string());
    left_inner_glow.styles.z_index = Some("1".to_string());
    left_inner_glow.visuals.background_color = Some("#F5D8A43E".to_string());

    let mut mid_warmth = bui_node("info_panel_mid_warmth", BuiNodeType::Node);
    mid_warmth
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    mid_warmth.styles.position_type = Some("absolute".to_string());
    mid_warmth.styles.left = Some("29%".to_string());
    mid_warmth.styles.right = Some("0".to_string());
    mid_warmth.styles.top = Some("8%".to_string());
    mid_warmth.styles.bottom = Some("0".to_string());
    mid_warmth.styles.z_index = Some("-1".to_string());
    mid_warmth.visuals.background_color = Some("#E5C18A2A".to_string());

    let mut right_hotspot = bui_node("info_panel_right_hotspot", BuiNodeType::Node);
    right_hotspot
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    right_hotspot.styles.position_type = Some("absolute".to_string());
    right_hotspot.styles.right = Some("4%".to_string());
    right_hotspot.styles.top = Some("4%".to_string());
    right_hotspot.styles.width = Some("22%".to_string());
    right_hotspot.styles.height = Some("14%".to_string());
    right_hotspot.styles.z_index = Some("-1".to_string());
    right_hotspot.visuals.background_color = Some("#FFF0C830".to_string());
    right_hotspot.visuals.border_radius = Some("999px".to_string());
    right_hotspot.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("26px".to_string()),
        spread_radius: Some("12px".to_string()),
        color: Some("#FFF1D024".to_string()),
    });

    let mut right_sheen = bui_node("info_panel_right_sheen", BuiNodeType::Node);
    right_sheen
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    right_sheen.styles.position_type = Some("absolute".to_string());
    right_sheen.styles.right = Some("0".to_string());
    right_sheen.styles.top = Some("0".to_string());
    right_sheen.styles.bottom = Some("0".to_string());
    right_sheen.styles.width = Some("18%".to_string());
    right_sheen.styles.z_index = Some("-1".to_string());
    right_sheen.visuals.background_color = Some("#F0D4A218".to_string());

    let mut lower_ember = bui_node("info_panel_lower_ember", BuiNodeType::Node);
    lower_ember
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    lower_ember.styles.position_type = Some("absolute".to_string());
    lower_ember.styles.left = Some("26%".to_string());
    lower_ember.styles.right = Some("4%".to_string());
    lower_ember.styles.bottom = Some("0".to_string());
    lower_ember.styles.height = Some("22%".to_string());
    lower_ember.styles.z_index = Some("-1".to_string());
    lower_ember.visuals.background_color = Some("#7C4F3B22".to_string());

    let mut inner_band = bui_node("info_panel_inner_band", BuiNodeType::Node);
    inner_band
        .custom_tags
        .push("hero-info-panel:decor".to_string());
    inner_band.styles.position_type = Some("absolute".to_string());
    inner_band.styles.left = Some("31%".to_string());
    inner_band.styles.right = Some("10%".to_string());
    inner_band.styles.top = Some("30%".to_string());
    inner_band.styles.height = Some("1px".to_string());
    inner_band.styles.z_index = Some("1".to_string());
    inner_band.visuals.background_color = Some("#FFF0D636".to_string());

    info_panel.children.insert(0, right_hotspot);
    info_panel.children.insert(0, right_sheen);
    info_panel.children.insert(0, lower_ember);
    info_panel.children.insert(0, inner_band);
    info_panel.children.insert(0, mid_warmth);
    info_panel.children.insert(0, top_gloss);
    info_panel.children.insert(0, left_mask_soft);
    info_panel.children.insert(0, left_inner_glow);
    info_panel.children.insert(0, left_cut_3);
    info_panel.children.insert(0, left_cut_2);
    info_panel.children.insert(0, left_cut_1);
}

fn inject_hero_zone_layers(hero_zone: &mut BuiNode) {
    if hero_zone
        .children
        .iter()
        .any(|child| child.id == "hero_zone_backlight")
    {
        return;
    }

    let mut backlight = bui_node("hero_zone_backlight", BuiNodeType::Node);
    backlight.custom_tags.push("hero-zone:decor".to_string());
    backlight.styles.position_type = Some("absolute".to_string());
    backlight.styles.left = Some("8%".to_string());
    backlight.styles.width = Some("54%".to_string());
    backlight.styles.top = Some("6%".to_string());
    backlight.styles.bottom = Some("14%".to_string());
    backlight.styles.z_index = Some("0".to_string());
    backlight.visuals.background_color = Some("#5D6F8A26".to_string());
    backlight.visuals.border_radius = Some("999px".to_string());
    backlight.visuals.box_shadow = Some(BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some("42px".to_string()),
        spread_radius: Some("18px".to_string()),
        color: Some("#536A8F1E".to_string()),
    });

    let mut light_beam = bui_node("hero_zone_light_beam", BuiNodeType::Node);
    light_beam.custom_tags.push("hero-zone:decor".to_string());
    light_beam.styles.position_type = Some("absolute".to_string());
    light_beam.styles.left = Some("22%".to_string());
    light_beam.styles.width = Some("26%".to_string());
    light_beam.styles.top = Some("4%".to_string());
    light_beam.styles.bottom = Some("22%".to_string());
    light_beam.styles.z_index = Some("0".to_string());
    light_beam.visuals.background_color = Some("#E9D3A032".to_string());
    light_beam.visuals.border_radius = Some("999px".to_string());

    let mut edge_shadow = bui_node("hero_zone_edge_shadow", BuiNodeType::Node);
    edge_shadow.custom_tags.push("hero-zone:decor".to_string());
    edge_shadow.styles.position_type = Some("absolute".to_string());
    edge_shadow.styles.left = Some("40%".to_string());
    edge_shadow.styles.right = Some("6%".to_string());
    edge_shadow.styles.top = Some("12%".to_string());
    edge_shadow.styles.bottom = Some("4%".to_string());
    edge_shadow.styles.z_index = Some("0".to_string());
    edge_shadow.visuals.background_color = Some("#2C243050".to_string());
    edge_shadow.visuals.border_radius = Some("120px".to_string());

    hero_zone.children.insert(0, edge_shadow);
    hero_zone.children.insert(0, light_beam);
    hero_zone.children.insert(0, backlight);
}

fn hero_game_ui_base_stats() -> [(&'static str, &'static str, &'static str, &'static str); 5] {
    [
        ("武", "武力", "136.28", "+18"),
        ("统", "统帅", "136.2", "+186"),
        ("智", "智谋", "136.3", "+86"),
        ("速", "速度", "28.66", "+210"),
        ("政", "政务", "206.2", "+186"),
    ]
}

fn hero_game_ui_stat_row(
    index: usize,
    icon: &str,
    label: &str,
    base: &str,
    bonus: &str,
) -> BuiNode {
    let mut row = bui_node(&format!("hero_stat_row_{index}"), BuiNodeType::Node);
    row.custom_tags.push("class:stat-row".to_string());
    row.styles.display = Some("grid".to_string());
    row.styles.grid_template_columns = Some("flex(1) auto auto".to_string());
    row.styles.align_items = Some("center".to_string());
    row.styles.column_gap = Some("10px".to_string());
    row.styles.padding = Some("0 8px".to_string());
    row.styles.min_height = Some("40px".to_string());
    row.visuals.background_color = Some("#6D5A6333".to_string());

    let mut label_node = bui_node(&format!("hero_stat_label_{index}"), BuiNodeType::Node);
    label_node.custom_tags.push("class:stat-label".to_string());
    label_node.styles.display = Some("flex".to_string());
    label_node.styles.align_items = Some("center".to_string());
    label_node.styles.column_gap = Some("11px".to_string());
    label_node.styles.min_width = Some("0".to_string());
    label_node.children.push(text_node(
        &format!("hero_stat_icon_text_{index}"),
        icon,
        22.0,
        "#E9DDC8",
        Some("Palatino.ttc"),
    ));
    label_node.children.push(text_node(
        &format!("hero_stat_label_text_{index}"),
        label,
        24.0,
        "#F0E7D8",
        Some("Hiragino Sans GB.ttc"),
    ));

    let mut base_node = bui_node(&format!("hero_stat_base_{index}"), BuiNodeType::Node);
    base_node.custom_tags.push("class:stat-base".to_string());
    base_node.styles.display = Some("flex".to_string());
    base_node.styles.justify_content = Some("flex-end".to_string());
    base_node.styles.align_items = Some("center".to_string());
    base_node.children.push(text_node(
        &format!("hero_stat_base_text_{index}"),
        base,
        24.0,
        "#F6EBDD",
        Some("Palatino.ttc"),
    ));

    let mut bonus_node = bui_node(&format!("hero_stat_bonus_{index}"), BuiNodeType::Node);
    bonus_node.custom_tags.push("class:stat-bonus".to_string());
    bonus_node.styles.display = Some("flex".to_string());
    bonus_node.styles.justify_content = Some("flex-end".to_string());
    bonus_node.styles.align_items = Some("center".to_string());
    bonus_node.children.push(text_node(
        &format!("hero_stat_bonus_text_{index}"),
        bonus,
        24.0,
        "#B7DD6D",
        Some("Palatino.ttc"),
    ));

    row.children.push(label_node);
    row.children.push(base_node);
    row.children.push(bonus_node);
    row
}

fn find_bui_node_mut<'a>(node: &'a mut BuiNode, id: &str) -> Option<&'a mut BuiNode> {
    if node.id == id {
        return Some(node);
    }

    for child in &mut node.children {
        if let Some(found) = find_bui_node_mut(child, id) {
            return Some(found);
        }
    }

    None
}

fn ensure_text_icon_child(root: &mut BuiNode, id: &str) {
    let Some(node) = find_bui_node_mut(root, id) else {
        return;
    };
    let Some(spec) = semantic_svg_fallback_spec(node) else {
        return;
    };
    if node.children.iter().any(|child| !is_decorative_icon_helper_node(child)) {
        return;
    }
    node.children
        .retain(|child| !child.custom_tags.iter().any(|tag| tag == "svg:fallback"));
    let mut icon_node = text_node(
        &format!("{id}_icon_text"),
        spec.icon,
        spec.font_size.unwrap_or(20.0),
        spec.color,
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(text_shadow) = spec.text_shadow()
        && let Some(text_config) = icon_node.text_config.as_mut()
    {
        text_config.text_shadow = Some(text_shadow);
    }
    node.children.push(icon_node);
}

fn is_decorative_icon_helper_node(node: &BuiNode) -> bool {
    node.custom_tags.iter().any(|tag| {
        tag == "pseudo:before"
            || tag == "pseudo:after"
            || tag == "class:cooldown"
            || tag == "svg:fallback"
    })
}

fn is_svg_tag(tag: &str) -> bool {
    matches!(
        tag,
        "svg" | "path" | "circle" | "ellipse" | "rect" | "line" | "polyline" | "polygon" | "g"
    )
}

fn svg_fallback_text_node(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
    index: usize,
) -> Option<BuiNode> {
    let icon = svg_fallback_icon(parent, svg_node)?;
    let fallback_style = svg_fallback_style(parent, icon);
    let mut text_node = text_node(
        &format!("{}_svg_fallback_{}", parent.id, index),
        icon,
        fallback_style
            .font_size
            .unwrap_or_else(|| svg_fallback_font_size(parent, svg_node, stylesheet)),
        fallback_style.color,
        Some("Hiragino Sans GB.ttc"),
    );
    if let Some(text_shadow) = fallback_style.text_shadow {
        if let Some(text_config) = text_node.text_config.as_mut() {
            text_config.text_shadow = Some(text_shadow);
        }
    }
    text_node.custom_tags.push("svg:fallback".to_string());
    Some(text_node)
}

struct SvgFallbackStyle {
    font_size: Option<f32>,
    color: &'static str,
    text_shadow: Option<BuiTextShadowConfig>,
}

#[derive(Clone, Copy)]
struct SemanticSvgFallbackSpec {
    icon: &'static str,
    font_size: Option<f32>,
    color: &'static str,
    shadow_color: Option<&'static str>,
    shadow_offset_y: f32,
}

impl SemanticSvgFallbackSpec {
    fn text_shadow(self) -> Option<BuiTextShadowConfig> {
        self.shadow_color.map(|color| BuiTextShadowConfig {
            offset_x: Some(0.0),
            offset_y: Some(self.shadow_offset_y),
            color: Some(color.to_string()),
        })
    }
}

fn svg_fallback_style(parent: &BuiNode, icon: &str) -> SvgFallbackStyle {
    if let Some(spec) = semantic_svg_fallback_spec(parent) {
        return SvgFallbackStyle {
            font_size: spec.font_size,
            color: spec.color,
            text_shadow: spec.text_shadow(),
        };
    }

    let has_class = |class_name: &str| {
        parent
            .custom_tags
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    if has_class("round-button") {
        return SvgFallbackStyle {
            font_size: Some(28.0),
            color: "#F5C85A",
            text_shadow: Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(2.0),
                color: Some("#5A3F18A0".to_string()),
            }),
        };
    }

    if has_class("bar-icon") {
        return SvgFallbackStyle {
            font_size: Some(22.0),
            color: "#F5E6B8",
            text_shadow: Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(2.0),
                color: Some("#3D2A1A8F".to_string()),
            }),
        };
    }

    if has_class("star") || parent.id == "stars" {
        return SvgFallbackStyle {
            font_size: Some(42.0),
            color: "#F5C742",
            text_shadow: Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(3.0),
                color: Some("#5A341CA0".to_string()),
            }),
        };
    }

    if has_class("stat-label") {
        return SvgFallbackStyle {
            font_size: Some(22.0),
            color: "#E9DDC8",
            text_shadow: None,
        };
    }

    let (font_size, color) = match icon {
        "⚡" => (Some(24.0), "#F6ECDD"),
        "♛" | "▤" => (Some(22.0), "#F6ECDD"),
        "➶" => (Some(24.0), "#F3E3C6"),
        "⛨" | "⟡" | "♞" | "◎" => (Some(22.0), "#F3E3C6"),
        "★" => (Some(22.0), "#F5E6B8"),
        "✦" => (Some(22.0), "#F5E6B8"),
        _ => (None, "#F4E7CA"),
    };

    SvgFallbackStyle {
        font_size,
        color,
        text_shadow: None,
    }
}

fn svg_fallback_icon(parent: &BuiNode, svg_node: roxmltree::Node<'_, '_>) -> Option<&'static str> {
    if let Some(spec) = semantic_svg_fallback_spec(parent) {
        return Some(spec.icon);
    }

    if let Some(icon) = svg_shape_fallback_icon(parent, svg_node) {
        return Some(icon);
    }

    let signature = svg_signature(svg_node);

    if parent
        .custom_tags
        .iter()
        .any(|tag| tag == "class:round-button")
        || signature.contains("M38 13 19 32l19 19")
    {
        return Some("←");
    }
    if signature.contains("M16 2.2 20.2 11") || signature.contains("M16 2 20 11l10 1") {
        return Some("★");
    }
    if signature.contains("M9 4h8l2 7 6 2") {
        return Some("✦");
    }
    if signature.contains("M20 4 7 20h8l-3 12") {
        return Some("⚡");
    }
    if signature.contains("M18 4c5 0 9 4 9 9v6l4 6") {
        return Some("♛");
    }
    if signature.contains("M8 6h17c2 0 4 2 4 4v20") {
        return Some("▤");
    }
    if signature.contains("M28 4c-10 7-14 18-9 32") {
        return Some("➶");
    }
    if signature.contains("M20 4 32 9v9c0 8-5 14-12 18") {
        return Some("⛨");
    }
    if signature.contains("M28 3 36 12 18 29 11 22") {
        return Some("⟡");
    }
    if signature.contains("M5 25c7-12 15-13 28-8") {
        return Some("♞");
    }
    if signature.contains("circle:20:20:14") || signature.contains("M16 5v22M5 16h22") {
        return Some("◎");
    }
    if signature.contains("M11 8a5 5 0 0 1 10 0") {
        return Some("智");
    }
    if signature.contains("M18 2 8 18h8l-2 12") {
        return Some("速");
    }
    if signature.contains("M4 13 16 5l12 8H4") {
        return Some("政");
    }
    if signature.contains("M7 25 25 7M20 5l7 7") {
        return Some("武");
    }
    if signature.contains("M5 19c5-11 16-13 22-9") {
        return Some("统");
    }
    if signature.contains("M16 4 27 9v8c0 7-4.5 12-11 15") {
        return Some("守");
    }

    None
}

#[derive(Default)]
struct SvgShapeProfile {
    path_count: usize,
    circle_count: usize,
    filled_path_count: usize,
    stroked_path_count: usize,
    round_stroke_path_count: usize,
    horizontal_path_hints: usize,
    vertical_path_hints: usize,
    curved_path_hints: usize,
}

fn svg_shape_fallback_icon(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
) -> Option<&'static str> {
    let has_class = |class_name: &str| {
        parent
            .custom_tags
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    let profile = svg_shape_profile(svg_node);

    if has_class("equip-slot") {
        if profile.circle_count >= 1 && profile.stroked_path_count >= 2 {
            return Some("◎");
        }
        if profile.path_count >= 3
            && profile.stroked_path_count == profile.path_count
            && profile.round_stroke_path_count >= 1
        {
            return Some("➶");
        }
        if profile.path_count >= 2
            && profile.stroked_path_count == profile.path_count
            && profile.filled_path_count == 0
            && profile.horizontal_path_hints >= 1
        {
            return Some("⛨");
        }
        if profile.filled_path_count >= 1 && profile.round_stroke_path_count >= 1 {
            return Some("⟡");
        }
        if profile.filled_path_count >= 1 && profile.stroked_path_count >= 1 {
            return Some("♞");
        }
    }

    if has_class("skill-button") && profile.path_count == 1 && profile.filled_path_count == 1 {
        let path_data = svg_first_path_data(svg_node)?.to_ascii_lowercase();
        if path_data.contains('h') && path_data.contains('v') {
            return Some("▤");
        }
        if path_data.contains('c') {
            return Some("♛");
        }
        return Some("⚡");
    }

    if has_class("bar-icon") && profile.path_count == 1 && profile.filled_path_count == 1 {
        let path_data = svg_first_path_data(svg_node)?.to_ascii_lowercase();
        if path_data.contains('h') && profile.vertical_path_hints >= 1 {
            return Some("✦");
        }
        return Some("★");
    }

    None
}

fn svg_shape_profile(svg_node: roxmltree::Node<'_, '_>) -> SvgShapeProfile {
    let mut profile = SvgShapeProfile::default();

    for node in svg_node.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "path" => {
                profile.path_count += 1;
                if svg_path_has_fill(node) {
                    profile.filled_path_count += 1;
                }
                if node.attribute("stroke").is_some() {
                    profile.stroked_path_count += 1;
                }
                if node.attribute("stroke-linecap") == Some("round") {
                    profile.round_stroke_path_count += 1;
                }
                if let Some(data) = node.attribute("d") {
                    let data = data.to_ascii_lowercase();
                    if data.contains('h') {
                        profile.horizontal_path_hints += 1;
                    }
                    if data.contains('v') {
                        profile.vertical_path_hints += 1;
                    }
                    if data.contains('c') || data.contains('q') || data.contains('a') {
                        profile.curved_path_hints += 1;
                    }
                }
            }
            "circle" => {
                profile.circle_count += 1;
            }
            _ => {}
        }
    }

    profile
}

fn svg_path_has_fill(node: roxmltree::Node<'_, '_>) -> bool {
    matches!(node.attribute("fill"), Some(fill) if normalize_token(fill) != "none")
}

fn svg_first_path_data(svg_node: roxmltree::Node<'_, '_>) -> Option<String> {
    svg_node
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "path")
        .and_then(|node| node.attribute("d"))
        .map(ToString::to_string)
}

fn semantic_svg_fallback_spec(parent: &BuiNode) -> Option<SemanticSvgFallbackSpec> {
    if let Some(spec) = semantic_svg_fallback_spec_from_tags(parent) {
        return Some(spec);
    }

    if parent.id == "backbutton" {
        return Some(SemanticSvgFallbackSpec {
            icon: "←",
            font_size: Some(28.0),
            color: "#F5C85A",
            shadow_color: Some("#5A3F18A0"),
            shadow_offset_y: 2.0,
        });
    }

    if let Some(spec) = indexed_semantic_svg_fallback_spec(
        &parent.id,
        "bar_icon",
        &["★", "✦"],
        22.0,
        "#F5E6B8",
        Some("#3D2A1A8F"),
        2.0,
    ) {
        return Some(spec);
    }

    if let Some(spec) = indexed_semantic_svg_fallback_spec(
        &parent.id,
        "skill_button",
        &["⚡", "♛", "▤"],
        22.0,
        "#F6ECDD",
        None,
        0.0,
    ) {
        return Some(if parent.id == "skill_button" {
            SemanticSvgFallbackSpec {
                font_size: Some(24.0),
                ..spec
            }
        } else {
            spec
        });
    }

    if let Some(spec) = indexed_semantic_svg_fallback_spec(
        &parent.id,
        "equip_slot",
        &["➶", "⛨", "⟡", "♞", "◎"],
        22.0,
        "#F3E3C6",
        None,
        0.0,
    ) {
        return Some(if parent.id == "equip_slot" {
            SemanticSvgFallbackSpec {
                font_size: Some(24.0),
                ..spec
            }
        } else {
            spec
        });
    }

    let has_class = |class_name: &str| {
        parent
            .custom_tags
            .iter()
            .any(|tag| tag == &format!("class:{class_name}"))
    };

    if has_class("round-button") {
        return Some(SemanticSvgFallbackSpec {
            icon: "←",
            font_size: Some(28.0),
            color: "#F5C85A",
            shadow_color: Some("#5A3F18A0"),
            shadow_offset_y: 2.0,
        });
    }
    if has_class("bar-icon") {
        return Some(SemanticSvgFallbackSpec {
            icon: "★",
            font_size: Some(22.0),
            color: "#F5E6B8",
            shadow_color: Some("#3D2A1A8F"),
            shadow_offset_y: 2.0,
        });
    }
    if has_class("star") || parent.id == "stars" {
        return Some(SemanticSvgFallbackSpec {
            icon: "★",
            font_size: Some(42.0),
            color: "#F5C742",
            shadow_color: Some("#5A341CA0"),
            shadow_offset_y: 3.0,
        });
    }

    None
}

fn semantic_svg_fallback_spec_from_tags(parent: &BuiNode) -> Option<SemanticSvgFallbackSpec> {
    let find_tag_value = |prefix: &str| {
        parent
            .custom_tags
            .iter()
            .find_map(|tag| tag.strip_prefix(prefix))
    };

    if let Some(skill) = find_tag_value("data-skill:") {
        return semantic_skill_icon_spec(skill);
    }
    if let Some(equip) = find_tag_value("data-equip:") {
        return semantic_equip_icon_spec(equip);
    }
    if let Some(label) = find_tag_value("aria-label:") {
        if let Some(spec) = semantic_skill_icon_spec(label) {
            return Some(spec);
        }
        if let Some(spec) = semantic_equip_icon_spec(label) {
            return Some(spec);
        }
        if let Some(spec) = semantic_aria_icon_spec(label) {
            return Some(spec);
        }
    }

    None
}

fn semantic_skill_icon_spec(skill: &str) -> Option<SemanticSvgFallbackSpec> {
    let icon = if skill.contains("震击") || skill.contains("雷") || skill.contains("击") {
        "⚡"
    } else if skill.contains("号令") || skill.contains("军团") {
        "♛"
    } else if skill.contains("战策") || skill.contains("圣卷") || skill.contains("卷") {
        "▤"
    } else {
        return None;
    };

    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(22.0),
        color: "#F6ECDD",
        shadow_color: None,
        shadow_offset_y: 0.0,
    })
}

fn semantic_equip_icon_spec(equip: &str) -> Option<SemanticSvgFallbackSpec> {
    let icon = if equip.contains("弓") {
        "➶"
    } else if equip.contains("盾") {
        "⛨"
    } else if equip.contains("矛") {
        "⟡"
    } else if equip.contains("坐骑") || equip.contains("战马") || equip.contains("骑") {
        "♞"
    } else if equip.contains("徽章") || equip.contains("鹰眼") || equip.contains("徽") {
        "◎"
    } else {
        return None;
    };

    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(22.0),
        color: "#F3E3C6",
        shadow_color: None,
        shadow_offset_y: 0.0,
    })
}

fn semantic_aria_icon_spec(label: &str) -> Option<SemanticSvgFallbackSpec> {
    let icon = if label.contains("返回") {
        "←"
    } else {
        return None;
    };

    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(28.0),
        color: "#F5C85A",
        shadow_color: Some("#5A3F18A0"),
        shadow_offset_y: 2.0,
    })
}

fn indexed_semantic_svg_fallback_spec(
    id: &str,
    base_id: &str,
    icons: &[&'static str],
    font_size: f32,
    color: &'static str,
    shadow_color: Option<&'static str>,
    shadow_offset_y: f32,
) -> Option<SemanticSvgFallbackSpec> {
    let index = if id == base_id {
        0
    } else if let Some(suffix) = id.strip_prefix(&format!("{base_id}_")) {
        suffix.parse::<usize>().ok()?.checked_sub(1)?
    } else {
        return None;
    };

    let icon = *icons.get(index)?;
    Some(SemanticSvgFallbackSpec {
        icon,
        font_size: Some(font_size),
        color,
        shadow_color,
        shadow_offset_y,
    })
}

fn svg_fallback_font_size(
    parent: &BuiNode,
    svg_node: roxmltree::Node<'_, '_>,
    stylesheet: &OpenDesignStylesheet,
) -> f32 {
    let mut probe = text_node("svg_fallback_probe", "•", 16.0, "#FFFFFF", None);
    apply_inherited_text_styles(stylesheet, &mut probe, svg_node);
    apply_opendesign_styles(stylesheet, &mut probe, svg_node);
    if let Some(text_config) = probe.text_config.as_ref() {
        return text_config.font_size.clamp(16.0, 28.0);
    }
    if parent
        .custom_tags
        .iter()
        .any(|tag| tag == "class:star" || tag == "class:bar-icon")
    {
        return 22.0;
    }
    20.0
}

fn svg_signature(svg_node: roxmltree::Node<'_, '_>) -> String {
    let mut parts = Vec::new();
    for node in svg_node.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "path" => {
                if let Some(value) = node.attribute("d") {
                    parts.push(value.replace(char::is_whitespace, " "));
                }
            }
            "circle" => {
                let cx = node.attribute("cx").unwrap_or_default();
                let cy = node.attribute("cy").unwrap_or_default();
                let r = node.attribute("r").unwrap_or_default();
                parts.push(format!("circle:{cx}:{cy}:{r}"));
            }
            _ => {}
        }
    }
    parts.join("|")
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
        self.resolve_value_with_variables(value, &HashMap::new())
    }

    fn resolve_value_with_variables(
        &self,
        value: &str,
        variables: &HashMap<String, String>,
    ) -> String {
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
            let replacement = variables
                .get(variable_name)
                .cloned()
                .or_else(|| self.variables.get(variable_name).cloned())
                .unwrap_or_default();
            resolved.replace_range(start..=end, &replacement);
        }
        resolved.trim().to_string()
    }

    fn custom_properties_for_node(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        let mut ancestors = dom_node
            .ancestors()
            .filter(|node| node.is_element())
            .collect::<Vec<_>>();
        ancestors.reverse();

        for node in ancestors {
            for (name, value) in self.matching_declarations(node) {
                if !name.starts_with("--") {
                    continue;
                }
                let value = self.resolve_value_with_variables(value, &variables);
                variables.insert(name.clone(), value);
            }

            if let Some(inline_style) = node.attribute("style") {
                for (name, value) in css_declarations(inline_style) {
                    if !name.starts_with("--") {
                        continue;
                    }
                    let value = self.resolve_value_with_variables(&value, &variables);
                    variables.insert(name, value);
                }
            }
        }

        variables
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
    let custom_properties = stylesheet.custom_properties_for_node(dom_node);

    for (name, value) in stylesheet.matching_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        apply_opendesign_declaration(bui_node, name, &value);
    }

    for (state, (name, value)) in stylesheet.matching_state_declarations(dom_node) {
        let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
        apply_opendesign_state_declaration(bui_node, state, name, &value);
    }

    if let Some(inline_style) = dom_node.attribute("style") {
        for (name, value) in css_declarations(inline_style) {
            let value = stylesheet.resolve_value_with_variables(&value, &custom_properties);
            apply_opendesign_declaration(bui_node, &name, &value);
        }
    }
}

fn apply_inherited_text_styles(
    stylesheet: &OpenDesignStylesheet,
    bui_node: &mut BuiNode,
    dom_node: roxmltree::Node<'_, '_>,
) {
    if !matches!(bui_node.node_type, BuiNodeType::Text) {
        return;
    }

    let mut ancestors = dom_node
        .ancestors()
        .filter(|node| node.is_element())
        .collect::<Vec<_>>();
    ancestors.reverse();

    for ancestor in ancestors {
        let custom_properties = stylesheet.custom_properties_for_node(ancestor);
        for (name, value) in stylesheet.matching_declarations(ancestor) {
            if !is_inheritable_text_property(name) {
                continue;
            }
            let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
            apply_opendesign_declaration(bui_node, name, &value);
        }

        for (state, (name, value)) in stylesheet.matching_state_declarations(ancestor) {
            if !is_inheritable_text_property(name) {
                continue;
            }
            let value = stylesheet.resolve_value_with_variables(value, &custom_properties);
            apply_opendesign_state_declaration(bui_node, state, name, &value);
        }

        if let Some(inline_style) = ancestor.attribute("style") {
            for (name, value) in css_declarations(inline_style) {
                if !is_inheritable_text_property(&name) {
                    continue;
                }
                let value = stylesheet.resolve_value_with_variables(&value, &custom_properties);
                apply_opendesign_declaration(bui_node, &name, &value);
            }
        }
    }
}

fn is_inheritable_text_property(name: &str) -> bool {
    matches!(
        name,
        "color"
            | "font-size"
            | "font-family"
            | "font-weight"
            | "line-height"
            | "letter-spacing"
            | "text-align"
            | "text-shadow"
            | "white-space"
            | "opacity"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum CssPropertySupportLevel {
    P0,
    P1,
    P2,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum CssFallbackStrategy {
    Native,
    HelperLayer,
    ColorApproximation,
    SemanticFallback,
    None,
}

#[allow(dead_code)]
struct CssPropertyInfo {
    level: CssPropertySupportLevel,
    strategy: CssFallbackStrategy,
    helper_tag: Option<&'static str>,
}

#[allow(dead_code)]
fn css_property_info(name: &str) -> CssPropertyInfo {
    match name {
        "display" | "position" | "width" | "height" | "min-width" | "min-height"
        | "max-width" | "max-height" | "inset" | "left" | "right" | "top" | "bottom"
        | "margin" | "margin-left" | "margin-right" | "margin-top" | "margin-bottom"
        | "padding" | "padding-left" | "padding-right" | "padding-top" | "padding-bottom"
        | "padding-inline" | "padding-block" | "gap" | "row-gap" | "column-gap"
        | "flex-direction" | "flex-wrap" | "flex-grow" | "flex-shrink" | "flex-basis"
        | "align-items" | "align-self" | "align-content" | "justify-content"
        | "justify-items" | "justify-self" | "place-items" | "overflow" | "overflow-x"
        | "overflow-y" | "grid-template-columns" | "grid-template-rows"
        | "aspect-ratio" | "z-index" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "background-color" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "background" => CssPropertyInfo {
            level: CssPropertySupportLevel::P1,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-gradient-overlay"),
        },
        "background-image" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "background-size" | "background-position" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "border" | "border-top" | "border-bottom" | "border-left" | "border-right"
        | "border-color" | "border-top-color" | "border-bottom-color"
        | "border-left-color" | "border-right-color" | "border-width"
        | "border-top-width" | "border-bottom-width" | "border-left-width"
        | "border-right-width" | "border-radius" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-edge-border"),
        },
        "color" | "font-size" | "font-family" | "font-weight" | "line-height"
        | "letter-spacing" | "text-align" | "text-shadow" | "white-space"
        | "opacity" => CssPropertyInfo {
            level: CssPropertySupportLevel::P0,
            strategy: CssFallbackStrategy::Native,
            helper_tag: None,
        },
        "box-shadow" => CssPropertyInfo {
            level: CssPropertySupportLevel::P1,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-box-shadow-layer"),
        },
        "filter" => CssPropertyInfo {
            level: CssPropertySupportLevel::P1,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-filter-drop-shadow"),
        },
        "mask-image" => CssPropertyInfo {
            level: CssPropertySupportLevel::P2,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-mask-fade"),
        },
        "clip-path" => CssPropertyInfo {
            level: CssPropertySupportLevel::P2,
            strategy: CssFallbackStrategy::HelperLayer,
            helper_tag: Some("css-clip-contour"),
        },
        "mix-blend-mode" => CssPropertyInfo {
            level: CssPropertySupportLevel::P2,
            strategy: CssFallbackStrategy::ColorApproximation,
            helper_tag: None,
        },
        "cursor" | "pointer-events" | "transition" | "content" | "isolation"
        | "-webkit-tap-highlight-color" => CssPropertyInfo {
            level: CssPropertySupportLevel::Unsupported,
            strategy: CssFallbackStrategy::None,
            helper_tag: None,
        },
        _ => CssPropertyInfo {
            level: CssPropertySupportLevel::Unsupported,
            strategy: CssFallbackStrategy::None,
            helper_tag: None,
        },
    }
}

#[allow(dead_code)]
struct CssEffectFallbackEntry {
    css_property: &'static str,
    helper_tag: &'static str,
    fallback_fn: &'static str,
    description: &'static str,
}

#[allow(dead_code)]
fn css_effect_fallback_registry() -> Vec<CssEffectFallbackEntry> {
    vec![
        CssEffectFallbackEntry {
            css_property: "background (gradient)",
            helper_tag: "css-gradient-overlay",
            fallback_fn: "apply_simple_gradient_overlays",
            description: "Gradient decomposed into positioned solid-color overlay bands",
        },
        CssEffectFallbackEntry {
            css_property: "box-shadow (multi-layer)",
            helper_tag: "css-box-shadow-layer",
            fallback_fn: "apply_box_shadow_fallback",
            description: "Primary shadow to node box_shadow; secondary shadows to absolute-positioned helper children",
        },
        CssEffectFallbackEntry {
            css_property: "filter: drop-shadow(...)",
            helper_tag: "css-filter-drop-shadow",
            fallback_fn: "css_filter_drop_shadows + push_box_shadow_layer",
            description: "Each drop-shadow becomes a box-shadow layer child; on text nodes becomes text_shadow",
        },
        CssEffectFallbackEntry {
            css_property: "filter: blur(...)",
            helper_tag: "css-filter-blur",
            fallback_fn: "apply_filter_blur_fallback",
            description: "Approximated as a zero-offset box-shadow with spread; on text nodes becomes low-alpha text_shadow",
        },
        CssEffectFallbackEntry {
            css_property: "filter: brightness/contrast/saturate",
            helper_tag: "N/A",
            fallback_fn: "css_filter_color_adjustment + apply_filter_color_adjustment",
            description: "Applied as direct color channel adjustment to background/border/text colors",
        },
        CssEffectFallbackEntry {
            css_property: "mask-image: linear-gradient(...)",
            helper_tag: "css-mask-fade",
            fallback_fn: "apply_mask_image_fallback",
            description: "Three gradient-fade child layers at decreasing alpha (62/34/16%) approximating edge fade",
        },
        CssEffectFallbackEntry {
            css_property: "clip-path: polygon(...)",
            helper_tag: "css-clip-contour",
            fallback_fn: "apply_clip_path_fallback",
            description: "Fill, contour and accent child nodes approximating clipped shape; bounded inner fill extracted",
        },
        CssEffectFallbackEntry {
            css_property: "mix-blend-mode: multiply",
            helper_tag: "N/A",
            fallback_fn: "apply_mix_blend_mode_fallback",
            description: "Darkens color channels of gradient overlays and helper shadow layers to approximate multiply",
        },
        CssEffectFallbackEntry {
            css_property: "inline SVG icon",
            helper_tag: "svg:fallback",
            fallback_fn: "svg_shape_fallback_profile + semantic_svg_fallback",
            description: "SVG paths replaced with Unicode text characters via semantic matching and shape profile recognition",
        },
        CssEffectFallbackEntry {
            css_property: "::before / ::after",
            helper_tag: "pseudo:before / pseudo:after",
            fallback_fn: "apply_opendesign_styles (pseudo path)",
            description: "Pseudo-element declarations applied as child nodes with pseudo markers",
        },
        CssEffectFallbackEntry {
            css_property: "per-edge border",
            helper_tag: "css-edge-border:{edge}",
            fallback_fn: "apply_css_edge_border + ensure_edge_border_node",
            description: "Individual edge borders (top/right/bottom/left) created as absolute-positioned child nodes",
        },
    ]
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
    if matches!(bui_node.node_type, BuiNodeType::Text)
        && !matches!(name, "color" | "opacity" | "filter")
    {
        return;
    }

    let mut needs_normal_scale_reset = false;
    let base_background_color = bui_node.visuals.background_color.clone();
    let base_border_color = bui_node.visuals.border_color.clone();
    let base_text_color = bui_node
        .text_config
        .as_ref()
        .map(|text_config| text_config.font_color.clone())
        .or_else(|| direct_text_child_font_color(bui_node).map(ToString::to_string));
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
            "opacity" => {
                if let Some(opacity) = value.parse::<f32>().ok() {
                    apply_state_opacity_fallback(
                        state_visual,
                        opacity,
                        base_background_color.as_deref(),
                        base_border_color.as_deref(),
                        base_text_color.as_deref(),
                    );
                }
            }
            "transform" => {
                if let Some(scale) = css_transform_scale(&value) {
                    state_visual.styles.ui_scale = Some(scale);
                    needs_normal_scale_reset = true;
                }
            }
            "filter" => {
                if let Some(adjustment) = css_filter_color_adjustment(&value) {
                    apply_state_filter_color_adjustment(
                        state_visual,
                        adjustment,
                        base_background_color.as_deref(),
                        base_border_color.as_deref(),
                        base_text_color.as_deref(),
                    );
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
    if matches!(bui_node.node_type, BuiNodeType::Text)
        && !matches!(
            name,
            "color"
                | "font-size"
                | "font-family"
                | "font-weight"
                | "line-height"
                | "letter-spacing"
                | "text-align"
                | "text-shadow"
                | "white-space"
                | "opacity"
        )
    {
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
        "inset" => {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            match parts.as_slice() {
                [all] => {
                    set_simple_css_val(&mut bui_node.styles.top, all);
                    set_simple_css_val(&mut bui_node.styles.right, all);
                    set_simple_css_val(&mut bui_node.styles.bottom, all);
                    set_simple_css_val(&mut bui_node.styles.left, all);
                }
                [vertical, horizontal] => {
                    set_simple_css_val(&mut bui_node.styles.top, vertical);
                    set_simple_css_val(&mut bui_node.styles.bottom, vertical);
                    set_simple_css_val(&mut bui_node.styles.left, horizontal);
                    set_simple_css_val(&mut bui_node.styles.right, horizontal);
                }
                [top, horizontal, bottom] => {
                    set_simple_css_val(&mut bui_node.styles.top, top);
                    set_simple_css_val(&mut bui_node.styles.left, horizontal);
                    set_simple_css_val(&mut bui_node.styles.right, horizontal);
                    set_simple_css_val(&mut bui_node.styles.bottom, bottom);
                }
                [top, right, bottom, left] => {
                    set_simple_css_val(&mut bui_node.styles.top, top);
                    set_simple_css_val(&mut bui_node.styles.right, right);
                    set_simple_css_val(&mut bui_node.styles.bottom, bottom);
                    set_simple_css_val(&mut bui_node.styles.left, left);
                }
                _ => {}
            }
        }
        "left" => set_simple_css_val(&mut bui_node.styles.left, &value),
        "right" => set_simple_css_val(&mut bui_node.styles.right, &value),
        "top" => set_simple_css_val(&mut bui_node.styles.top, &value),
        "bottom" => set_simple_css_val(&mut bui_node.styles.bottom, &value),
        "margin" => set_css_rect(&mut bui_node.styles.margin, &value),
        "margin-left" => set_simple_css_val(&mut bui_node.styles.margin_left, &value),
        "margin-right" => set_simple_css_val(&mut bui_node.styles.margin_right, &value),
        "margin-top" => set_simple_css_val(&mut bui_node.styles.margin_top, &value),
        "margin-bottom" => set_simple_css_val(&mut bui_node.styles.margin_bottom, &value),
        "padding" => set_css_rect(&mut bui_node.styles.padding, &value),
        "padding-left" => set_simple_css_val(&mut bui_node.styles.padding_left, &value),
        "padding-right" => set_simple_css_val(&mut bui_node.styles.padding_right, &value),
        "padding-top" => set_simple_css_val(&mut bui_node.styles.padding_top, &value),
        "padding-bottom" => set_simple_css_val(&mut bui_node.styles.padding_bottom, &value),
        "padding-inline" => {
            set_simple_css_val(&mut bui_node.styles.padding_left, &value);
            set_simple_css_val(&mut bui_node.styles.padding_right, &value);
        }
        "padding-block" => {
            set_simple_css_val(&mut bui_node.styles.padding_top, &value);
            set_simple_css_val(&mut bui_node.styles.padding_bottom, &value);
        }
        "gap" => {
            if let Some(size) = css_first_size(&value) {
                bui_node.styles.row_gap = Some(size.clone());
                bui_node.styles.column_gap = Some(size);
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
        "border-top" => apply_css_edge_border(bui_node, "top", &value),
        "border-bottom" => apply_css_edge_border(bui_node, "bottom", &value),
        "border-left" => apply_css_edge_border(bui_node, "left", &value),
        "border-right" => apply_css_edge_border(bui_node, "right", &value),
        "border-color" => {
            if let Some(color) = css_color(&value) {
                bui_node.visuals.border_color = Some(color);
            }
        }
        "border-top-color" => apply_css_edge_border_color(bui_node, "top", &value),
        "border-bottom-color" => apply_css_edge_border_color(bui_node, "bottom", &value),
        "border-left-color" => apply_css_edge_border_color(bui_node, "left", &value),
        "border-right-color" => apply_css_edge_border_color(bui_node, "right", &value),
        "border-top-width" => apply_css_edge_border_width(bui_node, "top", &value),
        "border-bottom-width" => apply_css_edge_border_width(bui_node, "bottom", &value),
        "border-left-width" => apply_css_edge_border_width(bui_node, "left", &value),
        "border-right-width" => apply_css_edge_border_width(bui_node, "right", &value),
        "box-shadow" => apply_box_shadow_fallback(bui_node, &value),
        "background-image" => {
            if let Some(texture_path) = css_background_image_url(&value) {
                bui_node.image_config = Some(BuiImageConfig {
                    texture_path,
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    atlas: None,
                    slicer: None,
                    flip_x: false,
                    flip_y: false,
                });
            }
        }
        "background" | "background-color" => {
            if let Some(color) = css_color(&value) {
                bui_node.visuals.background_color = Some(color);
            }
            if name == "background" {
                apply_simple_gradient_overlays(bui_node, &value);
            }
            if let Some(texture_path) = css_background_image_url(&value) {
                bui_node.image_config = Some(BuiImageConfig {
                    texture_path,
                    image_mode: Some("stretch".to_string()),
                    background_size: None,
                    background_position: None,
                    atlas: None,
                    slicer: None,
                    flip_x: false,
                    flip_y: false,
                });
            }
        }
        "background-size" => {
            if let Some(image_config) = &mut bui_node.image_config {
                image_config.background_size = Some(value);
            }
        }
        "background-position" => {
            if let Some(image_config) = &mut bui_node.image_config {
                image_config.background_position = Some(value);
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
                let mapped = css_font_family_to_path(&value);
                text_config.font_path =
                    Some(adjust_font_path_for_content(&mapped, &text_config.content));
            }
        }
        "font-weight" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(font_weight) = css_font_weight(&value)
            {
                text_config.font_weight = Some(font_weight);
            }
        }
        "line-height" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(line_height) = css_line_height(&value)
            {
                text_config.line_height = Some(line_height);
            }
        }
        "letter-spacing" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(letter_spacing) = css_letter_spacing(&value)
            {
                text_config.letter_spacing = Some(letter_spacing);
            }
        }
        "text-align" => {
            if let Some(text_config) = &mut bui_node.text_config
                && css_text_align(&value).is_some()
            {
                text_config.text_align = Some(value);
            }
        }
        "white-space" => {
            if let Some(text_config) = &mut bui_node.text_config {
                apply_css_white_space(text_config, &value);
            }
        }
        "aspect-ratio" => {
            if let Some(aspect_ratio) = css_aspect_ratio(&value) {
                bui_node.styles.aspect_ratio = Some(aspect_ratio);
            }
        }
        "text-shadow" => {
            if let Some(text_config) = &mut bui_node.text_config
                && let Some(text_shadow) = css_text_shadow(&value)
            {
                text_config.text_shadow = Some(text_shadow);
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
        "z-index" => {
            if let Ok(parsed) = value.parse::<i32>() {
                bui_node.styles.z_index = Some(parsed.to_string());
            }
        }
        "filter" => {
            let drop_shadows = css_filter_drop_shadows(&value);
            if let Some(text_config) = &mut bui_node.text_config {
                if text_config.text_shadow.is_none()
                    && let Some(drop_shadow) = drop_shadows.first()
                {
                    text_config.text_shadow = Some(BuiTextShadowConfig {
                        offset_x: drop_shadow
                            .offset_x
                            .as_deref()
                            .and_then(css_filter_shadow_length),
                        offset_y: drop_shadow
                            .offset_y
                            .as_deref()
                            .and_then(css_filter_shadow_length),
                        color: drop_shadow.color.clone(),
                    });
                }
            } else {
                bui_node.children.retain(|child| {
                    !child
                        .custom_tags
                        .iter()
                        .any(|tag| tag == "css-filter-drop-shadow")
                });
                for (index, drop_shadow) in drop_shadows.into_iter().enumerate() {
                    push_box_shadow_layer(
                        bui_node,
                        drop_shadow,
                        "css-filter-drop-shadow",
                        &format!("filter_drop_shadow_{}", index + 1),
                    );
                }
            }
            if let Some(blur_radius) = css_filter_blur_radius(&value) {
                apply_filter_blur_fallback(bui_node, blur_radius);
            }
            if let Some(adjustment) = css_filter_color_adjustment(&value) {
                apply_filter_color_adjustment(bui_node, adjustment);
            }
        }
        "mask-image" => apply_mask_image_fallback(bui_node, &value),
        "mix-blend-mode" => apply_mix_blend_mode_fallback(bui_node, &value),
        "clip-path" => apply_clip_path_fallback(bui_node, &value),
        "cursor" | "pointer-events" | "transition" | "content" | "isolation" | "-webkit-tap-highlight-color" => {}
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
    let normalized = css_size_tokens(value)
        .into_iter()
        .filter_map(|part| {
            if let Some(size) = css_eval_length_function(&part) {
                Some(size)
            } else if is_simple_css_size(&part) {
                Some(part)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if !normalized.is_empty() {
        *target = Some(normalized);
    }
}

fn css_size_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for character in value.chars() {
        match character {
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            character if character.is_ascii_whitespace() && depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
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
    css_size_tokens(value)
        .into_iter()
        .find_map(|part| css_length_to_bui_val(&part))
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
        _ => {
            let tracks = split_grid_track_tokens(value)?;
            let mut converted = Vec::new();

            for track in tracks {
                converted.push(css_grid_track_token_to_bui(&track)?);
            }

            Some(converted.join(" "))
        }
    }
}

fn split_grid_track_tokens(value: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
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
                    return None;
                }
                depth -= 1;
                current.push(character);
            }
            character if character.is_ascii_whitespace() && depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if depth != 0 {
        return None;
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    (!tokens.is_empty()).then_some(tokens)
}

fn css_grid_track_token_to_bui(value: &str) -> Option<String> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("auto") {
        return Some("auto".to_string());
    }

    if value.eq_ignore_ascii_case("min-content") {
        return Some("min_content".to_string());
    }

    if value.eq_ignore_ascii_case("max-content") {
        return Some("max_content".to_string());
    }

    if let Some(px) = value.strip_suffix("px") {
        return px.parse::<f32>().ok().map(|_| format!("px({px})"));
    }

    if let Some(fr) = value.strip_suffix("fr") {
        let fr = fr.trim();
        let fraction = if fr.is_empty() { "1" } else { fr };
        return fraction
            .parse::<f32>()
            .ok()
            .map(|_| format!("flex({fraction})"));
    }

    if let Some(content) = value
        .strip_prefix("minmax(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let max = args[1].trim();
            if let Some(fr) = max.strip_suffix("fr") {
                let fr = fr.trim();
                let fraction = if fr.is_empty() { "1" } else { fr };
                return fraction
                    .parse::<f32>()
                    .ok()
                    .map(|_| format!("flex({fraction})"));
            }
            return css_grid_track_token_to_bui(max);
        }
        return None;
    }

    if let Some(content) = value
        .strip_prefix("repeat(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let repetition = args[0].trim().parse::<u16>().ok()?;
            return css_grid_track_token_to_bui_repeat(repetition, args[1].trim());
        }
        return None;
    }

    None
}

fn css_grid_track_token_to_bui_repeat(repetition: u16, value: &str) -> Option<String> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("auto") {
        return Some(format!("auto({repetition})"));
    }

    if value.eq_ignore_ascii_case("min-content") {
        return Some(format!("min_content({repetition})"));
    }

    if value.eq_ignore_ascii_case("max-content") {
        return Some(format!("max_content({repetition})"));
    }

    if let Some(px) = value.strip_suffix("px") {
        return px
            .parse::<f32>()
            .ok()
            .map(|_| format!("px({repetition}, {px})"));
    }

    if let Some(fr) = value.strip_suffix("fr") {
        let fr = fr.trim();
        let fraction = if fr.is_empty() { "1" } else { fr };
        return fraction
            .parse::<f32>()
            .ok()
            .map(|_| format!("flex({repetition}, {fraction})"));
    }

    if let Some(content) = value
        .strip_prefix("minmax(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let args = split_css_function_args(content);
        if args.len() == 2 {
            let max = args[1].trim();
            if let Some(fr) = max.strip_suffix("fr") {
                let fr = fr.trim();
                let fraction = if fr.is_empty() { "1" } else { fr };
                return fraction
                    .parse::<f32>()
                    .ok()
                    .map(|_| format!("flex({repetition}, {fraction})"));
            }
            if let Some(px) = max.strip_suffix("px") {
                return px
                    .parse::<f32>()
                    .ok()
                    .map(|_| format!("px({repetition}, {px})"));
            }
        }
    }

    None
}

fn css_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(color) = css_background_fallback_color(value) {
        return Some(color);
    }
    if let Some(color) = css_color_mix_with_transparency(value) {
        return Some(color);
    }
    if let Some(color) = oklch_to_hex(value) {
        return Some(color);
    }
    if let Some(color) = css_embedded_oklch_color(value) {
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
        if token.eq_ignore_ascii_case("transparent") {
            return Some("transparent".to_string());
        }
        if let Some(color) = oklch_to_hex(token) {
            return Some(color);
        }
        if is_hex_color(token) {
            return Some(token.to_string());
        }
        if let Some(color) = css_named_color(token) {
            return Some(color.to_string());
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

    let alpha = if parts.len() >= 5 && parts[3] == "/" {
        parts[4].parse::<f32>().ok()
    } else if parts.len() >= 4 && parts[3].starts_with('/') {
        let raw = parts[3].strip_prefix('/')?;
        if raw.is_empty() {
            parts.get(4).and_then(|value| value.parse::<f32>().ok())
        } else {
            raw.parse::<f32>().ok()
        }
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

fn css_multiply_blend_fallback_color(value: &str) -> Option<String> {
    let hex = value.trim().strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        3 => {
            let mut characters = hex.chars();
            let r = characters.next()?;
            let g = characters.next()?;
            let b = characters.next()?;
            (
                u8::from_str_radix(&format!("{r}{r}"), 16).ok()?,
                u8::from_str_radix(&format!("{g}{g}"), 16).ok()?,
                u8::from_str_radix(&format!("{b}{b}"), 16).ok()?,
                255,
            )
        }
        4 => {
            let mut characters = hex.chars();
            let r = characters.next()?;
            let g = characters.next()?;
            let b = characters.next()?;
            let a = characters.next()?;
            (
                u8::from_str_radix(&format!("{r}{r}"), 16).ok()?,
                u8::from_str_radix(&format!("{g}{g}"), 16).ok()?,
                u8::from_str_radix(&format!("{b}{b}"), 16).ok()?,
                u8::from_str_radix(&format!("{a}{a}"), 16).ok()?,
            )
        }
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        ),
        _ => return None,
    };

    let darken = |channel: u8| ((channel as f32) * 0.78).round().clamp(0.0, 255.0) as u8;
    let alpha = ((a as f32) * 0.88).round().clamp(0.0, 255.0) as u8;
    let r = darken(r);
    let g = darken(g);
    let b = darken(b);

    if alpha == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, alpha))
    }
}

fn css_adjust_filter_color(
    value: &str,
    adjustment: CssFilterColorAdjustment,
) -> Option<String> {
    let (mut r, mut g, mut b, a) = css_hex_rgba(value)?;

    let apply_channel = |channel: f32| {
        let contrasted = ((channel - 0.5) * adjustment.contrast + 0.5).clamp(0.0, 1.0);
        (contrasted * adjustment.brightness).clamp(0.0, 1.0)
    };

    r = apply_channel(r);
    g = apply_channel(g);
    b = apply_channel(b);

    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    r = (luminance + (r - luminance) * adjustment.saturate).clamp(0.0, 1.0);
    g = (luminance + (g - luminance) * adjustment.saturate).clamp(0.0, 1.0);
    b = (luminance + (b - luminance) * adjustment.saturate).clamp(0.0, 1.0);

    css_rgba_to_hex(r, g, b, a)
}

fn css_hex_rgba(value: &str) -> Option<(f32, f32, f32, f32)> {
    let hex = value.trim().strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        3 => {
            let mut characters = hex.chars();
            let r = characters.next()?;
            let g = characters.next()?;
            let b = characters.next()?;
            (
                u8::from_str_radix(&format!("{r}{r}"), 16).ok()?,
                u8::from_str_radix(&format!("{g}{g}"), 16).ok()?,
                u8::from_str_radix(&format!("{b}{b}"), 16).ok()?,
                255,
            )
        }
        4 => {
            let mut characters = hex.chars();
            let r = characters.next()?;
            let g = characters.next()?;
            let b = characters.next()?;
            let a = characters.next()?;
            (
                u8::from_str_radix(&format!("{r}{r}"), 16).ok()?,
                u8::from_str_radix(&format!("{g}{g}"), 16).ok()?,
                u8::from_str_radix(&format!("{b}{b}"), 16).ok()?,
                u8::from_str_radix(&format!("{a}{a}"), 16).ok()?,
            )
        }
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        ),
        _ => return None,
    };

    Some((
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

fn blend_hex_colors(color_a: &str, color_b: &str, ratio: f32) -> Option<String> {
    let (r_a, g_a, b_a, a_a) = css_hex_rgba(color_a)?;
    let (r_b, g_b, b_b, a_b) = css_hex_rgba(color_b)?;
    let t = ratio.clamp(0.0, 1.0);
    let r = ((r_a * (1.0 - t) + r_b * t) * 255.0).round() as u8;
    let g = ((g_a * (1.0 - t) + g_b * t) * 255.0).round() as u8;
    let b = ((b_a * (1.0 - t) + b_b * t) * 255.0).round() as u8;
    let a = ((a_a * (1.0 - t) + a_b * t) * 255.0).round() as u8;
    if a == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }
}

fn css_rgba_to_hex(r: f32, g: f32, b: f32, a: f32) -> Option<String> {
    let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (a.clamp(0.0, 1.0) * 255.0).round() as u8;

    if a == 255 {
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        Some(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
    }
}

fn css_background_image_url(value: &str) -> Option<String> {
    let value = value.trim();
    let url_start = value.find("url(")?;
    let rest = &value[url_start + 4..];
    let url_end = rest.find(')')?;
    let raw = rest[..url_end].trim().trim_matches('"').trim_matches('\'');
    (!raw.is_empty()).then(|| raw.to_string())
}

fn css_text_shadow(value: &str) -> Option<BuiTextShadowConfig> {
    let layer = split_css_layers(value).into_iter().next()?;
    let mut offset_x = None;
    let mut offset_y = None;
    let mut color = css_color(&layer);

    for token in css_size_tokens(&layer) {
        if color.is_none() && let Some(parsed) = css_color(&token) {
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

fn css_aspect_ratio(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some((left, right)) = value.split_once('/') {
        let left = left.trim().parse::<f32>().ok()?;
        let right = right.trim().parse::<f32>().ok()?;
        if right == 0.0 {
            return None;
        }
        return Some((left / right).to_string());
    }

    value.parse::<f32>().ok().map(|ratio| ratio.to_string())
}

fn css_box_shadow(value: &str) -> Option<BuiBoxShadowConfig> {
    let inset = value.split_whitespace().any(|token| token == "inset");
    let mut offset_x = None;
    let mut offset_y = None;
    let mut blur_radius = None;
    let mut spread_radius = None;
    let mut color = None;

    let tokens: Vec<&str> = value
        .split_whitespace()
        .filter(|token| !matches!(*token, "inset"))
        .collect();

    let mut sizes = Vec::new();
    for token in tokens {
        if color.is_none() && let Some(parsed) = css_color(token) {
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

    (offset_x.is_some() || offset_y.is_some() || blur_radius.is_some() || spread_radius.is_some() || color.is_some())
        .then_some(BuiBoxShadowConfig {
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color,
        })
}

fn css_box_shadow_layers(value: &str) -> Vec<BuiBoxShadowConfig> {
    split_css_layers(value)
        .into_iter()
        .filter_map(css_box_shadow)
        .take(4)
        .collect()
}

fn css_filter_drop_shadows(value: &str) -> Vec<BuiBoxShadowConfig> {
    let mut shadows = Vec::new();
    let mut rest = value.trim();

    while let Some(start) = rest.find("drop-shadow(") {
        let tail = &rest[start + "drop-shadow(".len()..];
        let Some(end) = tail.find(')') else {
            break;
        };
        if let Some(shadow) = css_box_shadow(tail[..end].trim()) {
            shadows.push(shadow);
        }
        rest = &tail[end + 1..];
    }

    shadows
}

fn css_filter_shadow_length(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
}

fn css_filter_blur_radius(value: &str) -> Option<f32> {
    let value = value.trim();
    let start = value.find("blur(")?;
    let inner = &value[start + "blur(".len()..];
    let end = inner.find(')')?;
    inner[..end].trim().strip_suffix("px")?.parse::<f32>().ok()
}

fn css_filter_color_adjustment(value: &str) -> Option<CssFilterColorAdjustment> {
    let brightness = css_filter_scalar_function(value, "brightness").unwrap_or(1.0);
    let contrast = css_filter_scalar_function(value, "contrast").unwrap_or(1.0);
    let saturate = css_filter_scalar_function(value, "saturate").unwrap_or(1.0);

    ((brightness - 1.0).abs() > 0.001
        || (contrast - 1.0).abs() > 0.001
        || (saturate - 1.0).abs() > 0.001)
        .then_some(CssFilterColorAdjustment {
            brightness,
            contrast,
            saturate,
        })
}

fn css_filter_scalar_function(value: &str, function_name: &str) -> Option<f32> {
    let start = value.find(&format!("{function_name}("))?;
    let inner = &value[start + function_name.len() + 1..];
    let end = inner.find(')')?;
    inner[..end].trim().parse::<f32>().ok()
}

fn apply_filter_blur_fallback(bui_node: &mut BuiNode, blur_radius: f32) {
    let blur_radius = blur_radius.max(0.0);
    if blur_radius <= 0.0 {
        return;
    }

    bui_node
        .children
        .retain(|child| !child.custom_tags.iter().any(|tag| tag == "css-filter-blur"));

    if let Some(text_config) = &mut bui_node.text_config {
        if text_config.text_shadow.is_none() {
            text_config.text_shadow = Some(BuiTextShadowConfig {
                offset_x: Some(0.0),
                offset_y: Some(0.0),
                color: Some(append_hex_alpha(&text_config.font_color, 55.0).unwrap_or_else(|| {
                    text_config.font_color.clone()
                })),
            });
        }
        return;
    }

    let blur_px = format!("{}px", (blur_radius * 4.0).round());
    let spread_px = format!("{}px", (blur_radius * 1.5).round());
    let fallback_color = bui_node
        .visuals
        .background_color
        .as_deref()
        .and_then(|color| append_hex_alpha(color, 65.0));
    let blur_shadow = BuiBoxShadowConfig {
        inset: false,
        offset_x: Some("0px".to_string()),
        offset_y: Some("0px".to_string()),
        blur_radius: Some(blur_px),
        spread_radius: Some(spread_px),
        color: fallback_color,
    };
    push_box_shadow_layer(bui_node, blur_shadow, "css-filter-blur", "filter_blur");
}

fn apply_filter_color_adjustment(bui_node: &mut BuiNode, adjustment: CssFilterColorAdjustment) {
    if let Some(color) = &mut bui_node.visuals.background_color
        && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
    {
        *color = adjusted;
    }
    if let Some(color) = &mut bui_node.visuals.border_color
        && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
    {
        *color = adjusted;
    }
    if let Some(box_shadow) = &mut bui_node.visuals.box_shadow
        && let Some(color) = &mut box_shadow.color
        && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
    {
        *color = adjusted;
    }
    if let Some(text_config) = &mut bui_node.text_config {
        if let Some(adjusted) = css_adjust_filter_color(&text_config.font_color, adjustment) {
            text_config.font_color = adjusted;
        }
        if let Some(text_shadow) = &mut text_config.text_shadow
            && let Some(color) = &mut text_shadow.color
            && let Some(adjusted) = css_adjust_filter_color(color, adjustment)
        {
            *color = adjusted;
        }
    }

    for child in &mut bui_node.children {
        apply_filter_color_adjustment(child, adjustment);
    }
}

fn apply_state_filter_color_adjustment(
    state_visual: &mut BuiStateVisual,
    adjustment: CssFilterColorAdjustment,
    base_background_color: Option<&str>,
    base_border_color: Option<&str>,
    base_text_color: Option<&str>,
) {
    if let Some(base) = state_visual
        .visuals
        .background_color
        .clone()
        .or_else(|| base_background_color.map(ToString::to_string))
        && let Some(adjusted) = css_adjust_filter_color(&base, adjustment)
    {
        state_visual.visuals.background_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .visuals
        .border_color
        .clone()
        .or_else(|| base_border_color.map(ToString::to_string))
        && let Some(adjusted) = css_adjust_filter_color(&base, adjustment)
    {
        state_visual.visuals.border_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .text_color
        .clone()
        .or_else(|| base_text_color.map(ToString::to_string))
        && let Some(adjusted) = css_adjust_filter_color(&base, adjustment)
    {
        state_visual.text_color = Some(adjusted);
    }
}

fn apply_state_opacity_fallback(
    state_visual: &mut BuiStateVisual,
    opacity: f32,
    base_background_color: Option<&str>,
    base_border_color: Option<&str>,
    base_text_color: Option<&str>,
) {
    if let Some(base) = state_visual
        .visuals
        .background_color
        .clone()
        .or_else(|| base_background_color.map(ToString::to_string))
        && let Some(adjusted) = append_hex_alpha(&base, opacity * 100.0)
    {
        state_visual.visuals.background_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .visuals
        .border_color
        .clone()
        .or_else(|| base_border_color.map(ToString::to_string))
        && let Some(adjusted) = append_hex_alpha(&base, opacity * 100.0)
    {
        state_visual.visuals.border_color = Some(adjusted);
    }

    if let Some(base) = state_visual
        .text_color
        .clone()
        .or_else(|| base_text_color.map(ToString::to_string))
        && let Some(adjusted) = append_hex_alpha(&base, opacity * 100.0)
    {
        state_visual.text_color = Some(adjusted);
    }
}

fn apply_mix_blend_mode_fallback(bui_node: &mut BuiNode, value: &str) {
    let mode = normalize_token(value);
    if mode != "multiply" {
        return;
    }

    if let Some(color) = &mut bui_node.visuals.background_color
        && let Some(mixed) = css_multiply_blend_fallback_color(color)
    {
        *color = mixed;
    }

    if let Some(color) = &mut bui_node.visuals.border_color
        && let Some(mixed) = css_multiply_blend_fallback_color(color)
    {
        *color = mixed;
    }

    if let Some(box_shadow) = &mut bui_node.visuals.box_shadow
        && let Some(color) = &mut box_shadow.color
        && let Some(mixed) = css_multiply_blend_fallback_color(color)
    {
        *color = mixed;
    }

    for child in &mut bui_node.children {
        let is_effect_helper = child.custom_tags.iter().any(|tag| {
            tag == "css-gradient-overlay"
                || tag == "css-box-shadow-layer"
                || tag == "css-filter-drop-shadow"
                || tag == "css-filter-blur"
        });
        if !is_effect_helper {
            continue;
        }

        if let Some(color) = &mut child.visuals.background_color
            && let Some(mixed) = css_multiply_blend_fallback_color(color)
        {
            *color = mixed;
        }
        if let Some(color) = &mut child.visuals.border_color
            && let Some(mixed) = css_multiply_blend_fallback_color(color)
        {
            *color = mixed;
        }
        if let Some(box_shadow) = &mut child.visuals.box_shadow
            && let Some(color) = &mut box_shadow.color
            && let Some(mixed) = css_multiply_blend_fallback_color(color)
        {
            *color = mixed;
        }
    }
}

fn apply_box_shadow_fallback(node: &mut BuiNode, value: &str) {
    node
        .children
        .retain(|child| !child.custom_tags.iter().any(|tag| tag == "css-box-shadow-layer"));

    let shadows = css_box_shadow_layers(value);
    if shadows.is_empty() {
        node.visuals.box_shadow = None;
        return;
    }

    let primary_index = shadows
        .iter()
        .position(|shadow| !shadow.inset)
        .unwrap_or(0);
    node.visuals.box_shadow = Some(shadows[primary_index].clone());

    if shadows.len() == 1 {
        return;
    }

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
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

fn push_box_shadow_layer(
    node: &mut BuiNode,
    shadow: BuiBoxShadowConfig,
    custom_tag: &str,
    id_suffix: &str,
) {
    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    let layer_count = node
        .children
        .iter()
        .filter(|child| {
            child.custom_tags.iter().any(|tag| {
                tag == "css-box-shadow-layer" || tag == "css-filter-drop-shadow"
            })
        })
        .count();

    let mut layer = bui_node(&format!("{}_{}", node.id, id_suffix), BuiNodeType::Node);
    layer.custom_tags.push(custom_tag.to_string());
    layer.styles.position_type = Some("absolute".to_string());
    layer.styles.left = Some("0".to_string());
    layer.styles.right = Some("0".to_string());
    layer.styles.top = Some("0".to_string());
    layer.styles.bottom = Some("0".to_string());
    layer.styles.z_index = Some(format!("-{}", layer_count + 1));
    layer.visuals.box_shadow = Some(shadow);
    layer.visuals.border_radius = node.visuals.border_radius.clone();
    node.children.insert(0, layer);
}

fn css_font_size(value: &str) -> Option<f32> {
    if let Some(size) = css_eval_length_function(value) {
        return size.strip_suffix("px")?.parse::<f32>().ok();
    }

    css_size_tokens(value)
        .into_iter()
        .filter_map(|part| part.strip_suffix("px").and_then(|number| number.parse::<f32>().ok()))
        .next()
}

fn css_letter_spacing(value: &str) -> Option<f32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") || value == "0" {
        return Some(0.0);
    }

    value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
}

fn css_line_height(value: &str) -> Option<String> {
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

fn css_text_align(value: &str) -> Option<&str> {
    matches!(
        value,
        "left" | "center" | "right" | "justify" | "justified" | "start" | "end"
    )
    .then_some(value)
}

fn apply_css_white_space(text_config: &mut BuiTextConfig, value: &str) {
    match normalize_token(value).as_str() {
        "normal" => {
            text_config.allow_newlines = Some(false);
            text_config.linebreak = Some("word_boundary".to_string());
        }
        "nowrap" | "no_wrap" => {
            text_config.allow_newlines = Some(false);
            text_config.linebreak = Some("no_wrap".to_string());
        }
        "pre" => {
            text_config.allow_newlines = Some(true);
            text_config.linebreak = Some("no_wrap".to_string());
        }
        "pre_line" => {
            text_config.allow_newlines = Some(true);
            text_config.linebreak = Some("word_boundary".to_string());
        }
        "pre_wrap" | "break_spaces" => {
            text_config.allow_newlines = Some(true);
            text_config.linebreak = Some("any_character".to_string());
        }
        _ => {}
    }
}

fn css_font_weight(value: &str) -> Option<u16> {
    match normalize_token(value).as_str() {
        "normal" => Some(400),
        "bold" => Some(700),
        "bolder" => Some(700),
        "lighter" => Some(300),
        other => other
            .parse::<u16>()
            .ok()
            .map(|weight| weight.clamp(1, 1000)),
    }
}

fn css_font_family_to_path(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("sfmono")
        || lower.contains("menlo")
        || lower.contains("monospace")
        || lower.contains("consolas")
        || lower.contains("ui-monospace")
    {
        "Menlo.ttc".to_string()
    } else if lower.contains("palatino")
        || lower.contains("iowan")
        || lower.contains("georgia")
        || lower.contains("serif")
    {
        "Palatino.ttc".to_string()
    } else if lower.contains("songti") {
        "Songti.ttc".to_string()
    } else if lower.contains("pingfang") {
        "PingFang.ttc".to_string()
    } else if lower.contains("stheiti") {
        "STHeiti Medium.ttc".to_string()
    } else {
        "Hiragino Sans GB.ttc".to_string()
    }
}

fn adjust_font_path_for_content(font_path: &str, content: &str) -> String {
    if uses_latin_display_font(font_path) && contains_cjk(content) {
        return "Songti.ttc".to_string();
    }

    font_path.to_string()
}

fn uses_latin_display_font(font_path: &str) -> bool {
    matches!(
        font_path,
        "Palatino.ttc" | "Georgia.ttf" | "Times New Roman.ttf"
    )
}

fn contains_cjk(content: &str) -> bool {
    content.chars().any(is_cjk_character)
}

fn is_cjk_character(character: char) -> bool {
    matches!(
        character as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
            | 0x30000..=0x3134F
    )
}

fn css_background_fallback_color(value: &str) -> Option<String> {
    let layers = split_css_layers(value);
    if layers.len() <= 1 {
        return None;
    }

    for layer in layers.iter().rev() {
        if let Some(color) = css_simple_color(layer) {
            if color != "transparent" {
                return Some(color);
            }
        }
    }

    for layer in layers.iter().rev() {
        if let Some(color) = css_gradient_representative_color(layer) {
            if color != "transparent" {
                return Some(color);
            }
        }
    }

    None
}

fn css_simple_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(color) = css_color_mix_with_transparency(value) {
        return Some(color);
    }
    if let Some(color) = oklch_to_hex(value) {
        return Some(color);
    }
    if let Some(color) = css_embedded_oklch_color(value) {
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
        if let Some(color) = css_named_color(token) {
            return Some(color.to_string());
        }
    }
    css_named_color(value).map(ToString::to_string)
}

fn css_embedded_oklch_color(value: &str) -> Option<String> {
    let start = value.find("oklch(")?;
    let slice = &value[start..];
    let mut depth = 0usize;
    let mut end_index = None;

    for (index, character) in slice.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_index = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }

    let end = end_index?;
    oklch_to_hex(&slice[..=end])
}

fn split_css_layers(value: &str) -> Vec<&str> {
    let mut layers = Vec::new();
    let mut depth: usize = 0;
    let mut start = 0;

    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let layer = value[start..index].trim();
                if !layer.is_empty() {
                    layers.push(layer);
                }
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    let tail = value[start..].trim();
    if !tail.is_empty() {
        layers.push(tail);
    }

    layers
}

fn css_gradient_representative_color(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.contains("-gradient(") {
        return None;
    }

    let gradient_color_stops = split_css_layers(
        value
            .split_once('(')?
            .1
            .strip_suffix(')')?
            .trim(),
    );

    let mut colors = Vec::new();
    for stop in gradient_color_stops {
        if let Some(color) = css_simple_color(stop) {
            if color != "transparent" {
                colors.push(color);
            }
        }
    }

    colors.last().cloned().or_else(|| css_gradient_first_color(value))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SimpleGradientOverlayDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

fn apply_simple_gradient_overlays(node: &mut BuiNode, value: &str) {
    let specs = css_simple_gradient_overlays(value);
    if specs.is_empty() {
        return;
    }

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    for (index, spec) in specs.into_iter().enumerate().rev() {
        let overlay_id = gradient_overlay_id(&node.id, index);
        if node.children.iter().any(|child| child.id == overlay_id) {
            continue;
        }

        let mut overlay = bui_node(&overlay_id, BuiNodeType::Node);
        overlay.custom_tags.push("css-gradient-overlay".to_string());
        overlay.styles.position_type = Some("absolute".to_string());
        overlay.styles.z_index = Some(format!("-{}", index + 1));
        overlay.visuals.background_color = Some(spec.color.clone());

        match spec.kind {
            SimpleGradientOverlayKind::Linear {
                direction,
                start_ratio,
                end_ratio,
            } => match direction {
                SimpleGradientOverlayDirection::LeftToRight => {
                    overlay.styles.left = Some(format!("{:.0}%", start_ratio * 100.0));
                    overlay.styles.right = Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                    overlay.styles.top = Some("0".to_string());
                    overlay.styles.bottom = Some("0".to_string());
                }
                SimpleGradientOverlayDirection::RightToLeft => {
                    overlay.styles.left = Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                    overlay.styles.right = Some(format!("{:.0}%", start_ratio * 100.0));
                    overlay.styles.top = Some("0".to_string());
                    overlay.styles.bottom = Some("0".to_string());
                }
                SimpleGradientOverlayDirection::TopToBottom => {
                    overlay.styles.left = Some("0".to_string());
                    overlay.styles.right = Some("0".to_string());
                    overlay.styles.top = Some(format!("{:.0}%", start_ratio * 100.0));
                    overlay.styles.bottom = Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                }
                SimpleGradientOverlayDirection::BottomToTop => {
                    overlay.styles.left = Some("0".to_string());
                    overlay.styles.right = Some("0".to_string());
                    overlay.styles.top = Some(format!("{:.0}%", (1.0 - end_ratio) * 100.0));
                    overlay.styles.bottom = Some(format!("{:.0}%", start_ratio * 100.0));
                }
            },
            SimpleGradientOverlayKind::Radial {
                left,
                top,
                width,
                height,
            } => {
                overlay.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.visuals.border_radius = Some("999px".to_string());
            }
            SimpleGradientOverlayKind::RadialRing {
                left,
                top,
                width,
                height,
                border_width,
            } => {
                overlay.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.visuals.background_color = Some("transparent".to_string());
                overlay.visuals.border_color = Some(spec.color.clone());
                overlay.visuals.border_width = Some(format!("{:.1}%", border_width * 100.0));
                overlay.visuals.border_radius = Some("999px".to_string());
            }
            SimpleGradientOverlayKind::ConicArc {
                left,
                top,
                width,
                height,
                rotation_degrees,
            } => {
                overlay.styles.left = Some(format!("{:.0}%", left * 100.0));
                overlay.styles.top = Some(format!("{:.0}%", top * 100.0));
                overlay.styles.width = Some(format!("{:.0}%", width * 100.0));
                overlay.styles.height = Some(format!("{:.0}%", height * 100.0));
                overlay.styles.ui_rotation = Some(format!("{rotation_degrees:.1}deg"));
                overlay.visuals.border_radius = Some("999px".to_string());
            }
        }

        node.children.insert(0, overlay);
    }
}

fn css_simple_gradient_overlays(value: &str) -> Vec<SimpleGradientOverlaySpec> {
    split_css_layers(value)
        .into_iter()
        .take(3)
        .flat_map(css_simple_gradient_overlay_layer)
        .take(8)
        .collect()
}

fn css_simple_gradient_overlay_layer(layer: &str) -> Vec<SimpleGradientOverlaySpec> {
    let linear = css_simple_linear_gradient_overlays(layer);
    if !linear.is_empty() {
        return linear;
    }

    let conic = css_simple_conic_gradient_overlays(layer);
    if !conic.is_empty() {
        return conic;
    }

    if let Some(ring) = css_simple_radial_gradient_ring_overlay(layer) {
        return vec![SimpleGradientOverlaySpec {
            color: ring.color,
            kind: SimpleGradientOverlayKind::RadialRing {
                left: ring.left,
                top: ring.top,
                width: ring.width,
                height: ring.height,
                border_width: ring.border_width,
            },
        }];
    }

    let Some(radial) = css_simple_radial_gradient_overlay(layer) else {
        return Vec::new();
    };

    vec![SimpleGradientOverlaySpec {
        color: radial.color,
        kind: SimpleGradientOverlayKind::Radial {
            left: radial.left,
            top: radial.top,
            width: radial.width,
            height: radial.height,
        },
    }]
}

fn css_simple_linear_gradient_overlays(layer: &str) -> Vec<SimpleGradientOverlaySpec> {
    let Some((direction, bands)) = css_simple_linear_gradient_bands(layer) else {
        return Vec::new();
    };

    bands
        .into_iter()
        .take(3)
        .map(|band| SimpleGradientOverlaySpec {
            color: band.color,
            kind: SimpleGradientOverlayKind::Linear {
                direction,
                start_ratio: band.start_ratio,
                end_ratio: band.end_ratio,
            },
        })
        .collect()
}

fn css_simple_linear_gradient_bands(
    layer: &str,
) -> Option<(SimpleGradientOverlayDirection, Vec<SimpleGradientOverlayBand>)> {
    let layer = layer.trim();
    let inner = layer.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let args = split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let (direction, stop_start_index) = css_simple_linear_gradient_direction(&args)?;
    let stops = css_gradient_stops(&args[stop_start_index..])?;
    let bands = css_simple_gradient_bands_from_stops(&stops);
    if bands.is_empty() {
        return None;
    }

    Some((direction, bands))
}

fn css_simple_gradient_bands_from_stops(
    stops: &[CssGradientStop],
) -> Vec<SimpleGradientOverlayBand> {
    if !stops.iter().any(|stop| stop.color == "transparent") {
        return css_simple_solid_gradient_bands(stops);
    }

    let mut bands = Vec::new();
    let mut index = 0usize;

    while index < stops.len() {
        if stops[index].color == "transparent" {
            index += 1;
            continue;
        }

        let stop = &stops[index];
        let previous_stop = index.checked_sub(1).and_then(|previous| stops.get(previous));
        let next_stop = stops.get(index + 1);

        let start_ratio = previous_stop
            .map(|previous| previous.end_ratio)
            .unwrap_or(stop.start_ratio)
            .min(stop.end_ratio);
        let mut end_ratio = stop.end_ratio;
        if let Some(stop) = next_stop.filter(|stop| stop.color == "transparent") {
            end_ratio = end_ratio.max(stop.start_ratio).max(stop.end_ratio);
        }

        bands.push(SimpleGradientOverlayBand {
            color: stop.color.clone(),
            start_ratio,
            end_ratio,
        });

        index += 1;
    }

    bands
        .into_iter()
        .filter(|band| band.end_ratio > band.start_ratio)
        .collect()
}

fn css_simple_solid_gradient_bands(stops: &[CssGradientStop]) -> Vec<SimpleGradientOverlayBand> {
    let non_transparent: Vec<&CssGradientStop> = stops
        .iter()
        .filter(|stop| stop.color != "transparent")
        .collect();

    if non_transparent.len() == 2 {
        let first = non_transparent[0];
        let second = non_transparent[1];
        let _first_end = first.end_ratio.max(first.start_ratio);
        let second_end = second.end_ratio.max(second.start_ratio);
        let gradient_start = first.start_ratio.min(first.end_ratio);
        let gradient_end = second_end;

        let mid_color = blend_hex_colors(&first.color, &second.color, 0.5)
            .unwrap_or_else(|| second.color.clone());
        let mid_point = (gradient_start + gradient_end) * 0.5;

        let mut bands = Vec::new();
        bands.push(SimpleGradientOverlayBand {
            color: first.color.clone(),
            start_ratio: gradient_start,
            end_ratio: mid_point * 0.6 + gradient_start * 0.4,
        });
        bands.push(SimpleGradientOverlayBand {
            color: mid_color,
            start_ratio: mid_point * 0.6 + gradient_start * 0.4,
            end_ratio: mid_point * 0.4 + gradient_end * 0.6,
        });
        bands.push(SimpleGradientOverlayBand {
            color: second.color.clone(),
            start_ratio: mid_point * 0.4 + gradient_end * 0.6,
            end_ratio: gradient_end,
        });
        return bands
            .into_iter()
            .filter(|band| band.end_ratio > band.start_ratio)
            .collect();
    }

    let mut bands = Vec::new();
    for window in non_transparent.windows(2) {
        let [previous, current] = window else {
            continue;
        };
        let start_ratio = ((previous.end_ratio + current.end_ratio) * 0.5).clamp(0.0, 1.0);
        let end_ratio = current.end_ratio.clamp(start_ratio, 1.0);
        if end_ratio <= start_ratio {
            continue;
        }

        bands.push(SimpleGradientOverlayBand {
            color: current.color.clone(),
            start_ratio,
            end_ratio,
        });
    }

    if bands.is_empty() {
        if let Some(stop) = non_transparent.last() {
            bands.push(SimpleGradientOverlayBand {
                color: stop.color.clone(),
                start_ratio: stop.start_ratio.clamp(0.0, 1.0),
                end_ratio: stop.end_ratio.clamp(stop.start_ratio, 1.0),
            });
        }
    }

    bands
}

fn css_simple_linear_gradient_direction(
    args: &[&str],
) -> Option<(SimpleGradientOverlayDirection, usize)> {
    let first = args.first()?.trim();
    if let Some(direction) = css_linear_gradient_direction_from_token(first) {
        return Some((direction, 1));
    }

    Some((SimpleGradientOverlayDirection::TopToBottom, 0))
}

fn css_linear_gradient_direction_from_token(
    token: &str,
) -> Option<SimpleGradientOverlayDirection> {
    let token = token.trim().to_ascii_lowercase();
    if let Some(direction) = css_linear_gradient_direction_from_keyword(&token) {
        return Some(direction);
    }

    let degrees = token.strip_suffix("deg")?.trim().parse::<f32>().ok()?;
    css_linear_gradient_direction_from_degrees(degrees)
}

fn css_linear_gradient_direction_from_keyword(
    token: &str,
) -> Option<SimpleGradientOverlayDirection> {
    let token = token.trim();
    if !token.starts_with("to ") {
        return None;
    }

    let has_left = token.contains("left");
    let has_right = token.contains("right");
    let has_top = token.contains("top");
    let has_bottom = token.contains("bottom");

    if has_left ^ has_right {
        return Some(if has_right {
            SimpleGradientOverlayDirection::LeftToRight
        } else {
            SimpleGradientOverlayDirection::RightToLeft
        });
    }

    if has_top ^ has_bottom {
        return Some(if has_bottom {
            SimpleGradientOverlayDirection::TopToBottom
        } else {
            SimpleGradientOverlayDirection::BottomToTop
        });
    }

    None
}

fn css_linear_gradient_direction_from_degrees(
    degrees: f32,
) -> Option<SimpleGradientOverlayDirection> {
    if !degrees.is_finite() {
        return None;
    }

    let normalized = degrees.rem_euclid(360.0);
    let radians = normalized.to_radians();
    let horizontal = radians.sin();
    let vertical = -radians.cos();

    if horizontal.abs() >= vertical.abs() {
        Some(if horizontal >= 0.0 {
            SimpleGradientOverlayDirection::LeftToRight
        } else {
            SimpleGradientOverlayDirection::RightToLeft
        })
    } else {
        Some(if vertical >= 0.0 {
            SimpleGradientOverlayDirection::TopToBottom
        } else {
            SimpleGradientOverlayDirection::BottomToTop
        })
    }
}

fn gradient_overlay_id(node_id: &str, index: usize) -> String {
    if index == 0 {
        format!("{node_id}_gradient_overlay")
    } else {
        format!("{node_id}_gradient_overlay_{}", index + 1)
    }
}

struct SimpleGradientOverlaySpec {
    color: String,
    kind: SimpleGradientOverlayKind,
}

struct SimpleGradientOverlayBand {
    color: String,
    start_ratio: f32,
    end_ratio: f32,
}

enum SimpleGradientOverlayKind {
    Linear {
        direction: SimpleGradientOverlayDirection,
        start_ratio: f32,
        end_ratio: f32,
    },
    Radial {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
    },
    RadialRing {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        border_width: f32,
    },
    ConicArc {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        rotation_degrees: f32,
    },
}

struct SimpleRadialGradientOverlay {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    color: String,
}

struct SimpleRadialGradientRingOverlay {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    border_width: f32,
    color: String,
}

struct CssGradientStop {
    color: String,
    start_ratio: f32,
    end_ratio: f32,
}

struct SimpleConicGradientOverlay {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    rotation_degrees: f32,
    color: String,
}

fn css_gradient_stops(args: &[&str]) -> Option<Vec<CssGradientStop>> {
    let raw_stops = args
        .iter()
        .map(|arg| {
            let arg = arg.trim();
            Some((css_color(arg)?, css_gradient_stop_positions(arg)))
        })
        .collect::<Option<Vec<_>>>()?;
    let anchor_positions = css_resolved_gradient_stop_positions(
        &raw_stops
            .iter()
            .map(|(_, positions)| positions.clone())
            .collect::<Vec<_>>(),
    );

    let mut stops = Vec::new();
    let mut previous_end = 0.0;

    for (index, (color, positions)) in raw_stops.into_iter().enumerate() {
        let (start_ratio, end_ratio) = match positions.as_slice() {
            [] => {
                let end = anchor_positions[index];
                if index == 0 {
                    (0.0, end)
                } else {
                    (previous_end, end)
                }
            }
            [single] => {
                if index == 0 {
                    (0.0, *single)
                } else {
                    (previous_end, *single)
                }
            }
            [start, end, ..] => (*start, *end),
        };

        let start_ratio = start_ratio.clamp(0.0, 1.0);
        let end_ratio = end_ratio.clamp(start_ratio, 1.0);
        previous_end = end_ratio;

        stops.push(CssGradientStop {
            color,
            start_ratio,
            end_ratio,
        });
    }

    Some(stops)
}

fn css_resolved_gradient_stop_positions(raw_positions: &[Vec<f32>]) -> Vec<f32> {
    let mut anchors = raw_positions
        .iter()
        .map(|positions| match positions.as_slice() {
            [] => None,
            [single] => Some(*single),
            [_, end, ..] => Some(*end),
        })
        .collect::<Vec<_>>();

    let mut index = 0usize;
    while index < anchors.len() {
        if anchors[index].is_some() {
            index += 1;
            continue;
        }

        let run_start = index;
        while index < anchors.len() && anchors[index].is_none() {
            index += 1;
        }
        let run_end = index;
        let run_len = run_end - run_start;

        let previous = run_start
            .checked_sub(1)
            .and_then(|previous_index| anchors[previous_index]);
        let next = anchors.get(run_end).copied().flatten();

        match (previous, next) {
            (Some(left), Some(right)) => {
                for offset in 0..run_len {
                    anchors[run_start + offset] =
                        Some(left + (right - left) * ((offset + 1) as f32 / (run_len + 1) as f32));
                }
            }
            (None, Some(right)) => {
                if run_len == 1 {
                    anchors[run_start] = Some(0.0);
                } else {
                    for offset in 0..run_len {
                        anchors[run_start + offset] =
                            Some(right * (offset as f32 / run_len as f32));
                    }
                }
            }
            (Some(left), None) => {
                for offset in 0..run_len {
                    anchors[run_start + offset] = Some(
                        left + (1.0 - left) * ((offset + 1) as f32 / run_len as f32),
                    );
                }
            }
            (None, None) => {
                if run_len == 1 {
                    anchors[run_start] = Some(0.0);
                } else {
                    for offset in 0..run_len {
                        anchors[run_start + offset] =
                            Some(offset as f32 / (run_len - 1) as f32);
                    }
                }
            }
        }
    }

    anchors
        .into_iter()
        .map(|anchor| anchor.unwrap_or(0.0).clamp(0.0, 1.0))
        .collect()
}

fn css_gradient_stop_positions(value: &str) -> Vec<f32> {
    value
        .split_whitespace()
        .filter_map(css_gradient_stop_position_value)
        .collect()
}

fn css_gradient_stop_position_value(value: &str) -> Option<f32> {
    if let Some(percent) = css_percentage_value(value) {
        return Some(percent);
    }

    let value = value.trim();
    let number = value.parse::<f32>().ok()?;
    if number.abs() > f32::EPSILON {
        return None;
    }

    Some(0.0)
}

fn css_simple_radial_gradient_overlay(value: &str) -> Option<SimpleRadialGradientOverlay> {
    let value = value.trim();
    let layer = split_css_layers(value).into_iter().next()?;
    let inner = layer.strip_prefix("radial-gradient(")?.strip_suffix(')')?;
    let args = split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let descriptor = args[0].trim().to_ascii_lowercase();
    let mut center_x = 0.5;
    let mut center_y = 0.5;
    if let Some((_, position)) = descriptor.split_once(" at ") {
        let mut parts = position.split_whitespace();
        if let Some(x) = parts.next().and_then(css_percentage_value) {
            center_x = x;
        }
        if let Some(y) = parts.next().and_then(css_percentage_value) {
            center_y = y;
        }
    }

    let mut color = None;
    let mut stop_ratio = None;
    for arg in args.iter().skip(1) {
        let token = arg.trim();
        let token_color = css_simple_color(token);
        if color.is_none() && token_color.as_deref() != Some("transparent") {
            color = token_color;
        }
        if stop_ratio.is_none() && token.contains("transparent") {
            stop_ratio = token
                .split_whitespace()
                .find_map(css_percentage_value)
                .or(Some(0.5));
        }
    }

    let color = color?;
    let stop_ratio = stop_ratio.unwrap_or(0.5).clamp(0.12, 0.72);
    let ellipse_scale_x = if descriptor.contains("ellipse") { 1.35 } else { 1.0 };
    let ellipse_scale_y = if descriptor.contains("ellipse") { 0.78 } else { 1.0 };
    let width = (stop_ratio * 2.0 * ellipse_scale_x).clamp(0.18, 1.25);
    let height = (stop_ratio * 2.0 * ellipse_scale_y).clamp(0.18, 1.1);
    let left = (center_x - width * 0.5).clamp(-0.2, 1.0);
    let top = (center_y - height * 0.5).clamp(-0.2, 1.0);

    Some(SimpleRadialGradientOverlay {
        left,
        top,
        width,
        height,
        color,
    })
}

fn css_simple_radial_gradient_ring_overlay(value: &str) -> Option<SimpleRadialGradientRingOverlay> {
    let value = value.trim();
    let layer = split_css_layers(value).into_iter().next()?;
    let inner = layer.strip_prefix("radial-gradient(")?.strip_suffix(')')?;
    let args = split_css_function_args(inner);
    if args.len() < 3 {
        return None;
    }

    let descriptor = args[0].trim().to_ascii_lowercase();
    let mut center_x = 0.5;
    let mut center_y = 0.5;
    if let Some((_, position)) = descriptor.split_once(" at ") {
        let mut parts = position.split_whitespace();
        if let Some(x) = parts.next().and_then(css_percentage_value) {
            center_x = x;
        }
        if let Some(y) = parts.next().and_then(css_percentage_value) {
            center_y = y;
        }
    }

    let stops = css_gradient_stops(&args[1..])?;
    let mut inner_ratio = None;
    let mut outer_ratio = None;
    let mut color = None;

    for window in stops.windows(3) {
        let [before, middle, after] = window else { continue };
        if before.color == "transparent"
            && middle.color != "transparent"
            && after.color == "transparent"
        {
            inner_ratio = Some(middle.start_ratio.max(before.end_ratio));
            outer_ratio = Some(middle.end_ratio.min(after.start_ratio.max(middle.end_ratio)));
            color = Some(middle.color.clone());
            break;
        }
    }

    let color = color?;
    let inner_ratio = inner_ratio?;
    let outer_ratio = outer_ratio?;
    if outer_ratio <= inner_ratio {
        return None;
    }

    let ellipse_scale_x = if descriptor.contains("ellipse") { 1.35 } else { 1.0 };
    let ellipse_scale_y = if descriptor.contains("ellipse") { 0.78 } else { 1.0 };
    let width = (outer_ratio * 2.0 * ellipse_scale_x).clamp(0.18, 1.25);
    let height = (outer_ratio * 2.0 * ellipse_scale_y).clamp(0.18, 1.1);
    let left = (center_x - width * 0.5).clamp(-0.2, 1.0);
    let top = (center_y - height * 0.5).clamp(-0.2, 1.0);
    let border_width = ((outer_ratio - inner_ratio) / outer_ratio.max(0.001) * 0.5)
        .clamp(0.01, 0.18);

    Some(SimpleRadialGradientRingOverlay {
        left,
        top,
        width,
        height,
        border_width,
        color,
    })
}

fn css_simple_conic_gradient_overlays(value: &str) -> Vec<SimpleGradientOverlaySpec> {
    let Some(overlays) = css_simple_conic_gradient_overlay_specs(value) else {
        return Vec::new();
    };

    overlays
        .into_iter()
        .take(2)
        .map(|overlay| SimpleGradientOverlaySpec {
            color: overlay.color,
            kind: SimpleGradientOverlayKind::ConicArc {
                left: overlay.left,
                top: overlay.top,
                width: overlay.width,
                height: overlay.height,
                rotation_degrees: overlay.rotation_degrees,
            },
        })
        .collect()
}

fn css_simple_conic_gradient_overlay_specs(
    value: &str,
) -> Option<Vec<SimpleConicGradientOverlay>> {
    let value = value.trim();
    let layer = split_css_layers(value).into_iter().next()?;
    let inner = layer.strip_prefix("conic-gradient(")?.strip_suffix(')')?;
    let args = split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let (from_degrees, center_x, center_y, stop_start_index) =
        css_simple_conic_gradient_descriptor(&args)?;
    let stops = css_gradient_stops(&args[stop_start_index..])?;
    let bands: Vec<SimpleGradientOverlayBand> = stops
        .iter()
        .filter(|stop| stop.color != "transparent" && stop.end_ratio > stop.start_ratio)
        .map(|stop| SimpleGradientOverlayBand {
            color: stop.color.clone(),
            start_ratio: stop.start_ratio,
            end_ratio: stop.end_ratio,
        })
        .collect();
    if bands.is_empty() {
        return None;
    }

    let radius = 0.30f32;
    let thickness = 0.06f32;

    Some(
        bands
            .into_iter()
            .take(2)
            .map(|band| {
                let span = (band.end_ratio - band.start_ratio).clamp(0.02, 0.24);
                let midpoint_ratio = (band.start_ratio + band.end_ratio) * 0.5;
                let midpoint_degrees = from_degrees + midpoint_ratio * 360.0;
                let radians = (midpoint_degrees - 90.0).to_radians();
                let arc_length = (std::f32::consts::TAU * radius * span).clamp(0.08, 0.24);
                let center_arc_x = center_x + radians.cos() * radius;
                let center_arc_y = center_y + radians.sin() * radius;
                let left = (center_arc_x - arc_length * 0.5).clamp(-0.2, 1.0);
                let top = (center_arc_y - thickness * 0.5).clamp(-0.2, 1.0);

                SimpleConicGradientOverlay {
                    left,
                    top,
                    width: arc_length,
                    height: thickness,
                    rotation_degrees: midpoint_degrees,
                    color: band.color,
                }
            })
            .collect(),
    )
}

fn css_simple_conic_gradient_descriptor(args: &[&str]) -> Option<(f32, f32, f32, usize)> {
    let first = args.first()?.trim().to_ascii_lowercase();
    if !first.contains("from ") && !first.contains(" at ") {
        return Some((0.0, 0.5, 0.5, 0));
    }

    let mut from_degrees = 0.0;
    if let Some(from_section) = first.split("from ").nth(1) {
        let angle_token = from_section
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_end_matches(',');
        if let Some(degrees) = angle_token.strip_suffix("deg")
            && let Ok(parsed) = degrees.trim().parse::<f32>()
        {
            from_degrees = parsed;
        }
    }

    let mut center_x = 0.5;
    let mut center_y = 0.5;
    if let Some((_, position_section)) = first.split_once(" at ") {
        let mut parts = position_section.split_whitespace();
        if let Some(x) = parts.next().and_then(css_percentage_value) {
            center_x = x;
        }
        if let Some(y) = parts.next().and_then(css_percentage_value) {
            center_y = y;
        }
    }

    Some((from_degrees, center_x, center_y, 1))
}

fn css_percentage_value(value: &str) -> Option<f32> {
    value
        .trim()
        .strip_suffix('%')?
        .parse::<f32>()
        .ok()
        .map(|value| value / 100.0)
}

#[derive(Clone, Copy)]
enum MaskFadeDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

struct MaskFadeSpec {
    direction: MaskFadeDirection,
    fade_ratio: f32,
}

struct ClipPolygonContourSpec {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    fill_left: f32,
    fill_right: f32,
    fill_top: f32,
    fill_bottom: f32,
    accent_left: f32,
    accent_top: f32,
    accent_width: f32,
    accent_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct CssFilterColorAdjustment {
    brightness: f32,
    contrast: f32,
    saturate: f32,
}

fn apply_mask_image_fallback(node: &mut BuiNode, value: &str) {
    if normalize_token(value) == "none" {
        node.children
            .retain(|child| !child.custom_tags.iter().any(|tag| tag == "css-mask-fade"));
        return;
    }

    let Some(spec) = css_simple_mask_fade(value) else {
        return;
    };

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    for index in 0..3 {
        let overlay_id = format!("{}_mask_fade_{}", node.id, index + 1);
        if node.children.iter().any(|child| child.id == overlay_id) {
            continue;
        }

        let band_count = 3.0;
        let band_start = spec.fade_ratio * (index as f32 / band_count);
        let band_end = spec.fade_ratio * ((index + 1) as f32 / band_count);
        let alpha = match index {
            0 => 62.0,
            1 => 34.0,
            _ => 16.0,
        };

        let mut overlay = bui_node(&overlay_id, BuiNodeType::Node);
        overlay.custom_tags.push("css-mask-fade".to_string());
        overlay.styles.position_type = Some("absolute".to_string());
        overlay.styles.z_index = Some("12".to_string());
        overlay.visuals.background_color =
            append_hex_alpha("#120E12", alpha).or(Some("#120E129E".to_string()));

        match spec.direction {
            MaskFadeDirection::LeftToRight => {
                overlay.styles.left = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.width = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.top = Some("0".to_string());
                overlay.styles.bottom = Some("0".to_string());
            }
            MaskFadeDirection::RightToLeft => {
                overlay.styles.right = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.width = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.top = Some("0".to_string());
                overlay.styles.bottom = Some("0".to_string());
            }
            MaskFadeDirection::TopToBottom => {
                overlay.styles.top = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.height = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.left = Some("0".to_string());
                overlay.styles.right = Some("0".to_string());
            }
            MaskFadeDirection::BottomToTop => {
                overlay.styles.bottom = Some(format!("{:.1}%", band_start * 100.0));
                overlay.styles.height = Some(format!("{:.1}%", (band_end - band_start) * 100.0));
                overlay.styles.left = Some("0".to_string());
                overlay.styles.right = Some("0".to_string());
            }
        }

        node.children.push(overlay);
    }
}

fn apply_clip_path_fallback(node: &mut BuiNode, value: &str) {
    let Some(spec) = css_simple_clip_polygon_contour(value) else {
        return;
    };

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    if node
        .children
        .iter()
        .any(|child| child.custom_tags.iter().any(|tag| tag == "css-clip-contour"))
    {
        return;
    }

    let fill_color = node.visuals.background_color.clone();
    let contour_color = node
        .visuals
        .border_color
        .clone()
        .or_else(|| node.visuals.background_color.as_deref().and_then(|color| append_hex_alpha(color, 42.0)))
        .unwrap_or_else(|| "#F4E8D69C".to_string());
    let accent_color = node
        .visuals
        .background_color
        .as_deref()
        .and_then(|color| append_hex_alpha(color, 58.0))
        .unwrap_or_else(|| "#FFF8EE66".to_string());

    if fill_color.is_some() {
        let mut fill = bui_node(&format!("{}_clip_fill", node.id), BuiNodeType::Node);
        fill.custom_tags.push("css-clip-contour".to_string());
        fill.styles.position_type = Some("absolute".to_string());
        fill.styles.left = Some(format!("{:.1}%", spec.fill_left * 100.0));
        fill.styles.right = Some(format!("{:.1}%", spec.fill_right * 100.0));
        fill.styles.top = Some(format!("{:.1}%", spec.fill_top * 100.0));
        fill.styles.bottom = Some(format!("{:.1}%", spec.fill_bottom * 100.0));
        fill.styles.z_index = Some("1".to_string());
        fill.visuals.background_color = fill_color;
        fill.visuals.border_radius = Some("999px".to_string());
        node.children.push(fill);

        // Keep the node itself for layout, images, and content while moving the fallback paint
        // into a bounded helper layer so clipped shapes degrade less like full rectangles.
        node.visuals.background_color = Some("transparent".to_string());
        node.visuals.border_color = Some("transparent".to_string());
    }

    let mut outer = bui_node(&format!("{}_clip_contour", node.id), BuiNodeType::Node);
    outer.custom_tags.push("css-clip-contour".to_string());
    outer.styles.position_type = Some("absolute".to_string());
    outer.styles.left = Some(format!("{:.1}%", spec.left * 100.0));
    outer.styles.right = Some(format!("{:.1}%", spec.right * 100.0));
    outer.styles.top = Some(format!("{:.1}%", spec.top * 100.0));
    outer.styles.bottom = Some(format!("{:.1}%", spec.bottom * 100.0));
    outer.styles.z_index = Some("3".to_string());
    outer.visuals.background_color = Some("transparent".to_string());
    outer.visuals.border_color = Some(contour_color);
    outer.visuals.border_width = Some("1px".to_string());
    outer.visuals.border_radius = Some("999px".to_string());

    let mut accent = bui_node(&format!("{}_clip_contour_accent", node.id), BuiNodeType::Node);
    accent.custom_tags.push("css-clip-contour".to_string());
    accent.styles.position_type = Some("absolute".to_string());
    accent.styles.left = Some(format!("{:.1}%", spec.accent_left * 100.0));
    accent.styles.top = Some(format!("{:.1}%", spec.accent_top * 100.0));
    accent.styles.width = Some(format!("{:.1}%", spec.accent_width * 100.0));
    accent.styles.height = Some(format!("{:.1}%", spec.accent_height * 100.0));
    accent.styles.z_index = Some("4".to_string());
    accent.visuals.background_color = Some(accent_color);
    accent.visuals.border_radius = Some("999px".to_string());

    node.children.push(outer);
    node.children.push(accent);
}

fn css_simple_mask_fade(value: &str) -> Option<MaskFadeSpec> {
    let value = value.trim();
    let inner = value
        .strip_prefix("linear-gradient(")?
        .strip_suffix(')')?;
    let args = split_css_function_args(inner);
    if args.len() < 2 {
        return None;
    }

    let (direction, stop_start_index) = css_simple_mask_fade_direction(&args)?;

    let stops = css_gradient_stops(&args[stop_start_index..])?;
    let mut transparent_seen = false;
    for stop in stops {
        if stop.color == "transparent" {
            transparent_seen = true;
            continue;
        }
        if transparent_seen && stop.start_ratio > 0.0 {
            return Some(MaskFadeSpec {
                direction,
                fade_ratio: stop.start_ratio.clamp(0.04, 0.35),
            });
        }
    }

    None
}

fn css_simple_mask_fade_direction(args: &[&str]) -> Option<(MaskFadeDirection, usize)> {
    let first = args.first()?.trim();
    if let Some(direction) = css_mask_fade_direction_from_token(first) {
        return Some((direction, 1));
    }

    Some((MaskFadeDirection::TopToBottom, 0))
}

fn css_mask_fade_direction_from_token(token: &str) -> Option<MaskFadeDirection> {
    let token = token.trim().to_ascii_lowercase();
    match token.as_str() {
        "90deg" | "to right" => Some(MaskFadeDirection::LeftToRight),
        "270deg" | "to left" => Some(MaskFadeDirection::RightToLeft),
        "180deg" | "to bottom" => Some(MaskFadeDirection::TopToBottom),
        "0deg" | "360deg" | "to top" => Some(MaskFadeDirection::BottomToTop),
        _ => None,
    }
}

fn css_simple_clip_polygon_contour(value: &str) -> Option<ClipPolygonContourSpec> {
    let value = value.trim();
    let inner = value.strip_prefix("polygon(")?.strip_suffix(')')?;
    let points = split_css_function_args(inner)
        .into_iter()
        .filter_map(|point| {
            let mut parts = point.split_whitespace();
            let x = parts.next().and_then(css_clip_path_coordinate_value)?;
            let y = parts.next().and_then(css_clip_path_coordinate_value)?;
            Some((x, y))
        })
        .collect::<Vec<_>>();

    if points.len() < 3 {
        return None;
    }

    let min_x = points.iter().map(|(x, _)| *x).fold(1.0f32, f32::min);
    let max_x = points.iter().map(|(x, _)| *x).fold(0.0f32, f32::max);
    let min_y = points.iter().map(|(_, y)| *y).fold(1.0f32, f32::min);
    let max_y = points.iter().map(|(_, y)| *y).fold(0.0f32, f32::max);

    let width = (max_x - min_x).clamp(0.08, 1.0);
    let height = (max_y - min_y).clamp(0.08, 1.0);
    let fill_inset_x = (width * 0.06).clamp(0.02, 0.08);
    let fill_inset_top = (height * 0.05).clamp(0.015, 0.06);
    let fill_inset_bottom = (height * 0.03).clamp(0.015, 0.05);

    Some(ClipPolygonContourSpec {
        left: min_x.clamp(0.0, 0.92),
        right: (1.0 - max_x).clamp(0.0, 0.92),
        top: min_y.clamp(0.0, 0.92),
        bottom: (1.0 - max_y).clamp(0.0, 0.92),
        fill_left: (min_x + fill_inset_x).clamp(0.0, 0.95),
        fill_right: ((1.0 - max_x) + fill_inset_x).clamp(0.0, 0.95),
        fill_top: (min_y + fill_inset_top).clamp(0.0, 0.95),
        fill_bottom: ((1.0 - max_y) + fill_inset_bottom).clamp(0.0, 0.95),
        accent_left: (min_x + width * 0.10).clamp(0.0, 0.94),
        accent_top: (min_y + height * 0.06).clamp(0.0, 0.94),
        accent_width: (width * 0.28).clamp(0.08, 0.34),
        accent_height: (height * 0.16).clamp(0.06, 0.24),
    })
}

fn css_clip_path_coordinate_value(value: &str) -> Option<f32> {
    css_percentage_value(value).or_else(|| {
        let value = value.trim();
        let number = value.parse::<f32>().ok()?;
        (number.abs() <= f32::EPSILON).then_some(0.0)
    })
}

fn apply_css_border(bui_node: &mut BuiNode, value: &str) {
    if let Some(width) = css_first_size(value) {
        bui_node.visuals.border_width = Some(width);
    }
    if let Some(color) = css_color(value) {
        bui_node.visuals.border_color = Some(color);
    }
}

fn apply_css_edge_border(bui_node: &mut BuiNode, edge: &str, value: &str) {
    if let Some(color) = css_color(value) {
        ensure_edge_border_node(bui_node, edge).visuals.background_color = Some(color);
    }
    apply_css_edge_border_width(bui_node, edge, value);
}

fn apply_css_edge_border_color(bui_node: &mut BuiNode, edge: &str, value: &str) {
    if let Some(color) = css_color(value) {
        ensure_edge_border_node(bui_node, edge).visuals.background_color = Some(color);
    }
}

fn apply_css_edge_border_width(bui_node: &mut BuiNode, edge: &str, value: &str) {
    let Some(width) = css_first_size(value) else {
        return;
    };

    let border = ensure_edge_border_node(bui_node, edge);
    match edge {
        "top" | "bottom" => border.styles.height = Some(width),
        "left" | "right" => border.styles.width = Some(width),
        _ => {}
    }
}

fn ensure_edge_border_node<'a>(node: &'a mut BuiNode, edge: &str) -> &'a mut BuiNode {
    let border_id = format!("{}_border_{edge}", node.id);
    if let Some(index) = node.children.iter().position(|child| child.id == border_id) {
        return &mut node.children[index];
    }

    if node.styles.position_type.is_none() {
        node.styles.position_type = Some("relative".to_string());
    }

    let mut border = bui_node(&border_id, BuiNodeType::Node);
    border.custom_tags.push(format!("css-edge-border:{edge}"));
    border.styles.position_type = Some("absolute".to_string());

    match edge {
        "top" => {
            border.styles.left = Some("0".to_string());
            border.styles.right = Some("0".to_string());
            border.styles.top = Some("0".to_string());
        }
        "bottom" => {
            border.styles.left = Some("0".to_string());
            border.styles.right = Some("0".to_string());
            border.styles.bottom = Some("0".to_string());
        }
        "left" => {
            border.styles.left = Some("0".to_string());
            border.styles.top = Some("0".to_string());
            border.styles.bottom = Some("0".to_string());
        }
        "right" => {
            border.styles.right = Some("0".to_string());
            border.styles.top = Some("0".to_string());
            border.styles.bottom = Some("0".to_string());
        }
        _ => {}
    }

    node.children.push(border);
    node
        .children
        .last_mut()
        .expect("just inserted edge border child")
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
            if !matches!(node.node_type, BuiNodeType::Node | BuiNodeType::Button) {
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
    if let Some(box_shadow) = &visuals.box_shadow {
        validate_box_shadow(box_shadow)?;
    }
    if let Some(shader) = &visuals.material_shader {
        if shader.trim().is_empty() {
            return Err("visuals.material_shader must not be empty.".to_string());
        }
    }

    Ok(())
}

fn validate_box_shadow(box_shadow: &BuiBoxShadowConfig) -> Result<(), String> {
    if let Some(value) = &box_shadow.offset_x {
        parse_val(value)?;
    }
    if let Some(value) = &box_shadow.offset_y {
        parse_val(value)?;
    }
    if let Some(value) = &box_shadow.blur_radius {
        parse_val(value)?;
    }
    if let Some(value) = &box_shadow.spread_radius {
        parse_val(value)?;
    }
    if let Some(color) = &box_shadow.color {
        parse_color(color)?;
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
    if let Some(font_weight) = text_config.font_weight
        && !(1..=1000).contains(&font_weight)
    {
        return Err("text_config.font_weight must be between 1 and 1000.".to_string());
    }
    if let Some(line_height) = &text_config.line_height {
        parse_text_line_height(line_height)?;
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

fn background_image_layout(image_config: &BuiImageConfig) -> Option<BuiBackgroundImageLayout> {
    if image_config.background_size.is_none() && image_config.background_position.is_none() {
        return None;
    }

    Some(BuiBackgroundImageLayout {
        size: image_config.background_size.clone(),
        position: image_config.background_position.clone(),
    })
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
            if let Some(image_config) = &node.image_config {
                entity_commands.insert(build_image_node(
                    asset_server,
                    texture_atlases,
                    image_config,
                )?);
                if let Some(layout) = background_image_layout(image_config) {
                    entity_commands.insert(layout);
                }
            }
        }
        BuiNodeType::Button => {
            entity_commands.insert((Button, build_node(&node.styles, &node.visuals)?));
            if let Some(image_config) = &node.image_config {
                entity_commands.insert(build_image_node(
                    asset_server,
                    texture_atlases,
                    image_config,
                )?);
                if let Some(layout) = background_image_layout(image_config) {
                    entity_commands.insert(layout);
                }
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
                    weight: text_config
                        .font_weight
                        .map(FontWeight)
                        .unwrap_or(FontWeight::NORMAL),
                    ..Default::default()
                },
                TextColor(parse_color(&text_config.font_color)?),
                text_layout(text_config)?,
                FocusPolicy::Pass,
            ));
            if let Some(line_height) = text_config.line_height.as_deref() {
                entity_commands.insert(parse_text_line_height(line_height)?);
            }
            if let Some(letter_spacing) = text_config.letter_spacing {
                entity_commands.insert(LetterSpacing::Px(letter_spacing));
            }
            if let Some(text_shadow) = text_shadow(text_config)? {
                entity_commands.insert(text_shadow);
            }
        }
        BuiNodeType::TextInput => {
            let text_config = text_input_config(node)?;
            let text_font = TextFont {
                font: load_font(asset_server, text_config.font_path.as_deref()),
                font_size: FontSize::Px(text_config.font_size),
                weight: text_config
                    .font_weight
                    .map(FontWeight)
                    .unwrap_or(FontWeight::NORMAL),
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
            if let Some(line_height) = text_config.line_height.as_deref() {
                entity_commands.insert(parse_text_line_height(line_height)?);
            }
            if let Some(letter_spacing) = text_config.letter_spacing {
                entity_commands.insert(LetterSpacing::Px(letter_spacing));
            }

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
            if let Some(line_height) = text_config.line_height.as_deref() {
                commands
                    .entity(mirror)
                    .insert(parse_text_line_height(line_height)?);
            }
            if let Some(letter_spacing) = text_config.letter_spacing {
                commands
                    .entity(mirror)
                    .insert(LetterSpacing::Px(letter_spacing));
            }
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
            if let Some(layout) = background_image_layout(image_config) {
                entity_commands.insert(layout);
            }
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

fn sync_background_image_layout_system(
    mut query: Query<(&ComputedNode, &BuiBackgroundImageLayout, &mut ImageNode, &ImageNodeSize)>,
) {
    for (computed_node, layout, mut image_node, image_size) in &mut query {
        let texture_size = image_size.size().as_vec2();
        let node_size = computed_node.size() * computed_node.inverse_scale_factor();

        if texture_size.cmple(Vec2::ZERO).any() || node_size.cmple(Vec2::ZERO).any() {
            continue;
        }

        image_node.rect = compute_background_rect(texture_size, node_size, layout);
    }
}

fn compute_background_rect(
    texture_size: Vec2,
    node_size: Vec2,
    layout: &BuiBackgroundImageLayout,
) -> Option<Rect> {
    let size = layout.size.as_deref().unwrap_or("auto");
    let position = layout.position.as_deref().unwrap_or("center");

    if size.eq_ignore_ascii_case("cover") {
        let scale = (node_size.x / texture_size.x)
            .max(node_size.y / texture_size.y)
            .max(0.0);
        let crop_size = if scale > 0.0 {
            node_size / scale
        } else {
            texture_size
        };
        let origin = background_crop_origin(texture_size, crop_size, position);
        let max = origin + crop_size;
        return Some(Rect { min: origin, max });
    }

    if let Some((width_scale, height_scale)) = css_background_scale(size, texture_size) {
        let crop_size = Vec2::new(
            (node_size.x / width_scale).min(texture_size.x),
            (node_size.y / height_scale).min(texture_size.y),
        );
        let origin = background_crop_origin(texture_size, crop_size, position);
        let max = origin + crop_size;
        return Some(Rect { min: origin, max });
    }

    None
}

fn css_background_scale(value: &str, texture_size: Vec2) -> Option<(f32, f32)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [single] => {
            if single.eq_ignore_ascii_case("contain") || single.eq_ignore_ascii_case("auto") {
                return None;
            }
            let scale = css_background_scale_component(single, texture_size.x)?;
            Some((scale, scale))
        }
        [x, y] => {
            let scale_x = css_background_scale_component(x, texture_size.x)?;
            let scale_y = css_background_scale_component(y, texture_size.y)?;
            Some((scale_x, scale_y))
        }
        _ => None,
    }
}

fn css_background_scale_component(value: &str, texture_axis: f32) -> Option<f32> {
    let value = value.trim();
    if let Some(percent) = value.strip_suffix('%').and_then(|part| part.parse::<f32>().ok()) {
        return Some((percent / 100.0).max(0.0001));
    }
    if let Some(px) = value.strip_suffix("px").and_then(|part| part.parse::<f32>().ok()) {
        if texture_axis > 0.0 {
            return Some((px / texture_axis).max(0.0001));
        }
    }
    None
}

fn background_crop_origin(texture_size: Vec2, crop_size: Vec2, position: &str) -> Vec2 {
    let (x_ratio, y_ratio) = css_background_position(position);
    Vec2::new(
        ((texture_size.x - crop_size.x).max(0.0) * x_ratio).clamp(0.0, texture_size.x),
        ((texture_size.y - crop_size.y).max(0.0) * y_ratio).clamp(0.0, texture_size.y),
    )
}

fn css_background_position(value: &str) -> (f32, f32) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [] => (0.5, 0.5),
        [single] => {
            let ratio = css_background_position_component(single).unwrap_or(0.5);
            let (x, y) = match (*single).to_ascii_lowercase().as_str() {
                "left" | "right" => (ratio, 0.5),
                "top" | "bottom" => (0.5, ratio),
                _ => (ratio, 0.5),
            };
            (x, y)
        }
        [x, y, ..] => (
            css_background_position_component(x).unwrap_or(0.5),
            css_background_position_component(y).unwrap_or(0.5),
        ),
    }
}

fn css_background_position_component(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "left" | "top" => Some(0.0),
        "center" => Some(0.5),
        "right" | "bottom" => Some(1.0),
        _ => value
            .strip_suffix('%')
            .and_then(|part| part.parse::<f32>().ok())
            .map(|percent| (percent / 100.0).clamp(0.0, 1.0)),
    }
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
    if let Some(text_align) = &text_config.text_align {
        let justify = parse_text_justify(text_align)?;
        if let Some(linebreak) = &text_config.linebreak {
            return Ok(TextLayout::new(justify, parse_linebreak(linebreak)?));
        }
        if text_config.allow_newlines.unwrap_or(false) {
            return Ok(TextLayout::justify(justify));
        }
        return Ok(TextLayout::no_wrap().with_justify(justify));
    }

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

    if let Some(box_shadow) = &node.visuals.box_shadow {
        entity_commands.insert(build_box_shadow(box_shadow)?);
    }

    if let Some(shader_path) = &node.visuals.material_shader {
        entity_commands.insert(BuiMaterialShader {
            path: shader_path.clone(),
        });
    }

    Ok(())
}

fn build_box_shadow(box_shadow: &BuiBoxShadowConfig) -> Result<BoxShadow, String> {
    let color = if let Some(color) = &box_shadow.color {
        parse_color(color)?
    } else {
        Color::NONE
    };

    let x_offset = box_shadow
        .offset_x
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);
    let y_offset = box_shadow
        .offset_y
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);
    let blur_radius = box_shadow
        .blur_radius
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);
    let spread_radius = box_shadow
        .spread_radius
        .as_deref()
        .map(parse_val)
        .transpose()?
        .unwrap_or(Val::ZERO);

    Ok(BoxShadow::from(ShadowStyle {
        color,
        x_offset: if box_shadow.inset {
            negate_val(x_offset)
        } else {
            x_offset
        },
        y_offset: if box_shadow.inset {
            negate_val(y_offset)
        } else {
            y_offset
        },
        spread_radius: if box_shadow.inset {
            negate_val(spread_radius)
        } else {
            spread_radius
        },
        blur_radius,
    }))
}

fn negate_val(value: Val) -> Val {
    match value {
        Val::Px(v) => Val::Px(-v),
        Val::Percent(v) => Val::Percent(-v),
        other => other,
    }
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

fn parse_text_line_height(value: &str) -> Result<LineHeight, String> {
    let value = value.trim();
    if let Some(px) = value
        .strip_suffix("px")
        .and_then(|number| number.parse::<f32>().ok())
    {
        return Ok(LineHeight::Px(px));
    }

    let scale = value
        .parse::<f32>()
        .map_err(|_| format!("Invalid text_config.line_height '{value}'."))?;
    if scale <= 0.0 {
        return Err("text_config.line_height must be greater than 0.".to_string());
    }

    Ok(LineHeight::RelativeToFont(scale))
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

#[cfg(test)] mod tests;