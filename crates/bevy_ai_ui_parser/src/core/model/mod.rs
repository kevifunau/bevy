//! Core BUI/IR data model modules.

pub mod bui;
pub mod ir;
pub mod style;
pub mod visual;

pub(crate) use bui::{
    kind_to_node_type,
    node_type_to_kind,
    BuiActionBinding,
    BuiBinding,
    BuiDocument,
    BuiNode,
    BuiNodeType,
    BuiStateVisual,
};
pub(crate) use ir::BuiIrDocument;
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
