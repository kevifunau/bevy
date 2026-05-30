//! Validates a BUI JSON file without starting a Bevy app.
//!
//! Run with:
//! `cargo run --example validate_bui_json -- examples/UiParserTest/login_ui_parser_test/login_ui.json`

use std::{env, process::ExitCode};

use bevy_ai_ui_parser::validate_bui_json_file;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("Usage: cargo run --example validate_bui_json -- <path-to-bui-json>");
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
