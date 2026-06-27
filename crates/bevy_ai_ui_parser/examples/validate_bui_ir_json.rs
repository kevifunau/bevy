//! Validates a BUI 3.0-ir JSON file without starting Bevy.
//!
//! Run with:
//! `cargo run --example validate_bui_ir_json -- examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.ir.json`

use std::{env, process::ExitCode};

use bevy_ai_ui_parser::validate_bui_ir_json_file;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("Usage: cargo run --example validate_bui_ir_json -- <path-to-bui-ir-json>");
        return ExitCode::FAILURE;
    };

    match validate_bui_ir_json_file(&path) {
        Ok(()) => {
            println!("BUI IR JSON is valid: {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("BUI IR JSON is invalid: {error}");
            ExitCode::FAILURE
        }
    }
}
