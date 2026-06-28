use std::collections::HashMap;

use crate::core::{
    model::{bui_node, BuiDocument, BuiNode, BuiStageFitSemantics},
    opendesign::{
        dom::has_class,
        hero::enhance_hero_game_ui_defaults,
        preset::{apply_opendesign_preset, OpenDesignPreset},
        stylesheet::OpenDesignStylesheet,
        svg::SvgAssetEntry,
    },
    parse::validate::validate_bui_document,
    style::css_gradients::preserve_radial_circle_geometry,
    support::viewport::current_opendesign_viewport,
};

use super::tree::{generic_append_children, generic_element_node};

pub(crate) fn opendesign_html_to_generic_bui_document(
    stylesheet: &OpenDesignStylesheet,
    overlay: roxmltree::Node<'_, '_>,
    svg_assets: &mut Vec<SvgAssetEntry>,
) -> Result<BuiDocument, String> {
    let mut id_counts = HashMap::new();
    let is_stage_root = has_class(overlay, "game-stage")
        || has_class(overlay, "bevy-ui-root")
        || has_class(overlay, "stage");
    let mut source_root = generic_element_node("overlay_root", "node", stylesheet, overlay);
    apply_opendesign_preset(
        &mut source_root,
        if is_stage_root {
            OpenDesignPreset::GameStageRoot
        } else {
            OpenDesignPreset::OverlayRoot
        },
    );
    generic_append_children(
        &mut source_root,
        overlay,
        stylesheet,
        &mut id_counts,
        svg_assets,
    );

    let mut root = if is_stage_root {
        responsive_stage_document_root(stylesheet, overlay, source_root)
    } else {
        source_root
    };
    enhance_hero_game_ui_defaults(&mut root);
    preserve_radial_circle_geometry(&mut root);

    let document = BuiDocument {
        version: "3.0-ir".to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        imports: Vec::new(),
        state_model: crate::core::model::BuiStateModel::default(),
        interaction_model: crate::core::model::BuiInteractionModel::default(),
        resources: crate::core::model::BuiResources::default(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}

fn responsive_stage_document_root(
    stylesheet: &OpenDesignStylesheet,
    overlay: roxmltree::Node<'_, '_>,
    mut stage: BuiNode,
) -> BuiNode {
    let mut root = bui_node("overlay_root", "node");
    apply_opendesign_preset(&mut root, OpenDesignPreset::ViewportRoot);
    apply_viewport_background(stylesheet, overlay, &mut root);

    stage.id = overlay
        .attribute("id")
        .filter(|id| !id.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "stage_root".to_string());
    normalize_stage_viewport_sizing(&mut stage);
    root.children.push(stage);
    root
}

fn normalize_stage_viewport_sizing(stage: &mut BuiNode) {
    let design_width = stage
        .layout
        .styles
        .width
        .as_deref()
        .and_then(css_design_width_value);
    let design_height = stage
        .layout
        .styles
        .height
        .as_deref()
        .and_then(css_design_height_value)
        .or_else(|| {
            design_width
                .zip(
                    stage
                        .layout
                        .styles
                        .aspect_ratio
                        .as_deref()?
                        .parse::<f32>()
                        .ok(),
                )
                .and_then(|(width, ratio)| {
                    if ratio > 0.0 {
                        Some(width / ratio)
                    } else {
                        None
                    }
                })
        });

    if let (Some(width), Some(height)) = (design_width, design_height) {
        let mode = stage
            .markers
            .iter()
            .find_map(|marker| marker.strip_prefix("bui-stage-fit-mode:"))
            .unwrap_or("scale-down")
            .to_string();
        stage.markers.push(format!(
            "bui-stage-fit:{}x{}",
            format_number(width),
            format_number(height)
        ));
        stage.semantics.stage_fit = Some(BuiStageFitSemantics {
            design_width: width,
            design_height: height,
            mode,
        });
        stage.layout.styles.width = Some(format!("{}px", format_number(width)));
        stage.layout.styles.height = Some(format!("{}px", format_number(height)));
        stage.layout.styles.max_width = None;
        stage.layout.styles.max_height = None;
        stage.layout.styles.aspect_ratio = None;
    }
}

fn css_design_width_value(value: &str) -> Option<f32> {
    css_design_dimension_value(value, current_opendesign_viewport().width)
}

fn css_design_height_value(value: &str) -> Option<f32> {
    css_design_dimension_value(value, current_opendesign_viewport().height)
}

fn css_design_dimension_value(value: &str, viewport_axis: f32) -> Option<f32> {
    let value = value.trim();
    if let Some(px) = value.strip_suffix("px") {
        return px.parse::<f32>().ok();
    }
    if let Some(vw) = value.strip_suffix("vw") {
        return vw
            .parse::<f32>()
            .ok()
            .map(|number| current_opendesign_viewport().width * number / 100.0);
    }
    if let Some(vh) = value.strip_suffix("vh") {
        return vh
            .parse::<f32>()
            .ok()
            .map(|number| current_opendesign_viewport().height * number / 100.0);
    }
    value.parse::<f32>().ok().or_else(|| {
        value
            .strip_suffix('%')?
            .parse::<f32>()
            .ok()
            .map(|percent| viewport_axis * percent / 100.0)
    })
}

fn format_number(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        format!("{}", value as i32)
    } else {
        let formatted = format!("{value:.2}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn apply_viewport_background(
    stylesheet: &OpenDesignStylesheet,
    overlay: roxmltree::Node<'_, '_>,
    root: &mut BuiNode,
) {
    for tag in ["html", "body"] {
        for (name, value) in stylesheet.tag_declarations(tag) {
            if !matches!(
                name.as_str(),
                "background" | "background-color" | "background-image"
            ) {
                continue;
            }
            let value = stylesheet.resolve_value(value);
            crate::core::style::css_apply::apply_opendesign_declaration(root, name, &value);
        }
    }

    for ancestor in overlay.ancestors().filter(|node| node.is_element()) {
        let tag = ancestor.tag_name().name();
        if tag != "html" && tag != "body" {
            continue;
        }

        for (name, value) in stylesheet.matching_declarations(ancestor) {
            if matches!(
                name.as_str(),
                "background" | "background-color" | "background-image"
            ) {
                let value = stylesheet.resolve_value(value);
                crate::core::style::css_apply::apply_opendesign_declaration(root, name, &value);
            }
        }
    }

    root.layout.styles.width = Some("100%".to_string());
    root.layout.styles.height = Some("100%".to_string());
    root.layout.styles.justify_content = Some("center".to_string());
    root.layout.styles.align_items = Some("center".to_string());
    root.layout.styles.overflow = Some("clip".to_string());
}
