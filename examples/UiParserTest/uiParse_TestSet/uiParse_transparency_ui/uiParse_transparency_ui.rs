//! Renders the official `examples/ui/styling/transparency_ui.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_transparency_ui`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json("transparency_ui.ir.json");
}
