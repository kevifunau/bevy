//! Renders the official `examples/ui/layout/z_index.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_z_index`

#[path = "../common.rs"]
mod common;

fn main() {
    common::run_with_json("z_index.json");
}
