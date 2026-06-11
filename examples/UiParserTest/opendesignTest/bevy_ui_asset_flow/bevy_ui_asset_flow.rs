//! Renders the Bevy UI asset-flow example from the checked-in `3.0-ir` snapshot.
//!
//! Run with:
//! `cargo run --example bevy_ui_asset_flow`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_bui_file_without_button_feedback(
        "opendesignTest/bevy_ui_asset_flow/bevy_ui_asset_flow.ir.json",
    );
}
