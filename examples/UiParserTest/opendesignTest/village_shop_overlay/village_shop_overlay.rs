//! Renders the village shop overlay OpenDesign BUI IR test case.
//!
//! This example loads the checked-in `3.0-ir` snapshot generated from the
//! OpenDesign HTML village-shop-overlay design and renders it as a native Bevy
//! UI scene. Run with:
//! `cargo run --example village_shop_overlay`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_bui_file_without_button_feedback(
        "opendesignTest/village_shop_overlay/village-shop-overlay.ir.json",
    );
}
