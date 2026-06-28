use std::{
    fs,
    path::{Path, PathBuf},
};

use bevy_app::{App, Plugin, PostUpdate, Startup, Update};
use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_log::{error, info};
use bevy_ui::UiSystems;

use crate::core::{
    interaction::{
        action_registry::BuiActionRegistry,
        actions::dispatch_bui_actions_system,
        bindings::apply_bui_binding_updates_system,
        dropdown::{
            dispatch_bui_dropdown_selection_system, focused_dropdown_confirm_system,
            sync_bui_dropdown_selected_state_system,
        },
        keyboard::{
            focused_control_confirm_system, keyboard_focus_navigation_system, pointer_focus_system,
        },
        list::{
            json_array_to_object_list, seed_json_list_state, sync_bui_list_groups_system,
            sync_json_list_state_system, BuiJsonListData, BuiJsonListStore,
        },
        progress::sync_bui_progress_groups_system,
        schedule::{configure_bui_system_sets, BuiSystems},
        scroll::{
            dispatch_scroll_view_changed_system, focused_scroll_view_keyboard_input_system,
            scroll_view_wheel_input_system,
        },
        slider::{dispatch_slider_value_changed_system, focused_slider_keyboard_input_system},
        state::apply_bui_state_updates_system,
        state_init::emit_initial_bui_binding_updates_system,
        state_visual::apply_bui_visual_states_system,
        tabs::{dispatch_bui_tab_selection_system, sync_bui_tab_selected_state_system},
        text_input::{
            dispatch_text_input_focus_events_system, dispatch_text_input_submit_system,
            dispatch_text_input_value_changed_system, sync_text_input_mirror_system,
            text_input_proxy_focus_system, BuiTextInputFocusState,
        },
        toggle::{
            resolve_ui_target_camera_system, toggle_interaction_system, update_toggle_visual_system,
        },
        types::{
            BuiActionTriggered, BuiBindingUpdate, BuiBindingValue, BuiStateSet, BuiStateStore,
        },
    },
    model::BuiDocument,
    opendesign::{
        html::opendesign_html_to_bui_document_with_manifest,
        manifest::{discover_manifest_path, load_manifest_file},
    },
    parse::ir::parse_bui_document,
    parse::validate::validate_bui_document,
    runtime::components::{BuiDocumentResource, BuiIdMap, BuiRootEntity, BuiSourcePaths},
    runtime::{
        declarative_actions::{
            advance_delayed_declarative_actions_system, apply_declarative_action_system,
            BuiDelayedActionQueue,
        },
        diagnostics::{material_shader_notice_system, spawn_error_text},
        spawn::{spawn_bui_tree, sync_background_image_layout_system},
        stage_fit::sync_stage_fit_system,
    },
};

/// Plugin that parses BUI JSON and spawns a native Bevy UI tree.
pub struct AiUiPlugin {
    source: BuiSource,
    editor_enabled: bool,
}

impl AiUiPlugin {
    /// Load BUI JSON from a file path.
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::Path(path.into()),
            editor_enabled: false,
        }
    }

    /// Load BUI JSON from an in-memory string.
    pub fn from_json(json: impl Into<String>) -> Self {
        Self {
            source: BuiSource::Inline(json.into()),
            editor_enabled: false,
        }
    }

    /// Load an OpenDesign HTML artifact from a file path and compile it into BUI.
    pub fn from_html_path(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::HtmlPath(path.into()),
            editor_enabled: false,
        }
    }

    /// Load an OpenDesign HTML artifact from an in-memory string and compile it into BUI.
    pub fn from_html(html: impl Into<String>) -> Self {
        Self {
            source: BuiSource::HtmlInline(html.into()),
            editor_enabled: false,
        }
    }

    /// Load BUI JSON from a file path with editor enabled.
    pub fn from_path_with_editor(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::Path(path.into()),
            editor_enabled: true,
        }
    }

    /// Load BUI JSON from an in-memory string with editor enabled.
    pub fn from_json_with_editor(json: impl Into<String>) -> Self {
        Self {
            source: BuiSource::Inline(json.into()),
            editor_enabled: true,
        }
    }

    /// Load an OpenDesign HTML artifact from a file path with editor enabled.
    pub fn from_html_path_with_editor(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::HtmlPath(path.into()),
            editor_enabled: true,
        }
    }

    /// Load an OpenDesign HTML artifact from an in-memory string with editor enabled.
    pub fn from_html_with_editor(html: impl Into<String>) -> Self {
        Self {
            source: BuiSource::HtmlInline(html.into()),
            editor_enabled: true,
        }
    }
}

impl Plugin for AiUiPlugin {
    fn build(&self, app: &mut App) {
        configure_bui_system_sets(app);

        let source_paths = derive_source_paths(&self.source);

        app.insert_resource(AiUiSource(self.source.clone()))
            .insert_resource(source_paths)
            .init_resource::<BuiStateStore>()
            .init_resource::<BuiActionRegistry>()
            .init_resource::<BuiIdMap>()
            .init_resource::<BuiDelayedActionQueue>()
            .init_resource::<BuiTextInputFocusState>()
            .init_resource::<BuiPanelSwitch>()
            .init_resource::<BuiPanelPaths>()
            .init_resource::<BuiJsonListStore>()
            .add_message::<BuiActionTriggered>()
            .add_message::<BuiBindingUpdate>()
            .add_message::<BuiStateSet>()
            .add_systems(Startup, spawn_bui_scene)
            .add_systems(Update, process_panel_switch)
            .add_systems(
                Update,
                (
                    material_shader_notice_system,
                    sync_background_image_layout_system,
                    scroll_view_wheel_input_system,
                    focused_scroll_view_keyboard_input_system,
                    focused_slider_keyboard_input_system,
                    focused_dropdown_confirm_system,
                    keyboard_focus_navigation_system,
                    focused_control_confirm_system,
                    dispatch_bui_actions_system,
                    dispatch_text_input_value_changed_system,
                    dispatch_slider_value_changed_system,
                    dispatch_scroll_view_changed_system.after(scroll_view_wheel_input_system),
                    dispatch_text_input_submit_system,
                    dispatch_text_input_focus_events_system,
                    apply_declarative_action_system
                        .after(dispatch_bui_actions_system)
                        .after(dispatch_text_input_value_changed_system)
                        .after(dispatch_slider_value_changed_system)
                        .after(dispatch_scroll_view_changed_system)
                        .after(dispatch_text_input_submit_system)
                        .after(dispatch_text_input_focus_events_system),
                    advance_delayed_declarative_actions_system
                        .after(apply_declarative_action_system),
                    text_input_proxy_focus_system,
                    sync_text_input_mirror_system,
                    pointer_focus_system,
                    resolve_ui_target_camera_system,
                ),
            )
            .add_systems(
                Update,
                (
                    emit_initial_bui_binding_updates_system,
                    dispatch_bui_tab_selection_system,
                    dispatch_bui_dropdown_selection_system,
                    apply_bui_state_updates_system,
                    sync_json_list_state_system,
                )
                    .chain()
                    .in_set(BuiSystems::DataUpdate),
            )
            .add_systems(
                Update,
                (
                    apply_bui_binding_updates_system,
                    sync_bui_list_groups_system,
                    sync_bui_progress_groups_system,
                    sync_bui_tab_selected_state_system,
                    sync_bui_dropdown_selected_state_system,
                )
                    .in_set(BuiSystems::BindingSync),
            )
            .add_systems(
                Update,
                (
                    toggle_interaction_system,
                    apply_bui_visual_states_system,
                    update_toggle_visual_system,
                )
                    .chain()
                    .in_set(BuiSystems::VisualResolve),
            );

        app.configure_sets(Update, BuiSystems::DataUpdate.before(UiSystems::Prepare));
        app.configure_sets(Update, BuiSystems::BindingSync.before(UiSystems::Prepare));
        app.configure_sets(Update, BuiSystems::VisualResolve.before(UiSystems::Prepare));
        app.add_systems(Update, sync_stage_fit_system.before(UiSystems::Prepare));
        if self.editor_enabled && source_supports_editor(&self.source) {
            app.init_resource::<crate::core::editor::state::BuiEditorState>();
            app.add_systems(
                Update,
                crate::core::editor::auto_enable::maybe_enable_editor_on_first_frame_system
                    .before(crate::core::editor::toggle::toggle_editor_mode_system),
            );
            app.add_systems(
                Update,
                crate::core::editor::toggle::toggle_editor_mode_system,
            );
            app.add_systems(
                Update,
                crate::core::editor::borders::sync_editor_border_system,
            );
            app.add_systems(Update, crate::core::editor::hover::editor_hover_system);
            app.add_systems(Update, crate::core::editor::delete::editor_delete_system);
            app.add_systems(Update, crate::core::editor::drag::editor_drag_system);
            app.add_systems(
                Update,
                crate::core::editor::automation::run_debug_automation_system
                    .after(crate::core::editor::drag::editor_drag_system)
                    .after(crate::core::editor::delete::editor_delete_system),
            );
            app.add_systems(
                Update,
                crate::core::editor::debug::force_debug_hover_node_system
                    .after(crate::core::editor::automation::run_debug_automation_system)
                    .after(crate::core::editor::hover::editor_hover_system)
                    .after(crate::core::editor::drag::editor_drag_system),
            );
            app.add_systems(
                Update,
                crate::core::editor::borders::update_editor_border_visibility_system
                    .after(crate::core::editor::hover::editor_hover_system)
                    .after(crate::core::editor::drag::editor_drag_system)
                    .after(crate::core::editor::debug::force_debug_hover_node_system),
            );
            app.add_systems(
                Update,
                (
                    crate::core::editor::save::editor_save_system,
                    crate::core::editor::discard::editor_discard_system,
                )
                    .before(crate::core::editor::dialog::editor_dialog_system),
            );
            app.add_systems(Update, crate::core::editor::dialog::editor_dialog_system);
            app.add_systems(
                PostUpdate,
                crate::core::editor::borders::update_border_positions_system,
            );
        } else if self.editor_enabled {
            info!("BUI editor is only enabled for file-based JSON/IR sources.");
        }
    }
}

fn source_supports_editor(source: &BuiSource) -> bool {
    matches!(source, BuiSource::Path(_))
}

fn derive_source_paths(source: &BuiSource) -> BuiSourcePaths {
    match source {
        BuiSource::Path(path) => BuiSourcePaths {
            ir_json_path: Some(path.clone()),
            html_path: None,
        },
        BuiSource::Inline(_) => BuiSourcePaths {
            ir_json_path: None,
            html_path: None,
        },
        BuiSource::HtmlPath(path) => {
            let ir_path = path.with_extension("ir.json");
            BuiSourcePaths {
                ir_json_path: Some(ir_path),
                html_path: Some(path.clone()),
            }
        }
        BuiSource::HtmlInline(_) => BuiSourcePaths {
            ir_json_path: None,
            html_path: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        derive_source_paths, load_json_list_store, seed_json_list_state, source_supports_editor,
        BuiSource,
    };
    use crate::core::{
        interaction::types::{BuiBindingValue, BuiStateStore},
        model::{bui_node, BuiDocument, BuiResources, BuiSemantics, BuiStateModel},
    };

    #[test]
    fn html_editor_sources_write_to_sibling_ir_json() {
        let paths = derive_source_paths(&BuiSource::HtmlPath(PathBuf::from(
            "/tmp/example.hero.html",
        )));

        assert_eq!(
            paths.ir_json_path,
            Some(PathBuf::from("/tmp/example.hero.ir.json"))
        );
        assert_eq!(
            paths.html_path,
            Some(PathBuf::from("/tmp/example.hero.html"))
        );
    }

    #[test]
    fn inline_sources_do_not_expose_save_paths() {
        let paths = derive_source_paths(&BuiSource::Inline("{}".to_string()));

        assert_eq!(paths.ir_json_path, None);
        assert_eq!(paths.html_path, None);
    }

    #[test]
    fn editor_only_supports_file_based_ir_json_sources() {
        assert!(source_supports_editor(&BuiSource::Path(PathBuf::from(
            "/tmp/test.ir.json",
        ))));
        assert!(!source_supports_editor(&BuiSource::Inline(
            "{}".to_string()
        )));
        assert!(!source_supports_editor(&BuiSource::HtmlPath(
            PathBuf::from("/tmp/test.html",)
        )));
        assert!(!source_supports_editor(&BuiSource::HtmlInline(
            "<div></div>".to_string(),
        )));
    }

    #[test]
    fn json_list_sources_seed_regions_and_current_page_state() {
        let temp_dir =
            std::env::temp_dir().join(format!("bui-json-list-test-{}", std::process::id()));
        let asset_dir = temp_dir.join("Asset");
        fs::create_dir_all(&asset_dir).expect("asset dir should be created");
        fs::write(
            asset_dir.join("ServerInfo.json"),
            r#"[
                {"id":1,"name":"一服","state":0,"isNew":false},
                {"id":2,"name":"二服","state":1,"isNew":false},
                {"id":3,"name":"三服","state":2,"isNew":false},
                {"id":4,"name":"四服","state":3,"isNew":false},
                {"id":5,"name":"五服","state":4,"isNew":false},
                {"id":6,"name":"六服","state":0,"isNew":true}
            ]"#,
        )
        .expect("json fixture should be written");

        let mut root = bui_node("root", "node");
        let mut regions = bui_node("regions", "node");
        regions.semantics = BuiSemantics {
            list_binding_source: Some("server.regions".to_string()),
            list_json_source: Some("Asset/ServerInfo.json".to_string()),
            list_json_mode: Some("regions".to_string()),
            list_page_size: Some(5),
            ..Default::default()
        };
        regions
            .children
            .push(bui_node("region_{{index}}", "button"));

        let mut servers = bui_node("servers", "node");
        servers.semantics = BuiSemantics {
            list_binding_source: Some("server.servers".to_string()),
            list_json_source: Some("Asset/ServerInfo.json".to_string()),
            list_json_mode: Some("page".to_string()),
            list_page_size: Some(5),
            list_page_source: Some("server.region".to_string()),
            ..Default::default()
        };
        servers.children.push(bui_node("server_{{id}}", "button"));
        root.children = vec![regions, servers];

        let document = BuiDocument {
            version: "3.0-ir".to_string(),
            scene_name: "JsonListTest".to_string(),
            imports: Vec::new(),
            state_model: BuiStateModel::default(),
            interaction_model: Default::default(),
            resources: BuiResources::default(),
            root,
        };

        let json_lists = load_json_list_store(&document, Some(&temp_dir));
        let mut state_store = BuiStateStore::default();
        seed_json_list_state(&json_lists, &mut state_store);

        let Some(BuiBindingValue::ObjectList(regions)) = state_store.0.get("server.regions") else {
            panic!("regions list should be seeded");
        };
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].get("label").map(String::as_str), Some("1 - 5区"));
        assert_eq!(regions[1].get("label").map(String::as_str), Some("6 - 6区"));

        let Some(BuiBindingValue::ObjectList(first_page)) = state_store.0.get("server.servers")
        else {
            panic!("server page should be seeded");
        };
        assert_eq!(first_page.len(), 5);
        assert_eq!(
            first_page[0].get("label").map(String::as_str),
            Some("1区  一服")
        );

        state_store
            .0
            .insert("server.region".to_string(), BuiBindingValue::Number(1.0));
        seed_json_list_state(&json_lists, &mut state_store);
        let Some(BuiBindingValue::ObjectList(second_page)) = state_store.0.get("server.servers")
        else {
            panic!("server page should be recalculated");
        };
        assert_eq!(second_page.len(), 1);
        assert_eq!(
            second_page[0].get("label").map(String::as_str),
            Some("6区  六服")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}

#[derive(Resource, Clone)]
pub(crate) struct AiUiSource(pub(crate) BuiSource);

#[derive(Clone)]
pub(crate) enum BuiSource {
    Path(PathBuf),
    Inline(String),
    HtmlPath(PathBuf),
    HtmlInline(String),
}

pub(crate) fn spawn_bui_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut state_store: ResMut<BuiStateStore>,
    mut id_map_res: ResMut<BuiIdMap>,
    source: Res<AiUiSource>,
) {
    match load_bui_document(&source.0) {
        Ok(document) => {
            info!("Spawning BUI scene '{}'.", document.scene_name);

            let base_dir = source_base_dir(&source.0);
            seed_state_model(&document, &mut state_store);
            let json_lists = load_json_list_store(&document, base_dir.as_deref());
            seed_json_list_state(&json_lists, &mut state_store);

            match spawn_bui_tree(
                &mut commands,
                &asset_server,
                &mut texture_atlases,
                &document,
            ) {
                Ok((root, id_map)) => {
                    commands.insert_resource(BuiRootEntity(root));
                    commands.insert_resource(BuiDocumentResource(document));
                    commands.insert_resource(json_lists);
                    *id_map_res = BuiIdMap(id_map);
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

fn seed_state_model(document: &BuiDocument, state_store: &mut BuiStateStore) {
    for (key, value) in &document.state_model.values {
        state_store
            .0
            .insert(key.clone(), BuiBindingValue::Text(value.clone()));
    }
}

fn source_base_dir(source: &BuiSource) -> Option<PathBuf> {
    match source {
        BuiSource::Path(path) | BuiSource::HtmlPath(path) => path.parent().map(Path::to_path_buf),
        BuiSource::Inline(_) | BuiSource::HtmlInline(_) => None,
    }
}

fn load_json_list_store(document: &BuiDocument, base_dir: Option<&Path>) -> BuiJsonListStore {
    let mut store = BuiJsonListStore::default();
    collect_json_lists(&document.root, base_dir, &mut store);
    store
}

fn collect_json_lists(
    node: &crate::core::model::BuiNode,
    base_dir: Option<&Path>,
    store: &mut BuiJsonListStore,
) {
    if let (Some(binding_source), Some(json_source)) = (
        &node.semantics.list_binding_source,
        &node.semantics.list_json_source,
    ) {
        match load_json_list_source(json_source, base_dir) {
            Ok(items) => {
                store.lists.insert(
                    binding_source.clone(),
                    BuiJsonListData {
                        items,
                        mode: node
                            .semantics
                            .list_json_mode
                            .clone()
                            .unwrap_or_else(|| "all".to_string()),
                        page_size: node.semantics.list_page_size,
                        page_source: node.semantics.list_page_source.clone(),
                    },
                );
            }
            Err(error) => {
                error!(
                    "Failed to load JSON list source '{}' for '{}': {}",
                    json_source, binding_source, error
                );
            }
        }
    }

    for child in &node.children {
        collect_json_lists(child, base_dir, store);
    }
}

fn load_json_list_source(
    source: &str,
    base_dir: Option<&Path>,
) -> Result<Vec<std::collections::HashMap<String, String>>, String> {
    let path = resolve_json_source_path(source, base_dir);
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read '{}': {error}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("Failed to parse '{}': {error}", path.display()))?;
    json_array_to_object_list(&value)
}

fn resolve_json_source_path(source: &str, base_dir: Option<&Path>) -> PathBuf {
    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return source_path;
    }

    if let Some(base_dir) = base_dir {
        let direct = base_dir.join(source);
        if direct.exists() {
            return direct;
        }

        for ancestor in base_dir.ancestors() {
            let webgameui = ancestor.join("webgameui").join(source);
            if webgameui.exists() {
                return webgameui;
            }
        }

        return direct;
    }

    source_path
}

pub(crate) fn load_bui_document(source: &BuiSource) -> Result<BuiDocument, String> {
    match source {
        BuiSource::Path(path) => {
            let raw = fs::read_to_string(path).map_err(|error| {
                format!("Failed to read BUI JSON '{}': {error}", path.display())
            })?;
            let document = parse_bui_document(&raw)?;
            validate_bui_document(&document)?;
            Ok(document)
        }
        BuiSource::Inline(json) => {
            let document = parse_bui_document(json)?;
            validate_bui_document(&document)?;
            Ok(document)
        }
        BuiSource::HtmlPath(path) => {
            let raw = fs::read_to_string(path).map_err(|error| {
                format!(
                    "Failed to read OpenDesign HTML '{}': {error}",
                    path.display()
                )
            })?;
            let manifest_path = discover_manifest_path(path);
            let manifest = manifest_path
                .as_deref()
                .map(load_manifest_file)
                .transpose()?;
            let document = opendesign_html_to_bui_document_with_manifest(
                &raw,
                manifest.as_ref(),
                path.parent(),
            )?;
            let ir_path = path.with_extension("ir.json");
            let ir_json = serde_json::to_string_pretty(&document)
                .map_err(|error| format!("Failed to serialize IR JSON: {error}"))?;
            fs::write(&ir_path, ir_json).map_err(|error| {
                format!("Failed to write IR JSON '{}': {error}", ir_path.display())
            })?;
            Ok(document)
        }
        BuiSource::HtmlInline(html) => {
            opendesign_html_to_bui_document_with_manifest(html, None, None)
        }
    }
}

/// Load an IR JSON file, seed state model, and spawn a BUI entity tree.
///
/// This is the runtime equivalent of Unity's `VisualTreeAsset.CloneTree(root)` —
/// it creates a live Bevy UI entity tree from a compiled IR JSON file.
///
/// Use this in `OnEnter(State::X)` systems to spawn a panel, and tag the
/// returned root entity with `DespawnOnExit(State::X)` for automatic cleanup.
///
/// # Example
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ai_ui_parser::spawn_bui_ir;
/// fn login_enter(mut commands: Commands, asset_server: Res<AssetServer>,
///                texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
///                state_store: ResMut<bevy_ai_ui_parser::BuiStateStore>) {
///     let root = spawn_bui_ir(&mut commands, &asset_server, &mut texture_atlases,
///                             &mut state_store.into_inner(),
///                             "ui/login.ir.json").unwrap();
///     commands.entity(root).insert(bevy::state::state_scoped::DespawnOnExit(MyState::Login));
/// }
/// # #[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug)] enum MyState { Login }
/// ```
pub fn spawn_bui_ir(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    state_store: &mut BuiStateStore,
    ir_path: impl AsRef<Path>,
) -> Result<(Entity, std::collections::HashMap<String, Entity>), String> {
    let path = ir_path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read BUI IR '{}': {error}", path.display()))?;
    let document = parse_bui_document(&raw)?;
    validate_bui_document(&document)?;

    info!(
        "Spawning BUI scene '{}' from {}.",
        document.scene_name,
        path.display()
    );

    seed_state_model(&document, state_store);
    let json_lists = load_json_list_store(&document, path.parent());
    seed_json_list_state(&json_lists, state_store);

    let (root, id_map) = spawn_bui_tree(commands, asset_server, texture_atlases, &document)?;

    commands.insert_resource(BuiRootEntity(root));
    commands.insert_resource(BuiDocumentResource(document));
    commands.insert_resource(BuiIdMap(id_map.clone()));
    commands.insert_resource(json_lists);

    Ok((root, id_map))
}

/// Resource holding a pending panel switch request.
/// Game code calls `switch.show("register")` and the plugin handles the rest.
#[derive(Resource, Default)]
pub struct BuiPanelSwitch {
    /// The panel name to switch to, if any.
    pub pending: Option<String>,
}

impl BuiPanelSwitch {
    /// Request the plugin to switch to a different panel.
    /// Equivalent to Unity's `UIManager.ShowPanel<T>()`.
    pub fn show(&mut self, panel_name: &str) {
        self.pending = Some(panel_name.to_string());
    }
}

/// Resource mapping panel names to IR JSON file paths.
/// Register panels via `app.register_bui_panel("register", "path/to/register.ir.json")`.
#[derive(Resource, Default)]
pub struct BuiPanelPaths(pub std::collections::HashMap<String, String>);

/// Extension trait for registering BUI panels on a Bevy app.
pub trait BuiPanelAppExt {
    /// Register a panel name → IR JSON path mapping.
    /// After registering, game code can call `BuiPanelSwitch::show("name")` to switch to it.
    fn register_bui_panel(&mut self, name: &str, ir_path: impl AsRef<Path>) -> &mut Self;
}

impl BuiPanelAppExt for App {
    fn register_bui_panel(&mut self, name: &str, ir_path: impl AsRef<Path>) -> &mut Self {
        let mut paths = self
            .world_mut()
            .get_resource_or_insert_with::<BuiPanelPaths>(Default::default);
        paths.0.insert(
            name.to_string(),
            ir_path.as_ref().to_string_lossy().to_string(),
        );
        self
    }
}

/// System that processes pending panel switch requests.
/// Despawns the current BUI tree, loads the new IR JSON, and spawns the new tree.
fn process_panel_switch(
    mut switch: ResMut<BuiPanelSwitch>,
    panel_paths: Res<BuiPanelPaths>,
    root: Option<Res<BuiRootEntity>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut state_store: ResMut<BuiStateStore>,
) {
    let Some(panel_name) = switch.pending.take() else {
        return;
    };

    let Some(ir_path) = panel_paths.0.get(&panel_name) else {
        error!(
            "Panel '{}' not registered. Use app.register_bui_panel() first.",
            panel_name
        );
        return;
    };

    // Despawn old BUI tree
    if let Some(root) = root {
        commands.entity(root.0).despawn();
        info!("Despawned previous panel");
    }

    // Load and spawn new panel
    match spawn_bui_ir(
        &mut commands,
        &asset_server,
        &mut texture_atlases,
        &mut state_store,
        ir_path,
    ) {
        Ok((root, _id_map)) => {
            info!("Switched to panel: {}", panel_name);
            let _ = root;
        }
        Err(e) => {
            error!("Failed to switch to panel '{}': {}", panel_name, e);
        }
    }
}
