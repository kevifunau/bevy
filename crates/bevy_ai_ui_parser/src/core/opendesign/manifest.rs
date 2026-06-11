use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::core::model::{
    BuiDocument, BuiImageConfig, BuiNode, BuiStateVisual, BuiTextureAtlasConfig,
    BuiTextureSlicerConfig, BuiVisuals,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenDesignAssetManifest {
    pub(crate) version: String,
    pub(crate) ui_root: String,
    pub(crate) design_system: String,
    #[serde(default)]
    pub(crate) assets: Vec<OpenDesignAssetEntry>,
    #[serde(default)]
    pub(crate) components: Vec<OpenDesignComponentEntry>,
    #[serde(default)]
    pub(crate) icons: Vec<OpenDesignIconEntry>,
    #[serde(default)]
    pub(crate) atlases: Vec<OpenDesignAtlasEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenDesignAssetEntry {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) path: String,
    pub(crate) usage: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenDesignComponentEntry {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) node_selector: String,
    #[serde(default)]
    pub(crate) asset_ref: Option<String>,
    #[serde(default)]
    pub(crate) atlas_ref: Option<String>,
    #[serde(default)]
    pub(crate) states: HashMap<String, String>,
    #[serde(default)]
    pub(crate) image_mode: Option<String>,
    #[serde(default)]
    pub(crate) slicer: Option<BuiTextureSlicerConfig>,
    #[serde(default)]
    pub(crate) atlas: Option<BuiTextureAtlasConfig>,
    #[serde(default)]
    pub(crate) flip_x: bool,
    #[serde(default)]
    pub(crate) flip_y: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenDesignIconEntry {
    pub(crate) semantic: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenDesignAtlasEntry {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) tile_width: u32,
    pub(crate) tile_height: u32,
    pub(crate) columns: u32,
    pub(crate) rows: u32,
    #[serde(default)]
    pub(crate) padding_x: Option<u32>,
    #[serde(default)]
    pub(crate) padding_y: Option<u32>,
    #[serde(default)]
    pub(crate) index: usize,
}

pub(crate) fn discover_manifest_path(html_path: &Path) -> Option<PathBuf> {
    let manifest_path = html_path.with_file_name("bevy-ui.assets.json");
    manifest_path.exists().then_some(manifest_path)
}

pub(crate) fn load_manifest_file(path: &Path) -> Result<OpenDesignAssetManifest, String> {
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "Failed to read OpenDesign asset manifest '{}': {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse OpenDesign asset manifest '{}': {error}",
            path.display()
        )
    })
}

pub(crate) fn apply_manifest_to_document(
    document: &mut BuiDocument,
    manifest: &OpenDesignAssetManifest,
    base_dir: Option<&Path>,
) -> Result<(), String> {
    validate_manifest(manifest, base_dir)?;

    let assets_by_id: HashMap<&str, &OpenDesignAssetEntry> = manifest
        .assets
        .iter()
        .map(|asset| (asset.id.as_str(), asset))
        .collect();
    let atlases_by_id: HashMap<&str, &OpenDesignAtlasEntry> = manifest
        .atlases
        .iter()
        .map(|atlas| (atlas.id.as_str(), atlas))
        .collect();

    for component in &manifest.components {
        apply_component_to_node(&mut document.root, component, &assets_by_id, &atlases_by_id)?;
    }

    for icon in &manifest.icons {
        apply_icon_to_node(&mut document.root, icon);
    }

    Ok(())
}

fn validate_manifest(
    manifest: &OpenDesignAssetManifest,
    base_dir: Option<&Path>,
) -> Result<(), String> {
    if manifest.version.trim().is_empty() {
        return Err("OpenDesign asset manifest version must not be empty.".to_string());
    }
    if manifest.ui_root.trim().is_empty() {
        return Err("OpenDesign asset manifest ui_root must not be empty.".to_string());
    }
    if manifest.design_system.trim().is_empty() {
        return Err("OpenDesign asset manifest design_system must not be empty.".to_string());
    }

    let Some(base_dir) = base_dir else {
        return Ok(());
    };

    for asset in &manifest.assets {
        ensure_path_exists(base_dir, &asset.path)?;
    }
    for component in &manifest.components {
        for state_path in component.states.values() {
            ensure_path_exists(base_dir, state_path)?;
        }
    }
    for icon in &manifest.icons {
        ensure_path_exists(base_dir, &icon.path)?;
    }
    for atlas in &manifest.atlases {
        ensure_path_exists(base_dir, &atlas.path)?;
    }

    Ok(())
}

fn ensure_path_exists(base_dir: &Path, raw_path: &str) -> Result<(), String> {
    let trimmed = raw_path.trim().trim_start_matches("./");
    if trimmed.is_empty() {
        return Err("OpenDesign asset manifest path must not be empty.".to_string());
    }
    let candidate = base_dir.join(trimmed);
    if candidate.exists() {
        Ok(())
    } else {
        Err(format!(
            "OpenDesign asset manifest references missing file '{}'.",
            candidate.display()
        ))
    }
}

fn apply_component_to_node(
    node: &mut BuiNode,
    component: &OpenDesignComponentEntry,
    assets_by_id: &HashMap<&str, &OpenDesignAssetEntry>,
    atlases_by_id: &HashMap<&str, &OpenDesignAtlasEntry>,
) -> Result<(), String> {
    if selector_matches(node, &component.node_selector) {
        let base_image = component_base_image(component, assets_by_id, atlases_by_id)?;
        if let Some(image) = base_image.clone() {
            node.content.image = Some(image);
        }

        for (state_name, path) in &component.states {
            let normalized = normalize_state_name(state_name);
            let mut image = base_image.clone().unwrap_or_else(|| BuiImageConfig {
                texture_path: path.clone(),
                image_mode: component.image_mode.clone(),
                background_size: None,
                background_position: None,
                atlas: component.atlas.clone(),
                slicer: component.slicer.clone(),
                flip_x: component.flip_x,
                flip_y: component.flip_y,
            });
            image.texture_path = path.clone();
            let state_visual = node
                .state_visuals
                .entry(normalized)
                .or_insert_with(empty_state_visual);
            state_visual.image = Some(image);
        }
    }

    for child in &mut node.children {
        apply_component_to_node(child, component, assets_by_id, atlases_by_id)?;
    }

    Ok(())
}

fn component_base_image(
    component: &OpenDesignComponentEntry,
    assets_by_id: &HashMap<&str, &OpenDesignAssetEntry>,
    atlases_by_id: &HashMap<&str, &OpenDesignAtlasEntry>,
) -> Result<Option<BuiImageConfig>, String> {
    let texture_path = component
        .asset_ref
        .as_deref()
        .map(|asset_ref| {
            assets_by_id
                .get(asset_ref)
                .ok_or_else(|| {
                    format!(
                        "OpenDesign asset manifest references unknown asset '{}'.",
                        asset_ref
                    )
                })
                .map(|asset| asset.path.clone())
        })
        .transpose()?
        .or_else(|| component.states.get("idle").cloned())
        .or_else(|| {
            component
                .atlas_ref
                .as_deref()
                .and_then(|atlas_ref| atlases_by_id.get(atlas_ref))
                .map(|atlas| atlas.path.clone())
        });

    let atlas = component
        .atlas_ref
        .as_deref()
        .map(|atlas_ref| {
            atlases_by_id
                .get(atlas_ref)
                .ok_or_else(|| {
                    format!(
                        "OpenDesign asset manifest references unknown atlas '{}'.",
                        atlas_ref
                    )
                })
                .map(|atlas| BuiTextureAtlasConfig {
                    tile_width: atlas.tile_width,
                    tile_height: atlas.tile_height,
                    columns: atlas.columns,
                    rows: atlas.rows,
                    padding_x: atlas.padding_x,
                    padding_y: atlas.padding_y,
                    index: atlas.index,
                })
        })
        .transpose()?
        .or_else(|| component.atlas.clone());

    let Some(texture_path) = texture_path else {
        return Ok(None);
    };

    Ok(Some(BuiImageConfig {
        texture_path,
        image_mode: component.image_mode.clone(),
        background_size: None,
        background_position: None,
        atlas,
        slicer: component.slicer.clone(),
        flip_x: component.flip_x,
        flip_y: component.flip_y,
    }))
}

fn apply_icon_to_node(node: &mut BuiNode, icon: &OpenDesignIconEntry) {
    let Some((prefix, value)) = icon.semantic.split_once(':') else {
        return;
    };
    let marker = match prefix {
        "skill" => format!("data-skill:{value}"),
        "equip" => format!("data-equip:{value}"),
        _ => return,
    };

    if node.markers.iter().any(|existing| existing == &marker) && node.content.image.is_none() {
        node.content.image = Some(BuiImageConfig {
            texture_path: icon.path.clone(),
            image_mode: Some("stretch".to_string()),
            background_size: None,
            background_position: None,
            atlas: None,
            slicer: None,
            flip_x: false,
            flip_y: false,
        });
    }

    for child in &mut node.children {
        apply_icon_to_node(child, icon);
    }
}

fn selector_matches(node: &BuiNode, selector: &str) -> bool {
    let selector = selector.trim();
    if let Some(id) = selector.strip_prefix('#') {
        return node.id == id;
    }
    if let Some(class_name) = selector.strip_prefix('.') {
        return node
            .markers
            .iter()
            .any(|marker| marker == &format!("class:{class_name}"));
    }
    false
}

fn normalize_state_name(state_name: &str) -> String {
    match state_name.trim() {
        "idle" | "normal" => "normal".to_string(),
        "hover" | "hovered" => "hovered".to_string(),
        "pressed" | "active" => "pressed".to_string(),
        "disabled" => "disabled".to_string(),
        other => other.to_string(),
    }
}

fn empty_state_visual() -> BuiStateVisual {
    BuiStateVisual {
        styles: Default::default(),
        visuals: BuiVisuals::default(),
        text_color: None,
        image: None,
    }
}
