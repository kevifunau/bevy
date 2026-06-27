//! Validates a BUI IR JSON file without starting Bevy.
//!
//! Run with:
//! `cargo run -p bevy_ai_ui_parser --example validate_bui -- examples/opus48/Dev/action_arena/index.ir.json`

use std::{env, process::ExitCode};

use bevy_ai_ui_parser::validate_bui_json_file;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("Usage: cargo run -p bevy_ai_ui_parser --example validate_bui -- <path-to-bui-json>");
        return ExitCode::FAILURE;
    };

    match validate_bui_json_file(&path) {
        Ok(()) => {
            println!("BUI JSON is valid: {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("BUI JSON is invalid: {error}");
            ExitCode::FAILURE
        }
    }
}
