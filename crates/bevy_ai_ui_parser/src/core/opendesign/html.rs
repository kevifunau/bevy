mod village;

use crate::core::{
    model::BuiDocument,
    opendesign::{
        dom::has_class,
        generic::opendesign_html_to_generic_bui_document,
        stylesheet::OpenDesignStylesheet,
    },
    parse::validate::{EXPECTED_VERSION, validate_bui_document},
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

    let start = overlay_start
        .or(main_start)
        .ok_or_else(|| "OpenDesign HTML does not contain a recognized root container ('<div class=\"overlay' or '<main class=\"game-stage').".to_string())?;

    let visually_hidden_end = html[start..]
        .find("<p class=\"visually-hidden\"")
        .map(|offset| start + offset);

    let closing_main_end = html[start..]
        .find("</main>")
        .map(|offset| start + offset + "</main>".len());

    let end = visually_hidden_end.or(closing_main_end).ok_or_else(|| {
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
        .ok_or_else(|| {
            "OpenDesign HTML is missing a recognized root container (.overlay or .game-stage)."
                .to_string()
        })?;

    Ok(OpenDesignRootNodes { root, overlay })
}

pub(crate) fn opendesign_html_to_bui_document(html: &str) -> Result<BuiDocument, String> {
    let fragment = extract_opendesign_fragment(html)?;
    let wrapped = format!("<bui_root>{fragment}</bui_root>");
    let parsed = roxmltree::Document::parse(&wrapped)
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
        version: EXPECTED_VERSION.to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}
