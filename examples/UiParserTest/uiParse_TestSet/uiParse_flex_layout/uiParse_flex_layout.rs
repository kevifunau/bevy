//! Renders the official `examples/ui/layout/flex_layout.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_flex_layout`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json("flex_layout.json");
}
