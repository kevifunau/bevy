//! Renders the hero game UI directly from the OpenDesign HTML artifact.
//!
//! Run with:
//! `cargo run --example slg_ui_html`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_html_without_button_feedback("opendesignTest/slg_ui/index.html");
}
