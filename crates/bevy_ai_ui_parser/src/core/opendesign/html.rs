mod village;

use crate::core::{
    model::BuiDocument,
    opendesign::{
        dom::has_class, generic::opendesign_html_to_generic_bui_document,
        stylesheet::OpenDesignStylesheet,
    },
    parse::validate::validate_bui_document,
    support::viewport::OpenDesignViewport,
};

#[derive(Clone, Copy)]
struct OpenDesignRootNodes<'a, 'input> {
    root: roxmltree::Node<'a, 'input>,
    overlay: Option<roxmltree::Node<'a, 'input>>,
}

pub(crate) fn extract_opendesign_fragment(html: &str) -> Result<&str, String> {
    let overlay_start = html.find("<div class=\"overlay");
    let main_start = html.find("<main class=\"game-stage");
    let bevy_ui_root_div_start = html.find("<div class=\"bevy-ui-root");
    let bevy_ui_root_main_start = html.find("<main class=\"bevy-ui-root");

    let start = overlay_start
        .or(main_start)
        .or(bevy_ui_root_div_start)
        .or(bevy_ui_root_main_start)
        .ok_or_else(|| "OpenDesign HTML does not contain a recognized root container ('<div class=\"overlay', '<main class=\"game-stage', or class 'bevy-ui-root').".to_string())?;

    let visually_hidden_end = html[start..]
        .find("<p class=\"visually-hidden\"")
        .map(|offset| start + offset);

    let closing_main_end = html[start..]
        .find("</main>")
        .map(|offset| start + offset + "</main>".len());
    let closing_bevy_root_end = if bevy_ui_root_div_start == Some(start) {
        html.rfind("</div>").map(|offset| offset + "</div>".len())
    } else if bevy_ui_root_main_start == Some(start) {
        html[start..]
            .find("</main>")
            .map(|offset| start + offset + "</main>".len())
    } else {
        None
    };

    let end = visually_hidden_end
        .or(closing_main_end)
        .or(closing_bevy_root_end)
        .ok_or_else(|| {
            "OpenDesign HTML does not contain the expected closing marker after the root container."
                .to_string()
        })?;

    Ok(html[start..end].trim())
}

pub(crate) fn opendesign_compile_viewport(
    root_node: roxmltree::Node<'_, '_>,
) -> OpenDesignViewport {
    let is_hero_game_ui = has_class(root_node, "game-stage")
        && root_node
            .descendants()
            .any(|node| has_class(node, "hero-zone"))
        && root_node
            .descendants()
            .any(|node| has_class(node, "info-panel"))
        && root_node
            .descendants()
            .any(|node| has_class(node, "name-card"));

    if is_hero_game_ui {
        OpenDesignViewport::hero_game_ui_compile()
    } else {
        OpenDesignViewport::DEFAULT
    }
}

fn find_opendesign_root_nodes<'a, 'input>(
    parsed: &'a roxmltree::Document<'input>,
) -> Result<OpenDesignRootNodes<'a, 'input>, String> {
    let overlay = parsed
        .descendants()
        .find(|node| has_class(*node, "overlay"));

    let root = overlay
        .or_else(|| {
            parsed
                .descendants()
                .find(|node| has_class(*node, "game-stage"))
        })
        .or_else(|| {
            parsed
                .descendants()
                .find(|node| has_class(*node, "bevy-ui-root"))
        })
        .ok_or_else(|| {
            "OpenDesign HTML is missing a recognized root container (.overlay, .game-stage, or .bevy-ui-root)."
                .to_string()
        })?;

    Ok(OpenDesignRootNodes { root, overlay })
}

pub(crate) fn opendesign_html_to_bui_document(html: &str) -> Result<BuiDocument, String> {
    let fragment = extract_opendesign_fragment(html)?;
    let wrapped_storage = format!("<bui_root>{fragment}</bui_root>");
    let parsed = roxmltree::Document::parse(&wrapped_storage)
        .map_err(|error| format!("Failed to parse OpenDesign HTML fragment: {error}"))?;
    let root_nodes = find_opendesign_root_nodes(&parsed)?;
    let viewport = opendesign_compile_viewport(root_nodes.root);

    crate::core::support::viewport::with_opendesign_viewport(viewport, || {
        let stylesheet = OpenDesignStylesheet::parse(html);

        if root_nodes.overlay.is_none() {
            return opendesign_html_to_generic_bui_document(&stylesheet, root_nodes.root);
        }

        match village::compile_village_shop_overlay_document(&stylesheet, root_nodes.root) {
            Ok(root) => finalize_document(root),
            Err(_) => opendesign_html_to_generic_bui_document(&stylesheet, root_nodes.root),
        }
    })
}

fn finalize_document(root: crate::core::model::BuiNode) -> Result<BuiDocument, String> {
    let document = BuiDocument {
        version: "3.0-ir".to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        imports: Vec::new(),
        state_model: crate::core::model::BuiStateModel::default(),
        resources: crate::core::model::BuiResources::default(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}
