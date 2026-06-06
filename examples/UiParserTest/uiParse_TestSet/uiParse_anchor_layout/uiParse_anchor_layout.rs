//! Renders the official `examples/ui/layout/anchor_layout.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_anchor_layout`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json("anchor_layout.json");
}
