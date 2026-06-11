//! Renders the pure HTML/CSS text baseline directly from the OpenDesign HTML artifact.
//!
//! Run with:
//! `cargo run --example bevy_ui_text_baseline_html`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_html_without_button_feedback("opendesignTest/bevy_ui_text_baseline/index.html");
}
