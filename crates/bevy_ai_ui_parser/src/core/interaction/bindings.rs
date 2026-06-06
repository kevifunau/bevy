use bevy_camera::visibility::Visibility;
use bevy_ecs::prelude::*;
use bevy_text::{FontSize, LetterSpacing, LineHeight, TextBounds, TextColor, TextFont, TextLayout};
use bevy_ui::{prelude::*, widget::TextShadow};

use crate::core::{
    interaction::types::{BuiBindingUpdate, BuiBindingValue},
    legacy::BuiBindings,
    style::css_parser::{
        parse_color, parse_display, parse_rotation, parse_text_justify, parse_ui_rect,
        parse_val2, parse_vec2, parse_visibility,
    },
};

pub(crate) fn apply_bui_binding_updates_system(
    mut updates: MessageReader<BuiBindingUpdate>,
    mut nodes: Query<(&BuiBindings, &mut Node)>,
    mut ui_transforms: Query<(&BuiBindings, &mut UiTransform)>,
    mut texts: Query<(&BuiBindings, &mut Text)>,
    mut text_layouts: Query<(&BuiBindings, &mut TextLayout)>,
    mut text_bounds: Query<(&BuiBindings, &mut TextBounds)>,
    mut text_fonts: Query<(&BuiBindings, &mut TextFont)>,
    mut line_heights: Query<(&BuiBindings, &mut LineHeight)>,
    mut letter_spacings: Query<(&BuiBindings, &mut LetterSpacing)>,
    mut text_shadows: Query<(&BuiBindings, &mut TextShadow)>,
    mut text_colors: Query<(&BuiBindings, &mut TextColor)>,
    mut images: Query<(&BuiBindings, &mut ImageNode)>,
    mut backgrounds: Query<(&BuiBindings, &mut BackgroundColor)>,
    mut borders: Query<(&BuiBindings, &mut BorderColor)>,
    mut visibilities: Query<(&BuiBindings, &mut Visibility)>,
) {
    for update in updates.read() {
        for (bindings, mut node) in &mut nodes {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("display", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_display(value) {
                            node.display = parsed;
                        }
                    }
                    ("border_width", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_ui_rect(value) {
                            node.border = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut ui_transform) in &mut ui_transforms {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("ui_rotation", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_rotation(value) {
                            ui_transform.rotation = parsed;
                        }
                    }
                    ("ui_scale", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_vec2(value) {
                            ui_transform.scale = parsed;
                        }
                    }
                    ("ui_translation", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_val2(value) {
                            ui_transform.translation = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text) in &mut texts {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("text.content", BuiBindingValue::Text(value)) =
                    (binding.target.as_str(), &update.value)
                {
                    text.0 = value.clone();
                }
            }
        }

        for (bindings, mut text_layout) in &mut text_layouts {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("justify", BuiBindingValue::Text(value)) =
                    (binding.target.as_str(), &update.value)
                    && let Ok(parsed) = parse_text_justify(value)
                {
                    text_layout.justify = parsed;
                }
            }
        }

        for (bindings, mut bounds) in &mut text_bounds {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                match (binding.target.as_str(), &update.value) {
                    ("text_bounds.width", BuiBindingValue::Number(value)) => bounds.width = Some(*value),
                    ("text_bounds.height", BuiBindingValue::Number(value)) => bounds.height = Some(*value),
                    _ => {}
                }
            }
        }

        for (bindings, mut text_font) in &mut text_fonts {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("font_size", BuiBindingValue::Number(value)) =
                    (binding.target.as_str(), &update.value)
                {
                    text_font.font_size = FontSize::Px(*value);
                }
            }
        }

        for (bindings, mut line_height) in &mut line_heights {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("line_height", BuiBindingValue::Number(value)) =
                    (binding.target.as_str(), &update.value)
                {
                    *line_height = LineHeight::RelativeToFont(*value);
                }
            }
        }

        for (bindings, mut letter_spacing) in &mut letter_spacings {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("letter_spacing", BuiBindingValue::Number(value)) =
                    (binding.target.as_str(), &update.value)
                {
                    *letter_spacing = LetterSpacing::Px(*value);
                }
            }
        }

        for (bindings, mut text_shadow) in &mut text_shadows {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }

                match (binding.target.as_str(), &update.value) {
                    ("text_shadow.offset_x", BuiBindingValue::Number(value)) => text_shadow.offset.x = *value,
                    ("text_shadow.offset_y", BuiBindingValue::Number(value)) => text_shadow.offset.y = *value,
                    ("text_shadow.color", BuiBindingValue::Color(value)) => {
                        if let Ok(parsed) = parse_color(value) {
                            text_shadow.color = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }

        for (bindings, mut text_color) in &mut text_colors {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("text.color", BuiBindingValue::Color(value)) =
                    (binding.target.as_str(), &update.value)
                    && let Ok(parsed) = parse_color(value)
                {
                    text_color.0 = parsed;
                }
            }
        }

        for (bindings, mut image) in &mut images {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("image.tint", BuiBindingValue::Color(value)) =
                    (binding.target.as_str(), &update.value)
                    && let Ok(parsed) = parse_color(value)
                {
                    image.color = parsed;
                }
            }
        }

        for (bindings, mut background) in &mut backgrounds {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("background_color", BuiBindingValue::Color(value)) =
                    (binding.target.as_str(), &update.value)
                    && let Ok(parsed) = parse_color(value)
                {
                    background.0 = parsed;
                }
            }
        }

        for (bindings, mut border) in &mut borders {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                if let ("border_color", BuiBindingValue::Color(value)) =
                    (binding.target.as_str(), &update.value)
                    && let Ok(parsed) = parse_color(value)
                {
                    *border = BorderColor::all(parsed);
                }
            }
        }

        for (bindings, mut visibility) in &mut visibilities {
            for binding in &bindings.0 {
                if binding.source != update.source {
                    continue;
                }
                match (binding.target.as_str(), &update.value) {
                    ("visibility", BuiBindingValue::Bool(value)) => {
                        *visibility = if *value {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        };
                    }
                    ("visibility", BuiBindingValue::Text(value)) => {
                        if let Ok(parsed) = parse_visibility(value) {
                            *visibility = parsed;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
