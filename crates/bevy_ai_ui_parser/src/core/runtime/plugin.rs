use std::{
    fs,
    path::PathBuf,
};

use bevy_app::{App, Plugin, Startup, Update};
use bevy_asset::{AssetServer, Assets};
use bevy_ecs::prelude::*;
use bevy_image::TextureAtlasLayout;
use bevy_log::{error, info};
use bevy_ui::UiSystems;

use crate::core::{
    interaction::{
        actions::dispatch_bui_actions_system,
        bindings::apply_bui_binding_updates_system,
        list::sync_bui_list_groups_system,
        progress::sync_bui_progress_groups_system,
        schedule::{configure_bui_system_sets, BuiSystems},
        state::apply_bui_state_updates_system,
        state_init::emit_initial_bui_binding_updates_system,
        state_visual::apply_bui_visual_states_system,
        tabs::{dispatch_bui_tab_selection_system, sync_bui_tab_selected_state_system},
        text_input::{sync_text_input_mirror_system, text_input_proxy_focus_system},
        toggle::{
            resolve_ui_target_camera_system, toggle_interaction_system, update_toggle_visual_system,
        },
        types::{BuiActionTriggered, BuiBindingUpdate, BuiBindingValue, BuiStateSet, BuiStateStore},
    },
    runtime::components::BuiRootEntity,
    model::BuiDocument,
    opendesign::html::opendesign_html_to_bui_document,
    parse::ir::parse_bui_document,
    parse::validate::validate_bui_document,
    runtime::{
        diagnostics::{material_shader_notice_system, spawn_error_text},
        spawn::{spawn_bui_tree, sync_background_image_layout_system},
    },
};

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

    /// Load an OpenDesign HTML artifact from a file path and compile it into BUI.
    pub fn from_html_path(path: impl Into<PathBuf>) -> Self {
        Self {
            source: BuiSource::HtmlPath(path.into()),
        }
    }

    /// Load an OpenDesign HTML artifact from an in-memory string and compile it into BUI.
    pub fn from_html(html: impl Into<String>) -> Self {
        Self {
            source: BuiSource::HtmlInline(html.into()),
        }
    }
}

impl Plugin for AiUiPlugin {
    fn build(&self, app: &mut App) {
        configure_bui_system_sets(app);

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
                    dispatch_bui_actions_system,
                    text_input_proxy_focus_system,
                    sync_text_input_mirror_system,
                    resolve_ui_target_camera_system,
                ),
            )
            .add_systems(
                Update,
                (
                    emit_initial_bui_binding_updates_system,
                    dispatch_bui_tab_selection_system,
                    apply_bui_state_updates_system,
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
    source: Res<AiUiSource>,
) {
    match load_bui_document(&source.0) {
        Ok(document) => {
            info!("Spawning BUI scene '{}'.", document.scene_name);

            seed_state_model(&document, &mut state_store);

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

fn seed_state_model(document: &BuiDocument, state_store: &mut BuiStateStore) {
    for (key, value) in &document.state_model.values {
        state_store
            .0
            .insert(key.clone(), BuiBindingValue::Text(value.clone()));
    }
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
            opendesign_html_to_bui_document(&raw)
        }
        BuiSource::HtmlInline(html) => opendesign_html_to_bui_document(html),
    }
}