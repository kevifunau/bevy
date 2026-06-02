//! Renders the village shop overlay from the checked-in 2.x compatibility JSON.
//!
//! This keeps the compatibility snapshot path available for screenshot
//! comparison against the `3.0-ir` fixture path. Run with:
//! `cargo run --example village_shop_overlay_json`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json_without_button_feedback(
        "opendesignTest/village_shop_overlay/village-shop-overlay.json",
    );
}
