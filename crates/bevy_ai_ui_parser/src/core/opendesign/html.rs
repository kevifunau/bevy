mod village;

use crate::core::{
    model::{BuiDocument, BuiInteractionModel},
    opendesign::{
        dom::has_class,
        generic::opendesign_html_to_generic_bui_document,
        manifest::{apply_manifest_to_document, OpenDesignAssetManifest},
        stylesheet::OpenDesignStylesheet,
        svg::rasterize_svg_assets,
    },
    parse::validate::validate_bui_document,
    support::viewport::OpenDesignViewport,
};

const BUI_ACTION_SCRIPT_MARKERS: &[&str] = &[
    "window.BUI_ACTIONS",
    "globalThis.BUI_ACTIONS",
    "BUI_ACTIONS",
    "Bui.registerActions",
    "BUI.registerActions",
];

#[derive(Clone, Copy)]
struct OpenDesignRootNodes<'a, 'input> {
    root: roxmltree::Node<'a, 'input>,
    overlay: Option<roxmltree::Node<'a, 'input>>,
}

pub(crate) fn extract_opendesign_fragment(html: &str) -> Result<&str, String> {
    let overlay_start = find_element_with_class(html, "div", "overlay");
    let main_start = find_element_with_class(html, "main", "game-stage");
    let bevy_ui_root_div_start = find_element_with_class(html, "div", "bevy-ui-root");
    let bevy_ui_root_main_start = find_element_with_class(html, "main", "bevy-ui-root");
    let stage_main_start = find_element_with_class(html, "main", "stage");
    let stage_section_start = find_element_with_class(html, "section", "stage");
    let stage_div_start = find_element_with_class(html, "div", "stage");
    let page_div_start = find_element_with_class(html, "div", "page");

    let start = overlay_start
        .or(main_start)
        .or(bevy_ui_root_div_start)
        .or(bevy_ui_root_main_start)
        .or(stage_main_start)
        .or(stage_section_start)
        .or(stage_div_start)
        .or(page_div_start)
        .ok_or_else(|| "OpenDesign HTML does not contain a recognized root container ('<div class=\"overlay', '<main class=\"game-stage', '<main class=\"stage', '<section class=\"stage', '<div class=\"stage', '<div class=\"page', or class 'bevy-ui-root').".to_string())?;

    let visually_hidden_end = html[start..]
        .find("<p class=\"visually-hidden\"")
        .map(|offset| start + offset);

    let closing_main_end = html[start..]
        .find("</main>")
        .map(|offset| start + offset + "</main>".len());
    let closing_stage_section_end = if stage_section_start == Some(start) {
        html[start..]
            .find("</section>")
            .map(|offset| start + offset + "</section>".len())
    } else {
        None
    };
    let closing_bevy_root_end = if bevy_ui_root_div_start == Some(start) {
        find_matching_div_close(&html[start..]).map(|offset| start + offset)
    } else if bevy_ui_root_main_start == Some(start) {
        html[start..]
            .find("</main>")
            .map(|offset| start + offset + "</main>".len())
    } else {
        None
    };
    let closing_div_end = if page_div_start == Some(start)
        || stage_div_start == Some(start)
        || overlay_start == Some(start)
    {
        find_matching_div_close(&html[start..]).map(|offset| start + offset)
    } else {
        None
    };

    let end = visually_hidden_end
        .or(closing_main_end)
        .or(closing_stage_section_end)
        .or(closing_bevy_root_end)
        .or(closing_div_end)
        .ok_or_else(|| {
            "OpenDesign HTML does not contain the expected closing marker after the root container."
                .to_string()
        })?;

    Ok(html[start..end].trim())
}

fn find_element_with_class(html: &str, tag: &str, class_name: &str) -> Option<usize> {
    let exact = format!("<{tag} class=\"{class_name}\"");
    let prefix = format!("<{tag} class=\"{class_name} ");
    html.find(&exact).or(html.find(&prefix))
}

fn find_matching_div_close(fragment: &str) -> Option<usize> {
    let mut depth: u32 = 0;
    let mut pos: usize = 0;
    let bytes = fragment.as_bytes();
    while pos < bytes.len() {
        if bytes[pos..].starts_with(b"<div") {
            let abs_tag_end = bytes[pos..]
                .iter()
                .position(|b| *b == b'>')
                .map(|offset| pos + offset + 1);
            let tag_end = abs_tag_end.unwrap_or(bytes.len());
            let tag_str = &fragment[pos..fragment.len().min(tag_end)];
            if tag_str.ends_with("/>") {
                pos = tag_end;
                continue;
            }
            depth += 1;
            pos = tag_end;
        } else if bytes[pos..].starts_with(b"</div>") {
            depth -= 1;
            if depth == 0 {
                return Some(pos + "</div>".len());
            }
            pos += "</div>".len();
        } else {
            let ch = fragment[pos..].chars().next();
            pos += ch.map_or(1, |c| c.len_utf8());
        }
    }
    None
}

fn normalize_html_entities_for_xml(fragment: &str) -> String {
    // Strip all <script>...</script> tags before XML parsing.
    // The data-bui-actions JSON is already extracted separately via
    // parse_interaction_model(), so stripping script tags here is safe.
    // This prevents roxmltree from failing on JS content containing '<'
    // characters and boolean attributes like data-bui-actions.
    let fragment = strip_script_tags(fragment);

    let mut result = String::with_capacity(fragment.len());
    let mut chars = fragment.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '&' {
            let rest = &fragment[i + 1..];
            if rest.starts_with("nbsp;")
                || rest.starts_with("ensp;")
                || rest.starts_with("emsp;")
                || rest.starts_with("amp;")
                || rest.starts_with("lt;")
                || rest.starts_with("gt;")
                || rest.starts_with("quot;")
                || rest.starts_with("apos;")
                || rest.starts_with('#')
            {
                if rest.starts_with("nbsp;") {
                    result.push_str("&#160;");
                    for _ in 0.."nbsp;".len() {
                        chars.next();
                    }
                } else if rest.starts_with("ensp;") {
                    result.push_str("&#8194;");
                    for _ in 0.."ensp;".len() {
                        chars.next();
                    }
                } else if rest.starts_with("emsp;") {
                    result.push_str("&#8195;");
                    for _ in 0.."emsp;".len() {
                        chars.next();
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push_str("&amp;");
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Remove all `<script>...</script>` blocks from an HTML fragment.
/// Handles both `<script type="...">` and `<script>` (no attributes).
/// Self-closing `<script src="..."/>` and `<script src="..."></script>` are also removed.
fn strip_script_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let lower = html.to_ascii_lowercase();
    let mut pos = 0;

    while pos < html.len() {
        if let Some(start) = lower[pos..].find("<script") {
            let abs_start = pos + start;
            result.push_str(&html[pos..abs_start]);

            // Find the end of this script tag: either </script> or />
            if let Some(end) = find_script_end(&lower[abs_start..]) {
                pos = abs_start + end;
            } else {
                // No closing tag found, keep the rest as-is
                result.push_str(&html[abs_start..]);
                break;
            }
        } else {
            result.push_str(&html[pos..]);
            break;
        }
    }

    // Clean up multiple blank lines left by removed scripts
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result
}

/// Find the end position of a `<script>` block (including the closing `</script>` tag).
/// Returns the byte offset relative to the start of the input.
fn find_script_end(s: &str) -> Option<usize> {
    // Check for self-closing: <script ... />
    if let Some(close) = s.find("/>") {
        // Make sure this is the end of the opening tag
        if let Some(tag_end) = s.find('>') {
            if close < tag_end {
                return Some(close + 2);
            }
        }
    }

    // Find </script>
    s.find("</script>").map(|pos| pos + "</script>".len())
}

pub(crate) fn opendesign_compile_viewport(
    root_node: roxmltree::Node<'_, '_>,
) -> OpenDesignViewport {
    if has_class(root_node, "bevy-ui-root") {
        return OpenDesignViewport::bevy_ui_compile();
    }

    let is_hero_game_ui = is_hero_game_ui_root(root_node);

    if is_hero_game_ui {
        OpenDesignViewport::hero_game_ui_compile()
    } else {
        OpenDesignViewport::DEFAULT
    }
}

fn is_hero_game_ui_root(root_node: roxmltree::Node<'_, '_>) -> bool {
    if !has_class(root_node, "game-stage") {
        return false;
    }

    let has_hero_markers = root_node
        .descendants()
        .any(|node| has_class(node, "hero-zone"))
        && root_node
            .descendants()
            .any(|node| has_class(node, "info-panel"))
        && root_node
            .descendants()
            .any(|node| has_class(node, "name-card"));

    has_hero_markers
        || root_node.attribute("id") == Some("gameStage")
        || root_node
            .attribute("aria-label")
            .is_some_and(|label| label.contains("Olympia") && label.contains("英雄"))
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
        .or_else(|| parsed.descendants().find(|node| has_class(*node, "stage")))
        .or_else(|| parsed.descendants().find(|node| has_class(*node, "page")))
        .ok_or_else(|| {
            "OpenDesign HTML is missing a recognized root container (.overlay, .game-stage, .stage, .page, or .bevy-ui-root)."
                .to_string()
        })?;

    Ok(OpenDesignRootNodes { root, overlay })
}

pub(crate) fn opendesign_html_to_bui_document(html: &str) -> Result<BuiDocument, String> {
    opendesign_html_to_bui_document_with_manifest(html, None, None)
}

pub(crate) fn opendesign_html_to_bui_document_with_manifest(
    html: &str,
    manifest: Option<&OpenDesignAssetManifest>,
    base_dir: Option<&std::path::Path>,
) -> Result<BuiDocument, String> {
    let fragment = extract_opendesign_fragment(html)?;
    let normalized_fragment = normalize_html_entities_for_xml(fragment);
    let wrapped_storage = format!("<bui_root>{normalized_fragment}</bui_root>");
    let parsed = roxmltree::Document::parse(&wrapped_storage)
        .map_err(|error| format!("Failed to parse OpenDesign HTML fragment: {error}"))?;
    let root_nodes = find_opendesign_root_nodes(&parsed)?;
    let viewport = opendesign_compile_viewport(root_nodes.root);

    crate::core::support::viewport::with_opendesign_viewport(viewport, || {
        let stylesheet = OpenDesignStylesheet::parse(html);

        let mut svg_assets = Vec::new();

        let mut document = if root_nodes.overlay.is_none() {
            opendesign_html_to_generic_bui_document(&stylesheet, root_nodes.root, &mut svg_assets)?
        } else {
            match village::compile_village_shop_overlay_document(&stylesheet, root_nodes.root) {
                Ok(root) => finalize_document(root)?,
                Err(_) => opendesign_html_to_generic_bui_document(
                    &stylesheet,
                    root_nodes.root,
                    &mut svg_assets,
                )?,
            }
        };
        document.interaction_model = parse_interaction_model(html)?;

        if !svg_assets.is_empty()
            && let Some(dir) = base_dir
        {
            rasterize_svg_assets(&svg_assets, dir)?;
        }

        if let Some(manifest) = manifest {
            apply_manifest_to_document(&mut document, manifest, base_dir)?;
            validate_bui_document(&document)?;
        }

        Ok(document)
    })
}

fn parse_interaction_model(html: &str) -> Result<BuiInteractionModel, String> {
    if let Some(json) = extract_tag_body_with_marker(html, "data-bui-actions") {
        return serde_json::from_str(json.trim())
            .map_err(|error| format!("Failed to parse data-bui-actions JSON: {error}"));
    }

    for script in extract_script_bodies(html) {
        if let Some(json) = extract_bui_actions_from_javascript(script) {
            return serde_json::from_str(json.trim())
                .map_err(|error| format!("Failed to parse BUI actions from JavaScript: {error}"));
        }
    }

    Ok(BuiInteractionModel::default())
}

fn extract_tag_body_with_marker<'a>(html: &'a str, marker: &str) -> Option<&'a str> {
    extract_script_bodies(html)
        .into_iter()
        .find(|body| body_script_tag_contains_marker(html, body, marker))
}

fn extract_script_bodies<'a>(html: &'a str) -> Vec<&'a str> {
    let mut bodies = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<script") {
        rest = &rest[start..];
        let Some(tag_end) = rest.find('>') else {
            break;
        };
        let tag = &rest[..tag_end];
        let body_start = tag_end + 1;
        let Some(end) = rest[body_start..]
            .find("</script>")
            .map(|end| end + body_start)
        else {
            break;
        };
        if !tag.contains("src=") {
            bodies.push(&rest[body_start..end]);
        }
        rest = &rest[end + "</script>".len()..];
    }
    bodies
}

fn body_script_tag_contains_marker(html: &str, body: &str, marker: &str) -> bool {
    let Some(body_start) = html.find(body) else {
        return false;
    };
    let Some(script_start) = html[..body_start].rfind("<script") else {
        return false;
    };
    let tag = &html[script_start..body_start];
    tag.contains(marker)
}

fn extract_bui_actions_from_javascript(script: &str) -> Option<&str> {
    for marker in BUI_ACTION_SCRIPT_MARKERS {
        if let Some(marker_start) = script.find(marker) {
            let after_marker = &script[marker_start + marker.len()..];
            if marker.ends_with("registerActions") {
                if let Some(argument_start) = after_marker.find('(') {
                    let after_paren = &after_marker[argument_start + 1..];
                    if let Some((json, _)) = extract_balanced_json_object(after_paren) {
                        return Some(json);
                    }
                }
            } else if let Some(equal_start) = after_marker.find('=') {
                let after_equal = &after_marker[equal_start + 1..];
                if let Some((json, _)) = extract_balanced_json_object(after_equal) {
                    return Some(json);
                }
            }
        }
    }
    None
}

fn extract_balanced_json_object(input: &str) -> Option<(&str, usize)> {
    let start = input.find('{')?;
    let bytes = input.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for index in start..bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match byte {
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((&input[start..=index], index + 1));
                }
            }
            _ => {}
        }
    }

    None
}

fn finalize_document(root: crate::core::model::BuiNode) -> Result<BuiDocument, String> {
    let document = BuiDocument {
        version: "3.0-ir".to_string(),
        scene_name: "OpenDesignHtmlScene".to_string(),
        imports: Vec::new(),
        state_model: crate::core::model::BuiStateModel::default(),
        interaction_model: BuiInteractionModel::default(),
        resources: crate::core::model::BuiResources::default(),
        root,
    };
    validate_bui_document(&document)?;
    Ok(document)
}
