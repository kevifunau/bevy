//! Core BUI/IR data model modules.

pub mod ir;
pub mod style;
pub mod visual;

pub(crate) use ir::{
    bui_node,
    ensure_state_visual,
    text_node,
    BuiActionBinding,
    BuiBinding,
    BuiDocument,
    BuiNode,
    BuiNodeType,
    BuiResources,
    BuiStateModel,
    BuiStateVisual,
};
pub(crate) use style::BuiStyles;
pub(crate) use visual::{
    BuiBackgroundImageLayout,
    BuiBoxShadowConfig,
    BuiImageConfig,
    BuiTextConfig,
    BuiTextShadowConfig,
    BuiTextureAtlasConfig,
    BuiTextureSlicerConfig,
    BuiVisuals,
};