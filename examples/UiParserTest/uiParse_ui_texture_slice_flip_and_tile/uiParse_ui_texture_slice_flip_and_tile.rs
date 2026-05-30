//! Renders the official `examples/ui/images/ui_texture_slice_flip_and_tile.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_ui_texture_slice_flip_and_tile`

#[path = "../common.rs"]
mod common;

fn main() {
    common::run_with_json("ui_texture_slice_flip_and_tile.json");
}
