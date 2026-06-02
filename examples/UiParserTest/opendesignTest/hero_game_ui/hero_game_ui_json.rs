//! Renders the hero game UI from the checked-in 2.x compatibility JSON.
//!
//! Run with:
//! `cargo run --example hero_game_ui_json`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json_without_button_feedback(
        "opendesignTest/hero_game_ui/hero-game-ui.json",
    );
}