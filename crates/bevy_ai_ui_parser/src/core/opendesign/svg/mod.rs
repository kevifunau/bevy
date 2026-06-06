mod render;
mod semantic;
mod shape;

pub(crate) use render::{
    ensure_text_icon_child, is_decorative_icon_helper_node, is_svg_tag, svg_fallback_text_node,
};
#[cfg(test)]
pub(crate) use semantic::semantic_svg_fallback_spec;
#[cfg(test)]
pub(crate) use shape::svg_fallback_icon;
