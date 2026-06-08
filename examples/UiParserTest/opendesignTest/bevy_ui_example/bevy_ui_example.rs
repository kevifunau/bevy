//! Renders the Bevy UI example from the checked-in `3.0-ir` snapshot.
//!
//! Run with:
//! `cargo run --example bevy_ui_example`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_bui_file_without_button_feedback(
        "opendesignTest/bevy_ui_example/bevy-ui-example.ir.json",
    );
}
