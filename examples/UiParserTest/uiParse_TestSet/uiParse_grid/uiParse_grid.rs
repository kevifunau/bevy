//! Renders the official `examples/ui/layout/grid.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_grid`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json("grid.ir.json");
}
