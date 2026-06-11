use std::{fs, path::PathBuf};

use bevy_app::{App, Plugin, PostUpdate, Startup, Update};
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
        diagnostics::{material_shader_notice_system, spawn_error_text},
        spawn::{spawn_bui_tree, sync_background_image_layout_system},
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
            .init_resource::<BuiIdMap>()
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
    use std::path::PathBuf;

    use super::{derive_source_paths, source_supports_editor, BuiSource};

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

            seed_state_model(&document, &mut state_store);

            match spawn_bui_tree(
                &mut commands,
                &asset_server,
                &mut texture_atlases,
                &document,
            ) {
                Ok((root, id_map)) => {
                    commands.insert_resource(BuiRootEntity(root));
                    commands.insert_resource(BuiDocumentResource(document));
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
            opendesign_html_to_bui_document_with_manifest(&raw, manifest.as_ref(), path.parent())
        }
        BuiSource::HtmlInline(html) => {
            opendesign_html_to_bui_document_with_manifest(html, None, None)
        }
    }
}
