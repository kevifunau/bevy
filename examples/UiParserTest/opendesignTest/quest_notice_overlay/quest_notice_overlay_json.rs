//! Renders the quest notice overlay from the checked-in 2.x compatibility JSON.
//!
//! Run with:
//! `cargo run --example quest_notice_overlay_json`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json_without_button_feedback(
        "opendesignTest/quest_notice_overlay/quest-notice-overlay.json",
    );
}
