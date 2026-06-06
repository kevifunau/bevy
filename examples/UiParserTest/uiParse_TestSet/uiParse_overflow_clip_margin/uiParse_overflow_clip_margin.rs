//! Renders the official `examples/ui/scroll_and_overflow/overflow_clip_margin.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_overflow_clip_margin`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json("overflow_clip_margin.ir.json");
}
