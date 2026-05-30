//! Renders the HUD-oriented BUI JSON test case.
//!
//! Run with:
//! `cargo run --example ui_parser_test`

#[path = "../common.rs"]
mod common;

fn main() {
    common::run_with_json("test_ui.bui.json");
}
