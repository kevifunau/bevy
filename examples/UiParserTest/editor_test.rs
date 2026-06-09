//! Runs the hero game UI IR snapshot with the BUI editor enabled.
//!
//! Press F8 to toggle editor mode.
//!
//! Run with:
//! `cargo run --example editor_test`

#[path = "common.rs"]
mod common;

fn main() {
    let file = std::env::var("BUI_EDITOR_FILE")
        .unwrap_or_else(|_| "opendesignTest/hero_game_ui/hero-game-ui.ir.json".to_string());
    common::run_with_bui_file_with_editor(&file);
}
