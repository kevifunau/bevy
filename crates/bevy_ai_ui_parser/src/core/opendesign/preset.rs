use crate::core::model::BuiNode;

#[derive(Debug, Clone, Copy)]
pub(crate) enum OpenDesignPreset {
    OverlayRoot,
    GameStageRoot,
    Panel,
    PanelHeader,
    TitleBoard,
    CloseButton,
    ShopBody,
    ShopScroll,
    ShopCard,
    ItemMain,
    AssetStack,
    AssetSlot,
    Stars,
    ItemCopy,
    ItemBonus,
    Purchase,
    PriceTag,
    PriceCoin,
    BuyButton,
    FootHint,
}

pub(crate) fn apply_opendesign_preset(node: &mut BuiNode, preset: OpenDesignPreset) {
    match preset {
        OpenDesignPreset::OverlayRoot => {
            node.layout.styles.width = Some("100%".to_string());
            node.layout.styles.height = Some("100%".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.style.visuals.background_color = Some("#3B281862".to_string());
        }
        OpenDesignPreset::GameStageRoot => {
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.style.visuals.background_color = Some("#3B281862".to_string());
        }
        OpenDesignPreset::Panel => {
            node.layout.styles.width = Some("92%".to_string());
            node.layout.styles.height = Some("90%".to_string());
            node.layout.styles.max_width = Some("720px".to_string());
            node.layout.styles.max_height = Some("860px".to_string());
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.padding = Some("0px".to_string());
            node.style.visuals.background_color = Some("#F8ECD0".to_string());
            node.style.visuals.border_color = Some("#8B5F33".to_string());
            node.style.visuals.border_width = Some("4px".to_string());
            node.style.visuals.border_radius = Some("28px".to_string());
        }
        OpenDesignPreset::PanelHeader => {
            node.layout.styles.flex_direction = Some("row".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.padding = Some("28px 64px 18px 64px".to_string());
        }
        OpenDesignPreset::TitleBoard => {
            node.layout.styles.min_width = Some("220px".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.padding = Some("14px 32px 16px 32px".to_string());
            node.style.visuals.background_color = Some("#8B5F33".to_string());
            node.style.visuals.border_width = Some("3px".to_string());
            node.style.visuals.border_color = Some("#3B2818D8".to_string());
            node.style.visuals.border_radius = Some("18px".to_string());
        }
        OpenDesignPreset::CloseButton => {
            node.layout.styles.width = Some("48px".to_string());
            node.layout.styles.height = Some("48px".to_string());
            node.layout.styles.position_type = Some("absolute".to_string());
            node.layout.styles.top = Some("18px".to_string());
            node.layout.styles.right = Some("18px".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.style.visuals.background_color = Some("#CC4D3F".to_string());
            node.style.visuals.border_width = Some("0px".to_string());
            node.style.visuals.border_color = Some("transparent".to_string());
            node.style.visuals.border_radius = Some("48px".to_string());
        }
        OpenDesignPreset::ShopBody => {
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.padding = Some("0px 16px 18px 16px".to_string());
            node.layout.styles.flex_grow = Some("1".to_string());
        }
        OpenDesignPreset::ShopScroll => {
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.overflow = Some("scroll_y".to_string());
            node.layout.styles.padding = Some("8px 6px 8px 2px".to_string());
            node.layout.styles.row_gap = Some("14px".to_string());
            node.layout.styles.max_height = Some("560px".to_string());
        }
        OpenDesignPreset::ShopCard => {
            node.layout.styles.display = Some("grid".to_string());
            node.layout.styles.grid_template_columns = Some("flex(1) auto".to_string());
            node.layout.styles.align_items = Some("stretch".to_string());
            node.layout.styles.padding = Some("14px".to_string());
            node.style.visuals.background_color = Some("#F8ECD0".to_string());
            node.style.visuals.border_width = Some("2px".to_string());
            node.style.visuals.border_color = Some("#8B5F33E6".to_string());
            node.style.visuals.border_radius = Some("20px".to_string());
        }
        OpenDesignPreset::ItemMain => {
            node.layout.styles.display = Some("grid".to_string());
            node.layout.styles.grid_template_columns = Some("px(92) flex(1)".to_string());
            node.layout.styles.flex_grow = Some("1".to_string());
        }
        OpenDesignPreset::AssetStack => {
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.row_gap = Some("10px".to_string());
            node.layout.styles.align_items = Some("stretch".to_string());
            node.layout.styles.width = Some("92px".to_string());
        }
        OpenDesignPreset::AssetSlot => {
            node.layout.styles.min_height = Some("92px".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.padding = Some("10px".to_string());
            node.style.visuals.background_color = Some("#F8ECD0".to_string());
            node.style.visuals.border_width = Some("2px".to_string());
            node.style.visuals.border_color = Some("#8B5F33D2".to_string());
            node.style.visuals.border_radius = Some("18px".to_string());
        }
        OpenDesignPreset::Stars => {
            node.layout.styles.flex_direction = Some("row".to_string());
            node.layout.styles.column_gap = Some("6px".to_string());
            node.layout.styles.justify_content = Some("space_evenly".to_string());
            node.layout.styles.min_height = Some("24px".to_string());
            node.layout.styles.padding = Some("0px 2px".to_string());
        }
        OpenDesignPreset::ItemCopy => {
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.row_gap = Some("6px".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.flex_grow = Some("1".to_string());
            node.layout.styles.min_width = Some("0px".to_string());
        }
        OpenDesignPreset::ItemBonus => {
            node.layout.styles.flex_direction = Some("row".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.column_gap = Some("6px".to_string());
            node.layout.styles.padding = Some("6px 10px".to_string());
            node.style.visuals.background_color = Some("#D89A1F2E".to_string());
            node.style.visuals.border_radius = Some("48px".to_string());
        }
        OpenDesignPreset::Purchase => {
            node.layout.styles.flex_direction = Some("column".to_string());
            node.layout.styles.justify_content = Some("space_between".to_string());
            node.layout.styles.align_items = Some("flex_end".to_string());
            node.layout.styles.min_width = Some("120px".to_string());
            node.layout.styles.width = Some("140px".to_string());
            node.layout.styles.row_gap = Some("12px".to_string());
        }
        OpenDesignPreset::PriceTag => {
            node.layout.styles.flex_direction = Some("row".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.column_gap = Some("6px".to_string());
            node.layout.styles.padding = Some("8px 12px".to_string());
            node.style.visuals.background_color = Some("#FFFFFF2E".to_string());
            node.style.visuals.border_width = Some("2px".to_string());
            node.style.visuals.border_color = Some("#8B5F33DE".to_string());
            node.style.visuals.border_radius = Some("14px".to_string());
        }
        OpenDesignPreset::PriceCoin => {
            node.layout.styles.width = Some("16px".to_string());
            node.layout.styles.height = Some("16px".to_string());
            node.style.visuals.background_color = Some("#D89A1F".to_string());
            node.style.visuals.border_width = Some("1px".to_string());
            node.style.visuals.border_color = Some("#3B28189C".to_string());
            node.style.visuals.border_radius = Some("16px".to_string());
        }
        OpenDesignPreset::BuyButton => {
            node.layout.styles.min_width = Some("112px".to_string());
            node.layout.styles.min_height = Some("48px".to_string());
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.align_items = Some("center".to_string());
            node.layout.styles.padding = Some("0px 20px".to_string());
            node.style.visuals.background_color = Some("#3FB45A".to_string());
            node.style.visuals.border_width = Some("0px".to_string());
            node.style.visuals.border_color = Some("transparent".to_string());
            node.style.visuals.border_radius = Some("18px".to_string());
        }
        OpenDesignPreset::FootHint => {
            node.layout.styles.justify_content = Some("center".to_string());
            node.layout.styles.padding = Some("6px 18px 18px 18px".to_string());
        }
    }
}