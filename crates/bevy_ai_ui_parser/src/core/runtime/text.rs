use std::path::{Path, PathBuf};

use bevy_asset::{io::AssetSourceId, AssetPath, AssetServer};
use bevy_ecs::{prelude::*, system::EntityCommands};
use bevy_log::{info, warn};
use bevy_text::{
    EditableText, FontSize, FontSource, FontWeight, LetterSpacing, TextColor, TextCursorStyle,
    TextFont, TextLayout,
};
use bevy_ui::{prelude::*, widget::TextShadow, FocusPolicy};

use crate::core::{
    legacy::{BuiTextInput, BuiTextInputMirror},
    model::{BuiNode, BuiNodeType, BuiTextConfig},
    style::css_parser::{parse_color, parse_linebreak, parse_text_justify, parse_text_line_height},
};

pub(crate) fn spawn_text_node(
    entity_commands: &mut EntityCommands,
    asset_server: &AssetServer,
    node: &BuiNode,
    base_node: Node,
) -> Result<(), String> {
    let text_config = node
        .text_config
        .as_ref()
        .ok_or_else(|| format!("Text node '{}' is missing text_config.", node.id))?;
    if text_config.placeholder.is_some() {
        return Err(format!(
            "Text node '{}' cannot use text_config.placeholder.",
            node.id
        ));
    }

    entity_commands.insert((
        base_node,
        Text::new(text_config.content.clone()),
        text_font(asset_server, text_config),
        TextColor(parse_color(&text_config.font_color)?),
        text_layout(text_config)?,
        FocusPolicy::Pass,
    ));
    insert_optional_text_style_components(entity_commands, text_config)?;
    Ok(())
}

pub(crate) fn spawn_text_input_node(
    entity_commands: &mut EntityCommands,
    asset_server: &AssetServer,
    node: &BuiNode,
    base_node: Node,
) -> Result<TextInputMirrorSpec, String> {
    let text_config = text_input_config(node)?;
    let text_font = text_font(asset_server, text_config);
    let text_color = TextColor(parse_color(&text_config.font_color)?);
    let text_layout = text_layout(text_config)?;
    entity_commands.insert((
        base_node,
        EditableText {
            visible_width: text_config.visible_width.or(Some(24.0)),
            allow_newlines: text_config.allow_newlines.unwrap_or(false),
            ..EditableText::new(&text_config.content)
        },
        text_layout.clone(),
        text_font.clone(),
        text_color,
        TextCursorStyle::default(),
        FocusPolicy::Block,
        BuiTextInput,
        text_config.clone(),
    ));
    insert_optional_text_style_components(entity_commands, text_config)?;
    Ok(TextInputMirrorSpec {
        text: initial_text_input_display(text_config, false),
        text_font,
        text_color,
        text_layout,
        line_height: text_config.line_height.clone(),
        letter_spacing: text_config.letter_spacing,
        text_shadow: text_shadow(text_config)?,
    })
}

pub(crate) struct TextInputMirrorSpec {
    pub(crate) text: String,
    pub(crate) text_font: TextFont,
    pub(crate) text_color: TextColor,
    pub(crate) text_layout: TextLayout,
    pub(crate) line_height: Option<String>,
    pub(crate) letter_spacing: Option<f32>,
    pub(crate) text_shadow: Option<TextShadow>,
}

pub(crate) fn spawn_text_input_mirror(
    commands: &mut Commands,
    entity: Entity,
    spec: TextInputMirrorSpec,
) -> Result<(), String> {
    let mirror = commands
        .spawn((
            Text::new(spec.text),
            spec.text_font,
            spec.text_color,
            spec.text_layout,
            FocusPolicy::Pass,
            BuiTextInputMirror { target: entity },
        ))
        .id();
    if let Some(line_height) = spec.line_height.as_deref() {
        commands
            .entity(mirror)
            .insert(parse_text_line_height(line_height)?);
    }
    if let Some(letter_spacing) = spec.letter_spacing {
        commands.entity(mirror).insert(LetterSpacing::Px(letter_spacing));
    }
    if let Some(text_shadow) = spec.text_shadow {
        commands.entity(mirror).insert(text_shadow);
    }
    commands.entity(entity).add_child(mirror);
    Ok(())
}

fn insert_optional_text_style_components(
    entity_commands: &mut EntityCommands,
    text_config: &BuiTextConfig,
) -> Result<(), String> {
    if let Some(line_height) = text_config.line_height.as_deref() {
        entity_commands.insert(parse_text_line_height(line_height)?);
    }
    if let Some(letter_spacing) = text_config.letter_spacing {
        entity_commands.insert(LetterSpacing::Px(letter_spacing));
    }
    if let Some(text_shadow) = text_shadow(text_config)? {
        entity_commands.insert(text_shadow);
    }
    Ok(())
}

fn text_input_config(node: &BuiNode) -> Result<&BuiTextConfig, String> {
    if let Some(text_config) = &node.text_config {
        return Ok(text_config);
    }

    node.children
        .iter()
        .find_map(|child| {
            matches!(child.node_type, BuiNodeType::Text)
                .then_some(child.text_config.as_ref())
                .flatten()
        })
        .ok_or_else(|| {
            format!(
                "TextInput node '{}' is missing text_config and has no Text child fallback.",
                node.id
            )
        })
}

fn text_font(asset_server: &AssetServer, text_config: &BuiTextConfig) -> TextFont {
    TextFont {
        font: load_font(asset_server, text_config.font_path.as_deref()),
        font_size: FontSize::Px(text_config.font_size),
        weight: text_config
            .font_weight
            .map(FontWeight)
            .unwrap_or(FontWeight::NORMAL),
        ..Default::default()
    }
}

fn text_shadow(text_config: &BuiTextConfig) -> Result<Option<TextShadow>, String> {
    let Some(shadow) = &text_config.text_shadow else {
        return Ok(None);
    };

    let mut text_shadow = TextShadow::default();
    if let Some(offset_x) = shadow.offset_x {
        text_shadow.offset.x = offset_x;
    }
    if let Some(offset_y) = shadow.offset_y {
        text_shadow.offset.y = offset_y;
    }
    if let Some(color) = &shadow.color {
        text_shadow.color = parse_color(color)?;
    }

    Ok(Some(text_shadow))
}

fn text_layout(text_config: &BuiTextConfig) -> Result<TextLayout, String> {
    if let Some(text_align) = &text_config.text_align {
        let justify = parse_text_justify(text_align)?;
        if let Some(linebreak) = &text_config.linebreak {
            return Ok(TextLayout::new(justify, parse_linebreak(linebreak)?));
        }
        if text_config.allow_newlines.unwrap_or(false) {
            return Ok(TextLayout::justify(justify));
        }
        return Ok(TextLayout::no_wrap().with_justify(justify));
    }

    if let Some(linebreak) = &text_config.linebreak {
        return Ok(TextLayout::linebreak(parse_linebreak(linebreak)?));
    }

    if text_config.allow_newlines.unwrap_or(false) {
        return Ok(TextLayout::default());
    }

    Ok(TextLayout::no_wrap())
}

fn load_font(asset_server: &AssetServer, font_path: Option<&str>) -> FontSource {
    let Some(font_path) = font_path else {
        return FontSource::default();
    };

    if asset_path_exists(font_path) {
        return FontSource::from(asset_server.load(font_path.to_owned()));
    }

    if let Some(windows_font) = windows_font_asset_path(font_path) {
        warn!(
            "BUI font path '{}' does not exist under the Bevy asset root. Loading '{}' from the optional windows_fonts asset source.",
            font_path, windows_font
        );
        return FontSource::from(asset_server.load(windows_font));
    }

    if let Some(macos_font) = macos_font_asset_path(font_path) {
        info!(
            "BUI font path '{}' does not exist under the Bevy asset root. Loading '{}' from an optional macOS font asset source.",
            font_path, macos_font
        );
        return FontSource::from(asset_server.load(macos_font));
    }

    warn!(
        "BUI font path '{}' does not exist under the Bevy asset root. Falling back to the default font.",
        font_path
    );
    FontSource::default()
}

fn asset_path_exists(asset_path: &str) -> bool {
    Path::new("assets").join(asset_path).exists()
}

fn windows_font_asset_path(asset_path: &str) -> Option<AssetPath<'static>> {
    let file_name = Path::new(asset_path).file_name()?.to_owned();
    let windows_font = Path::new("/mnt/c/Windows/Fonts").join(&file_name);

    windows_font.exists().then(|| {
        AssetPath::from_path_buf(PathBuf::from(file_name))
            .with_source(AssetSourceId::from("windows_fonts"))
    })
}

fn macos_font_asset_path(asset_path: &str) -> Option<AssetPath<'static>> {
    let file_name = Path::new(asset_path).file_name()?.to_owned();

    let macos_font = Path::new("/System/Library/Fonts").join(&file_name);
    if macos_font.exists() {
        return Some(
            AssetPath::from_path_buf(PathBuf::from(file_name))
                .with_source(AssetSourceId::from("macos_fonts")),
        );
    }

    let file_name = Path::new(asset_path).file_name()?.to_owned();
    let supplemental_font = Path::new("/System/Library/Fonts/Supplemental").join(&file_name);
    supplemental_font.exists().then(|| {
        AssetPath::from_path_buf(PathBuf::from(file_name))
            .with_source(AssetSourceId::from("macos_supplemental_fonts"))
    })
}

fn initial_text_input_display(text_config: &BuiTextConfig, is_focused: bool) -> String {
    if text_config.content.is_empty() && !is_focused {
        return text_config.placeholder.clone().unwrap_or_default();
    }

    text_config.content.clone()
}
