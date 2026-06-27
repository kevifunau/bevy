//! Compiles an OpenDesign HTML artifact into BUI JSON without starting Bevy.
//!
//! Run with:
//! `cargo run --example compile_opendesign_html -- <input.html> <output.json> [manifest.json]`

use std::{env, fs, process::ExitCode};

use bevy_ai_ui_parser::{
    opendesign_html_file_to_bui_json, opendesign_html_file_to_bui_json_with_manifest,
};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(input_path) = args.next() else {
        eprintln!(
            "Usage: cargo run --example compile_opendesign_html -- <input.html> <output.json>"
        );
        return ExitCode::FAILURE;
    };
    let Some(output_path) = args.next() else {
        eprintln!(
            "Usage: cargo run --example compile_opendesign_html -- <input.html> <output.json> [manifest.json]"
        );
        return ExitCode::FAILURE;
    };
    let manifest_path = args.next();

    let result = if let Some(manifest_path) = manifest_path.as_deref() {
        opendesign_html_file_to_bui_json_with_manifest(&input_path, Some(manifest_path))
    } else {
        opendesign_html_file_to_bui_json(&input_path)
    };

    match result {
        Ok(json) => match fs::write(&output_path, json) {
            Ok(()) => {
                println!("Compiled OpenDesign HTML '{input_path}' to BUI JSON '{output_path}'.");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Failed to write BUI JSON '{output_path}': {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("Failed to compile OpenDesign HTML: {error}");
            ExitCode::FAILURE
        }
    }
}
