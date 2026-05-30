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
use serde::Deserialize;

const EXPECTED_VERSION: &str = "2.0";

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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiDocument {
    version: String,
    scene_name: String,
    root: BuiNode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiNode {
    id: String,
    #[serde(rename = "type")]
    node_type: BuiNodeType,
    #[serde(default)]
    custom_tags: Vec<String>,
    #[serde(default)]
    actions: Vec<BuiActionBinding>,
    #[serde(default)]
    bindings: Vec<BuiBinding>,
    #[serde(default)]
    tab_group_name: Option<String>,
    #[serde(default)]
    tab_binding_source: Option<String>,
    #[serde(default)]
    tab_value: Option<String>,
    #[serde(default)]
    progress_binding_source: Option<String>,
    #[serde(default)]
    progress_fill: bool,
    #[serde(default)]
    list_binding_source: Option<String>,
    #[serde(default)]
    state_visuals: HashMap<String, BuiStateVisual>,
    #[serde(default)]
    styles: BuiStyles,
    #[serde(default)]
    visuals: BuiVisuals,
    #[serde(default)]
    text_config: Option<BuiTextConfig>,
    #[serde(default)]
    image_config: Option<BuiImageConfig>,
    #[serde(default)]
    children: Vec<BuiNode>,
}

#[derive(Debug, Clone, Deserialize)]
enum BuiNodeType {
    Node,
    Text,
    TextInput,
    Toggle,
    Button,
    Image,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
/// Declarative action binding parsed from a BUI node.
pub struct BuiActionBinding {
    event: String,
    emit: String,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiStateVisual {
    #[serde(default)]
    visuals: BuiVisuals,
    #[serde(default)]
    text_color: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiStyles {
    display: Option<String>,
    visibility: Option<String>,
    width: Option<String>,
    height: Option<String>,
    aspect_ratio: Option<String>,
    min_width: Option<String>,
    min_height: Option<String>,
    max_width: Option<String>,
    max_height: Option<String>,
    left: Option<String>,
    right: Option<String>,
    top: Option<String>,
    bottom: Option<String>,
    overflow: Option<String>,
    overflow_clip_margin: Option<String>,
    margin: Option<String>,
    margin_left: Option<String>,
    margin_right: Option<String>,
    margin_top: Option<String>,
    margin_bottom: Option<String>,
    padding: Option<String>,
    padding_left: Option<String>,
    padding_right: Option<String>,
    padding_top: Option<String>,
    padding_bottom: Option<String>,
    flex_direction: Option<String>,
    flex_wrap: Option<String>,
    flex_grow: Option<String>,
    flex_shrink: Option<String>,
    flex_basis: Option<String>,
    row_gap: Option<String>,
    column_gap: Option<String>,
    justify_content: Option<String>,
    justify_items: Option<String>,
    align_content: Option<String>,
    align_items: Option<String>,
    align_self: Option<String>,
    justify_self: Option<String>,
    ui_translation: Option<String>,
    ui_scale: Option<String>,
    ui_rotation: Option<String>,
    tab_group: Option<String>,
    tab_index: Option<String>,
    auto_focus: Option<bool>,
    relative_cursor_position: Option<bool>,
    ui_target_camera: Option<String>,
    position_type: Option<String>,
    fixed_node: Option<bool>,
    z_index: Option<String>,
    global_z_index: Option<String>,
    grid_template_columns: Option<String>,
    grid_template_rows: Option<String>,
    grid_column: Option<String>,
    grid_row: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiVisuals {
    background_color: Option<String>,
    border_color: Option<String>,
    border_width: Option<String>,
    border_radius: Option<String>,
    material_shader: Option<String>,
}

#[derive(Component, Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiTextConfig {
    content: String,
    #[serde(default)]
    placeholder: Option<String>,
    font_size: f32,
    font_color: String,
    #[serde(default)]
    font_path: Option<String>,
    #[serde(default)]
    text_shadow: Option<BuiTextShadowConfig>,
    #[serde(default)]
    linebreak: Option<String>,
    #[serde(default)]
    visible_width: Option<f32>,
    #[serde(default)]
    allow_newlines: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiTextShadowConfig {
    #[serde(default)]
    offset_x: Option<f32>,
    #[serde(default)]
    offset_y: Option<f32>,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiImageConfig {
    texture_path: String,
    #[serde(default)]
    image_mode: Option<String>,
    #[serde(default)]
    atlas: Option<BuiTextureAtlasConfig>,
    #[serde(default)]
    slicer: Option<BuiTextureSlicerConfig>,
    #[serde(default)]
    flip_x: bool,
    #[serde(default)]
    flip_y: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiTextureAtlasConfig {
    tile_width: u32,
    tile_height: u32,
    columns: u32,
    rows: u32,
    #[serde(default)]
    padding_x: Option<u32>,
    #[serde(default)]
    padding_y: Option<u32>,
    #[serde(default)]
    index: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiTextureSlicerConfig {
    border: f32,
    #[serde(default)]
    center_scale_mode: Option<String>,
    #[serde(default)]
    sides_scale_mode: Option<String>,
    #[serde(default)]
    stretch_value: Option<f32>,
    #[serde(default)]
    max_corner_scale: Option<f32>,
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
    let raw = match source {
        BuiSource::Path(path) => fs::read_to_string(path)
            .map_err(|error| format!("Failed to read BUI JSON '{}': {error}", path.display()))?,
        BuiSource::Inline(json) => json.clone(),
    };

    parse_bui_document(&raw)
}

fn parse_bui_document(raw: &str) -> Result<BuiDocument, String> {
    let document: BuiDocument =
        serde_json::from_str(raw).map_err(|error| format!("Invalid BUI JSON: {error}"))?;

    validate_bui_document(&document)?;

    Ok(document)
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

        let Some(group) = tab_groups.iter().find(|group| group.group == tab_item.group) else {
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
        let Some(group) = tab_groups.iter().find(|group| group.group == tab_item.group) else {
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
                    let template = instantiate_list_item_template_text(
                        &list.item_template,
                        index,
                        item,
                    );
                    let Ok(child_entity) =
                        spawn_bui_node(&mut commands, &asset_server, &mut texture_atlases, &template)
                    else {
                        continue;
                    };
                    commands.entity(entity).add_child(child_entity);
                }
            }
            Some(BuiBindingValue::ObjectList(items)) => {
                for (index, item) in items.iter().enumerate() {
                    let template =
                        instantiate_list_item_template_object(&list.item_template, index, item);
                    let Ok(child_entity) =
                        spawn_bui_node(&mut commands, &asset_server, &mut texture_atlases, &template)
                    else {
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
    visual_states: Query<
        (
            Entity,
            &BuiVisualStateDefinitions,
            Option<&BuiVisualState>,
            Option<&Interaction>,
            Has<Checked>,
            Has<BuiDisabled>,
        ),
    >,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut borders: Query<&mut BorderColor>,
    mut text_colors: Query<&mut TextColor>,
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
            .or_else(|| definitions.states.contains_key("normal").then_some("normal"));

        let Some(state_name) = resolve_visual_state_name(
            definitions,
            base_state.as_deref(),
            auto_state,
        ) else {
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
