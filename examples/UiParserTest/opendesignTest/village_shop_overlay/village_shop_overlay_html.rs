//! Renders the village shop overlay directly from the OpenDesign HTML artifact.
//!
//! This keeps the HTML compile path available for screenshot comparison against
//! the checked-in `3.0-ir` fixture. Run with:
//! `cargo run --example village_shop_overlay_html`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_html_without_button_feedback(
        "opendesignTest/village_shop_overlay/village-shop-overlay.html",
    );
}
