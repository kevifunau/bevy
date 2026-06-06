//! Renders the official `examples/ui/images/image_node.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_image_node`

#[path = "../../common.rs"]
mod common;

fn main() {
    common::run_with_json("image_node.ir.json");
}
