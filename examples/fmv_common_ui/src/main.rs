//! FMV Common UI - Interactive test example for the data-driven declarative UI system.

mod blackboard;
mod data_schema;
mod director;
mod expression_eval;
mod ui_renderer;
mod views;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use blackboard::BlackboardPlugin;
use director::DirectorPlugin;
use ui_renderer::UiRendererPlugin;
use views::ViewsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FMV Common UI - Interactive Test".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            BlackboardPlugin,
            DirectorPlugin,
            UiRendererPlugin,
            ViewsPlugin,
        ))
        .init_state::<FmvAppState>()
        .add_systems(Startup, spawn_camera)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum FmvAppState {
    #[default]
    MainMenu,
    ChapterSelect,
    Settings,
    InGameHud,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
