use crate::core::model::ir::BuiNode;
use crate::core::style::css_gradients::{
    css_simple_linear_gradient_bands, SimpleGradientOverlayDirection,
};

pub(super) fn css_simple_linear_gradient_overlay(
    layer: &str,
) -> Option<(
    SimpleGradientOverlayDirection,
    Option<f32>,
    String,
    f32,
    f32,
)> {
    let (direction, diagonal_angle, bands) = css_simple_linear_gradient_bands(layer)?;
    let band = bands.into_iter().next()?;
    Some((
        direction,
        diagonal_angle,
        band.color,
        band.start_ratio,
        band.end_ratio,
    ))
}

pub(super) const VILLAGE_SHOP_HTML: &str = include_str!(
    "../../../../examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.html"
);
pub(super) const VILLAGE_SHOP_IR: &str = include_str!(
    "../../../../examples/UiParserTest/opendesignTest/village_shop_overlay/village-shop-overlay.ir.json"
);
pub(super) const QUEST_NOTICE_HTML: &str = include_str!(
    "../../../../examples/UiParserTest/opendesignTest/quest_notice_overlay/quest-notice-overlay.html"
);
pub(super) const HERO_GAME_UI_HTML: &str =
    include_str!("../../../../examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.html");
pub(super) const HERO_GAME_UI_IR: &str = include_str!(
    "../../../../examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.ir.json"
);
pub(super) const BEVY_UI_EXAMPLE_HTML: &str = include_str!(
    "../../../../examples/UiParserTest/opendesignTest/bevy_ui_example/bevy-ui-example.html"
);

pub(super) fn find_bui_node<'a>(node: &'a BuiNode, id: &str) -> &'a BuiNode {
    find_bui_node_optional(node, id).unwrap_or_else(|| panic!("IR node '{id}' should exist"))
}

pub(super) fn find_bui_node_optional<'a>(node: &'a BuiNode, id: &str) -> Option<&'a BuiNode> {
    if node.id == id {
        return Some(node);
    }

    node.children
        .iter()
        .find_map(|child| find_bui_node_optional(child, id))
}
