//! Renders the quest notice overlay from the checked-in `3.0-ir` snapshot.
//!
//! Run with:
//! `cargo run --example quest_notice_overlay`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_bui_file_without_button_feedback(
        "opendesignTest/quest_notice_overlay/quest-notice-overlay.ir.json",
    );
}
