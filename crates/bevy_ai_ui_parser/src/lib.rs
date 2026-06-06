//! A JSON-driven UI parser plugin for Bevy.

mod core;

pub use core::{
    AiUiPlugin,
    BuiId,
    BuiLogicTags,
    BuiRootEntity,
    BuiTextInput,
    opendesign_html_file_to_bui_ir_json,
    opendesign_html_file_to_bui_json,
    opendesign_html_to_bui_ir_json_str,
    opendesign_html_to_bui_json_str,
    validate_bui_ir_json_file,
    validate_bui_ir_json_str,
    validate_bui_json_file,
    validate_bui_json_str,
};
