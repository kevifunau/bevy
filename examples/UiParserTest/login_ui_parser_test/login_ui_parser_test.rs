//! Renders the login screen BUI JSON test case.
//!
//! Run with:
//! `cargo run --example login_ui_parser_test`

#[path = "../common.rs"]
mod common;

fn main() {
    common::run_with_json("login_ui.json");
}
