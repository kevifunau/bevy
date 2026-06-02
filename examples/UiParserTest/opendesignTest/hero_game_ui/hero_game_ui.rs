//! Renders the hero game UI from the checked-in `3.0-ir` snapshot.
//!
//! Run with:
//! `cargo run --example hero_game_ui`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_bui_file_without_button_feedback(
        "opendesignTest/hero_game_ui/hero-game-ui.ir.json",
    );
}