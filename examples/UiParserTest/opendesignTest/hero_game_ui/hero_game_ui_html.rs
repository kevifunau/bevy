//! Renders the hero game UI directly from the OpenDesign HTML artifact.
//!
//! Run with:
//! `cargo run --example hero_game_ui_html`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_html_without_button_feedback(
        "opendesignTest/hero_game_ui/hero-game-ui.html",
    );
}