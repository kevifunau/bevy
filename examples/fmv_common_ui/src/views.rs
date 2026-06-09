mod chapter_select;
mod in_game_hud;
mod main_menu;
mod settings;

use bevy::prelude::*;

use crate::FmvAppState;

pub struct ViewsPlugin;

impl Plugin for ViewsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(FmvAppState::MainMenu), main_menu::setup_main_menu)
            .add_systems(
                Update,
                main_menu::main_menu_interaction.run_if(in_state(FmvAppState::MainMenu)),
            )
            .add_systems(OnExit(FmvAppState::MainMenu), main_menu::cleanup_main_menu)
            .add_systems(
                OnEnter(FmvAppState::ChapterSelect),
                chapter_select::setup_chapter_select,
            )
            .add_systems(
                Update,
                (
                    chapter_select::chapter_select_interaction,
                    chapter_select::update_selection_highlight,
                )
                    .run_if(in_state(FmvAppState::ChapterSelect)),
            )
            .add_systems(
                OnExit(FmvAppState::ChapterSelect),
                chapter_select::cleanup_chapter_select,
            )
            .add_systems(OnEnter(FmvAppState::Settings), settings::setup_settings)
            .add_systems(
                Update,
                settings::settings_interaction.run_if(in_state(FmvAppState::Settings)),
            )
            .add_systems(OnExit(FmvAppState::Settings), settings::cleanup_settings)
            .add_systems(
                OnEnter(FmvAppState::InGameHud),
                in_game_hud::setup_in_game_hud,
            )
            .add_systems(
                Update,
                in_game_hud::in_game_hud_interaction.run_if(in_state(FmvAppState::InGameHud)),
            )
            .add_systems(
                OnExit(FmvAppState::InGameHud),
                in_game_hud::cleanup_in_game_hud,
            );
    }
}
