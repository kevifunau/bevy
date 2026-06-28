//! Core BUI/IR data model modules.

pub mod ir;
pub mod style;
pub mod visual;

pub(crate) use ir::{bui_node, ensure_state_visual, kind_to_node_type, text_node};
pub use ir::{
    BuiActionBinding, BuiBinding, BuiContent, BuiDocument, BuiInteractionModel, BuiInteractionStep,
    BuiLayout, BuiNode, BuiNodeType, BuiResources, BuiScrollViewSemantics, BuiSemantics,
    BuiSliderSemantics, BuiStageFitSemantics, BuiStateModel, BuiStateVisual, BuiStyle,
};
pub use style::BuiStyles;
pub(crate) use visual::BuiBackgroundImageLayout;
pub use visual::{
    BuiBoxShadowConfig, BuiImageConfig, BuiTextConfig, BuiTextShadowConfig, BuiTextureAtlasConfig,
    BuiTextureSlicerConfig, BuiVisuals,
};
