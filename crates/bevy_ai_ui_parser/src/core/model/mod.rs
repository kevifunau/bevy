//! Core BUI/IR data model modules.

pub mod ir;
pub mod style;
pub mod visual;

pub use ir::{
    BuiActionBinding, BuiBinding, BuiContent, BuiDocument, BuiInteractionModel,
    BuiInteractionStep, BuiLayout, BuiNode, BuiNodeType, BuiResources, BuiScrollViewSemantics,
    BuiSemantics, BuiSliderSemantics, BuiStageFitSemantics, BuiStateModel, BuiStateVisual,
    BuiStyle,
};
pub(crate) use ir::{bui_node, ensure_state_visual, text_node, kind_to_node_type};
pub use style::BuiStyles;
pub use visual::{
    BuiBoxShadowConfig, BuiImageConfig, BuiTextConfig, BuiTextShadowConfig,
    BuiTextureAtlasConfig, BuiTextureSlicerConfig, BuiVisuals,
};
pub(crate) use visual::BuiBackgroundImageLayout;
