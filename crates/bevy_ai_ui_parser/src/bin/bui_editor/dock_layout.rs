use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use bevy_egui::PrimaryEguiContext;
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::app_state::{EditorState, LibraryItem, PendingAction};
use crate::panels;

#[derive(Debug)]
pub enum EditorTab {
    Canvas,
    Hierarchy,
    Library,
    Inspector,
    StyleEditor,
    Console,
}

pub fn create_dock_state() -> DockState<EditorTab> {
    let mut state = DockState::new(vec![EditorTab::Canvas]);
    let tree = state.main_surface_mut();
    let [canvas, inspector] = tree.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Inspector]);
    let [canvas, left] = tree.split_left(canvas, 0.2, vec![EditorTab::Hierarchy]);
    let [hierarchy, library] = tree.split_below(left, 0.5, vec![EditorTab::Library]);
    let [_canvas, bottom] = tree.split_below(
        canvas,
        0.8,
        vec![EditorTab::StyleEditor, EditorTab::Console],
    );
    let _ = (inspector, hierarchy, library, bottom);
    state
}

pub fn show_dock_ui(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<&mut bevy_egui::EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
    else {
        return;
    };
    let ctx = egui_context.get_mut().clone();

    let mut ui = egui::Ui::new(
        ctx.clone(),
        "bui_editor".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    world.resource_scope::<EditorState, _>(|world, mut editor_state| {
        editor_state.ui(&mut ui, world);
    });
}

impl EditorState {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        egui::Panel::top("menu_bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        if self.dirty {
                            self.pending_action = Some(PendingAction::New);
                        } else {
                            self.do_new(world);
                        }
                        ui.close();
                    }
                    if ui.button("Open...").clicked() {
                        if self.dirty {
                            self.pending_action = Some(PendingAction::Open);
                        } else {
                            self.do_open(world);
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Save (Ctrl+S)").clicked() {
                        self.do_save(world);
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        self.do_save_as(world);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Compile HTML...").clicked() {
                        if self.dirty {
                            self.pending_action = Some(PendingAction::CompileHtml);
                        } else {
                            self.do_compile_html(world);
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        if self.dirty {
                            self.pending_action = Some(PendingAction::Quit);
                        } else {
                            std::process::exit(0);
                        }
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        self.dock_state = create_dock_state();
                        ui.close();
                    }
                });
            });
        });

        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let dirty_text = if self.dirty { "● " } else { "  " };
                ui.colored_label(
                    if self.dirty {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::DARK_GRAY
                    },
                    dirty_text,
                );
                ui.label(format!("File: {}", self.ir_path.display()));

                ui.separator();

                let can_undo = self.undo_stack.can_undo();
                let can_redo = self.undo_stack.can_redo();
                ui.add_enabled_ui(can_undo, |ui| {
                    if ui.button("Undo (Ctrl+Z)").clicked() {
                        let mut doc_resource =
                            world.resource_mut::<bevy_ai_ui_parser::BuiDocumentResource>();
                        let mut doc = doc_resource.0.clone();
                        if self.undo_stack.undo(&mut doc) {
                            doc_resource.0 = doc;
                        }
                    }
                });
                ui.add_enabled_ui(can_redo, |ui| {
                    if ui.button("Redo (Ctrl+Y)").clicked() {
                        let mut doc_resource =
                            world.resource_mut::<bevy_ai_ui_parser::BuiDocumentResource>();
                        let mut doc = doc_resource.0.clone();
                        if self.undo_stack.redo(&mut doc) {
                            doc_resource.0 = doc;
                        }
                    }
                });

                ui.separator();

                if let Some(id) = &self.selected_node_id {
                    ui.label(format!("Selected: {}", id));
                } else {
                    ui.colored_label(egui::Color32::DARK_GRAY, "No selection");
                }
            });
        });

        // "Save changes?" modal dialog
        if self.pending_action.is_some() {
            let action_name = match self.pending_action {
                Some(PendingAction::New) => "create a new project",
                Some(PendingAction::Open) => "open another file",
                Some(PendingAction::CompileHtml) => "compile another HTML",
                Some(PendingAction::Quit) => "quit",
                None => unreachable!(),
            };
            egui::Window::new("Save changes?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "You have unsaved changes. Do you want to save before you {}?",
                        action_name
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.do_save(world);
                            if let Some(action) = self.pending_action.take() {
                                self.execute_pending(action, world);
                            }
                        }
                        if ui.button("Don't Save").clicked() {
                            if let Some(action) = self.pending_action.take() {
                                self.execute_pending(action, world);
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_action = None;
                        }
                    });
                });
        }

        let mut tab_viewer = TabViewer {
            world,
            selected_node_id: &mut self.selected_node_id,
            viewport_rect: &mut self.viewport_rect,
            canvas_zoom: &mut self.canvas_zoom,
            canvas_pan: &mut self.canvas_pan,
            collapsed_nodes: &mut self.collapsed_nodes,
            dragging_library_item: &mut self.dragging_library_item,
            drag_hover_node_id: &mut self.drag_hover_node_id,
        };

        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(&ui.global_style()))
            .show_inside(ui, &mut tab_viewer);
    }

    /// Execute a pending action (called after "Save changes?" dialog resolves).
    fn execute_pending(&mut self, action: PendingAction, world: &mut World) {
        match action {
            PendingAction::New => self.do_new(world),
            PendingAction::Open => self.do_open(world),
            PendingAction::CompileHtml => self.do_compile_html(world),
            PendingAction::Quit => std::process::exit(0),
        }
    }

    fn do_save(&mut self, world: &World) {
        let doc = {
            let doc_resource = world.resource::<bevy_ai_ui_parser::BuiDocumentResource>();
            doc_resource.0.clone()
        };
        if let Err(e) = self.save(&doc) {
            error!("Save failed: {e}");
        } else {
            self.dirty = false;
        }
    }

    fn do_save_as(&mut self, world: &World) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("BUI IR JSON", &["ir.json"])
            .set_file_name("untitled.ir.json")
            .save_file()
        {
            self.ir_path = path;
            self.do_save(world);
        }
    }

    fn do_new(&mut self, world: &mut World) {
        let new_doc = new_empty_document();
        let mut doc_resource = world.resource_mut::<bevy_ai_ui_parser::BuiDocumentResource>();
        doc_resource.0 = new_doc;
        self.ir_path = std::path::PathBuf::from("untitled.ir.json");
        self.dirty = false;
        self.undo_stack.clear();
        self.selected_node_id = None;
        self.collapsed_nodes.clear();
        info!("New project created");
    }

    fn do_open(&mut self, world: &mut World) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("BUI IR JSON", &["ir.json", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<bevy_ai_ui_parser::BuiDocument>(&content) {
                        Ok(doc) => {
                            let mut doc_resource =
                                world.resource_mut::<bevy_ai_ui_parser::BuiDocumentResource>();
                            doc_resource.0 = doc;
                            self.ir_path = path;
                            self.dirty = false;
                            self.undo_stack.clear();
                            self.selected_node_id = None;
                            self.collapsed_nodes.clear();
                            info!("Opened: {}", self.ir_path.display());
                        }
                        Err(e) => error!("Parse error: {e}"),
                    }
                }
                Err(e) => error!("Read error: {e}"),
            }
        }
    }

    fn do_compile_html(&mut self, world: &mut World) {
        if let Some(html_path) = rfd::FileDialog::new()
            .add_filter("HTML", &["html"])
            .pick_file()
        {
            match bevy_ai_ui_parser::opendesign_html_file_to_bui_json(&html_path) {
                Ok(json_str) => {
                    match serde_json::from_str::<bevy_ai_ui_parser::BuiDocument>(&json_str) {
                        Ok(doc) => {
                            let ir_path = html_path.with_extension("ir.json");
                            let mut doc_resource =
                                world.resource_mut::<bevy_ai_ui_parser::BuiDocumentResource>();
                            doc_resource.0 = doc;
                            self.ir_path = ir_path;
                            self.dirty = false;
                            self.undo_stack.clear();
                            self.selected_node_id = None;
                            self.collapsed_nodes.clear();
                            info!("Compiled: {}", html_path.display());
                        }
                        Err(e) => error!("Parse compiled IR error: {e}"),
                    }
                }
                Err(e) => error!("Compile error: {e}"),
            }
        }
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_node_id: &'a mut Option<String>,
    viewport_rect: &'a mut egui::Rect,
    canvas_zoom: &'a mut f32,
    canvas_pan: &'a mut egui::Vec2,
    collapsed_nodes: &'a mut std::collections::HashSet<String>,
    dragging_library_item: &'a mut Option<crate::app_state::LibraryItem>,
    drag_hover_node_id: &'a mut Option<String>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Canvas => {
                *self.viewport_rect = ui.clip_rect();
                let zoom = *self.canvas_zoom;
                let pan = *self.canvas_pan;
                if let Some((new_zoom, new_pan)) = panels::canvas_panel(ui, self.world, zoom, pan) {
                    *self.canvas_zoom = new_zoom;
                    *self.canvas_pan = new_pan;
                }
            }
            EditorTab::Hierarchy => {
                panels::hierarchy_panel(
                    ui,
                    self.world,
                    self.selected_node_id,
                    self.collapsed_nodes,
                    self.dragging_library_item,
                    self.drag_hover_node_id,
                );
            }
            EditorTab::Library => {
                panels::library_panel(
                    ui,
                    self.world,
                    self.dragging_library_item,
                    self.drag_hover_node_id,
                );
            }
            EditorTab::Inspector => {
                panels::inspector_panel(ui, self.world, &self.selected_node_id.clone());
            }
            EditorTab::StyleEditor => {
                panels::style_editor_panel(ui, self.world, &self.selected_node_id.clone());
            }
            EditorTab::Console => {
                panels::console_panel(ui);
            }
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::Canvas)
    }
}

pub fn set_camera_viewport(
    editor_state: Res<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam: Single<&mut Camera, With<crate::CanvasCamera>>,
) {
    let scale_factor = window.scale_factor();
    let viewport_rect = editor_state.viewport_rect;

    // Guard against uninitialized viewport
    if viewport_rect.width() < 1.0 || viewport_rect.height() < 1.0 {
        cam.viewport = None;
        return;
    }

    let physical_position = bevy::math::UVec2::new(
        (viewport_rect.left().max(0.0) * scale_factor) as u32,
        (viewport_rect.top().max(0.0) * scale_factor) as u32,
    );

    let window_size = window.physical_size();
    let max_w = window_size.x.saturating_sub(physical_position.x);
    let max_h = window_size.y.saturating_sub(physical_position.y);

    let physical_size = bevy::math::UVec2::new(
        ((viewport_rect.width() * scale_factor) as u32)
            .min(max_w)
            .max(1),
        ((viewport_rect.height() * scale_factor) as u32)
            .min(max_h)
            .max(1),
    );

    cam.viewport = Some(Viewport {
        physical_position,
        physical_size,
        depth: 0.0..1.0,
    });
}

/// Create a minimal empty BuiDocument for "New" project.
fn new_empty_document() -> bevy_ai_ui_parser::BuiDocument {
    use bevy_ai_ui_parser::*;
    BuiDocument {
        version: "3.0-ir".to_string(),
        scene_name: "Untitled".to_string(),
        imports: vec![],
        state_model: BuiStateModel::default(),
        interaction_model: BuiInteractionModel::default(),
        resources: BuiResources::default(),
        root: BuiNode {
            id: "root".to_string(),
            kind: "node".to_string(),
            markers: vec![],
            classes: vec![],
            actions: vec![],
            bindings: vec![],
            layout: BuiLayout {
                styles: BuiStyles {
                    display: Some("flex".to_string()),
                    flex_direction: Some("column".to_string()),
                    width: Some("100%".to_string()),
                    height: Some("100%".to_string()),
                    ..Default::default()
                },
            },
            style: Default::default(),
            content: Default::default(),
            semantics: Default::default(),
            state_visuals: Default::default(),
            children: vec![],
        },
    }
}
