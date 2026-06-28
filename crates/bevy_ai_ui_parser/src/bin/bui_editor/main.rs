//! Standalone BUI IR JSON editor with egui dock UI.
//!
//! Run with:
//! `cargo run -p bevy_ai_ui_parser --bin bui_editor --features editor -- <path/to/file.ir.json>`
//!
//! Shortcuts:
//! - Ctrl/Cmd+S: Save
//! - Ctrl/Cmd+Z: Undo
//! - Ctrl/Cmd+Shift+Z / Ctrl/Cmd+Y: Redo
//! - Ctrl/Cmd+C/V: Copy/Paste node
//! - Delete/Backspace: Delete node
//! - Arrow keys: Navigate hierarchy

mod app_state;
mod dock_layout;
mod node_factory;
mod panels;
mod systems;
mod undo;

use std::path::Path;

use bevy::asset::io::AssetSourceBuilder;
use bevy::asset::{AssetPlugin, UnapprovedPathMode};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_ai_ui_parser::AiUiPlugin;
use bevy_egui::{egui, EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

use app_state::EditorState;

fn main() {
    let ir_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example bui_editor -- <path/to/file.ir.json>");
        eprintln!("  No argument provided, using default.");
        "examples/opus48/Dev/action_arena/index.ir.json".to_string()
    });

    let ir_path = std::path::PathBuf::from(&ir_path);
    let ir_path = if ir_path.is_absolute() {
        ir_path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(&ir_path)
    };

    if !ir_path.exists() {
        eprintln!("Error: IR file not found: {}", ir_path.display());
        std::process::exit(1);
    }

    let asset_root = ir_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let mut app = App::new();

    register_optional_macos_fonts_source(&mut app);

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: asset_root.to_string_lossy().to_string(),
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "BUI Editor".to_string(),
                    resolution: WindowResolution::new(1600, 900),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(EguiPlugin::default())
    .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)
    .add_plugins(AiUiPlugin::from_path(ir_path.clone()))
    .insert_resource(app_state::EditorState {
        ir_path: ir_path.clone(),
        dirty: false,
        selected_node_id: None,
        viewport_rect: egui::Rect::NOTHING,
        dock_state: dock_layout::create_dock_state(),
        undo_stack: undo::UndoStack::default(),
        canvas_zoom: 1.0,
        canvas_pan: egui::Vec2::ZERO,
        collapsed_nodes: std::collections::HashSet::new(),
        dragging_library_item: None,
        drag_hover_node_id: None,
        pending_action: None,
    })
    .insert_resource(ClearColor(Color::srgb_u8(30, 30, 35)))
    .insert_resource(systems::DragState::default())
    .insert_resource(systems::Clipboard::default())
    .add_systems(Startup, setup)
    .add_systems(bevy_egui::EguiPrimaryContextPass, dock_layout::show_dock_ui)
    .add_systems(
        Update,
        (
            systems::handle_canvas_click,
            systems::handle_canvas_drag,
            systems::handle_hotkeys,
        ),
    )
    .add_systems(
        PostUpdate,
        (
            dock_layout::set_camera_viewport,
            apply_canvas_transform,
            systems::update_selection_highlight,
        ),
    )
    .run();
}

fn register_optional_macos_fonts_source(app: &mut App) {
    const MACOS_FONTS: &str = "/System/Library/Fonts";
    const MACOS_SUPPLEMENTAL_FONTS: &str = "/System/Library/Fonts/Supplemental";

    let macos_fonts = Path::new(MACOS_FONTS);
    if macos_fonts.exists() {
        app.register_asset_source(
            "macos_fonts",
            AssetSourceBuilder::platform_default(MACOS_FONTS, None),
        );
    }

    let macos_supplemental_fonts = Path::new(MACOS_SUPPLEMENTAL_FONTS);
    if macos_supplemental_fonts.exists() {
        app.register_asset_source(
            "macos_supplemental_fonts",
            AssetSourceBuilder::platform_default(MACOS_SUPPLEMENTAL_FONTS, None),
        );
    }
}

/// Marker for the BUI canvas camera (the one that renders the edited UI).
#[derive(Component)]
struct CanvasCamera;

fn setup(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    egui_global_settings.auto_create_primary_context = false;

    commands.spawn((
        Camera2d,
        CanvasCamera,
        Name::new("BUI canvas camera"),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(
                bevy::color::palettes::css::DARK_SLATE_GRAY.into(),
            ),
            ..default()
        },
    ));

    commands.spawn((
        Camera2d,
        Name::new("Egui camera"),
        PrimaryEguiContext,
        RenderLayers::none(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
}

/// Apply canvas zoom via UiScale (scales all Bevy UI) and pan via camera translation.
fn apply_canvas_transform(
    editor_state: Res<EditorState>,
    mut cam: Single<&mut Transform, With<CanvasCamera>>,
    mut ui_scale: ResMut<bevy::ui::UiScale>,
) {
    let zoom = editor_state.canvas_zoom;
    let pan = editor_state.canvas_pan;

    // Zoom: scale the entire UI tree
    if ui_scale.0 != zoom {
        ui_scale.0 = zoom;
    }

    // Pan: move the canvas camera
    cam.translation.x = pan.x;
    cam.translation.y = -pan.y;
}
