//! Renders the quest notice overlay directly from the OpenDesign HTML artifact.
//!
//! Run with:
//! `cargo run --example quest_notice_overlay_html`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_html_without_button_feedback(
        "opendesignTest/quest_notice_overlay/quest-notice-overlay.html",
    );
}
