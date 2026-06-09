//! A JSON-driven UI parser plugin for Bevy.

mod core;

pub use core::{
    opendesign_html_file_to_bui_json, opendesign_html_to_bui_json_str, validate_bui_json_file,
    validate_bui_json_str, AiUiPlugin, BuiActionTrigger, BuiActionTriggered, BuiBindingValue,
    BuiDisabled, BuiId, BuiLogicTags, BuiRootEntity, BuiStateSet, BuiStateStore, BuiTextInput,
    BuiToggle, BuiVisualState,
};
