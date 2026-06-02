//! Renders the village shop overlay BUI JSON test case.
//!
//! This example converts the HTML village-shop-overlay design into a Bevy UI
//! scene using the BUI JSON 2.0 contract. Run with:
//! `cargo run --example village_shop_overlay`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json_without_button_feedback(
        "opendesignTest/village_shop_overlay/village-shop-overlay.json",
    );
}
