//! Compiles an OpenDesign HTML artifact into BUI 3.0-ir JSON without starting Bevy.
//!
//! Run with:
//! `cargo run --example compile_opendesign_ir -- <input.html> <output.ir.json>`

use std::{env, fs, process::ExitCode};

use bevy_ai_ui_parser::opendesign_html_file_to_bui_json;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(input_path) = args.next() else {
        eprintln!(
            "Usage: cargo run --example compile_opendesign_ir -- <input.html> <output.ir.json>"
        );
        return ExitCode::FAILURE;
    };
    let Some(output_path) = args.next() else {
        eprintln!(
            "Usage: cargo run --example compile_opendesign_ir -- <input.html> <output.ir.json>"
        );
        return ExitCode::FAILURE;
    };

    match opendesign_html_file_to_bui_json(&input_path) {
        Ok(json) => match fs::write(&output_path, json) {
            Ok(()) => {
                println!("Compiled OpenDesign HTML '{input_path}' to BUI IR '{output_path}'.");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Failed to write BUI IR '{output_path}': {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("Failed to compile OpenDesign HTML to BUI IR: {error}");
            ExitCode::FAILURE
        }
    }
}
