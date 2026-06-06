//! Legacy compatibility re-exports during the structural split.

pub use super::interaction::components::{
    BuiDisabled,
    BuiTextInput,
    BuiToggle,
    BuiVisualState,
};
pub(crate) use super::interaction::components::{
    BuiListDefinition,
    BuiProgressFill,
    BuiProgressGroup,
    BuiTabGroupDefinition,
    BuiTabItem,
    BuiTextInputMirror,
    BuiTextInputProxy,
    BuiVisualStateDefinitions,
    PendingUiTargetCamera,
};
pub use super::runtime::components::{BuiId, BuiLogicTags, BuiRootEntity};
pub(crate) use super::runtime::components::{BuiActions, BuiBindings};
