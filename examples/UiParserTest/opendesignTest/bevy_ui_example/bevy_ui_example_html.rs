//! Renders the Bevy UI example directly from the OpenDesign HTML artifact.
//!
//! Run with:
//! `cargo run --example bevy_ui_example_html`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_html_without_button_feedback(
        "opendesignTest/bevy_ui_example/bevy-ui-example.html",
    );
}
