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
        .unwrap_or_else(|_| "opus48/Dev/action_arena/index.ir.json".to_string());
    common::run_with_editor(&file);
}
