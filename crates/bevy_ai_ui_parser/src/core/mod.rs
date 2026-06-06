pub(crate) mod legacy;

pub mod api;
pub mod interaction;
pub mod model;
pub mod opendesign;
pub mod parse;
pub mod runtime;
pub mod style;
pub mod support;

#[cfg(test)]
#[path = "../tests/mod.rs"]
mod tests;

pub use api::{
    opendesign_html_file_to_bui_ir_json,
    opendesign_html_file_to_bui_json,
    opendesign_html_to_bui_ir_json_str,
    opendesign_html_to_bui_json_str,
    validate_bui_ir_json_file,
    validate_bui_ir_json_str,
    validate_bui_json_file,
    validate_bui_json_str,
};
pub use legacy::{
    BuiId,
    BuiLogicTags,
    BuiRootEntity,
    BuiTextInput,
};
pub use runtime::plugin::AiUiPlugin;
