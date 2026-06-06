use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_log::info;
use bevy_text::{TextColor, TextFont};
use bevy_ui::prelude::*;

use crate::core::legacy::BuiId;

/// Marker data for nodes that request a custom UI material shader.
#[derive(Component, Debug, Clone)]
pub struct BuiMaterialShader {
    /// Shader asset path copied from `visuals.material_shader`.
    pub path: String,
}

pub(crate) fn spawn_error_text(commands: &mut Commands, error: String) {
    let root = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: px(24).all(),
            ..Default::default()
        })
        .id();

    let text = commands
        .spawn((
            Text::new(error),
            TextFont::from_font_size(22.0),
            TextColor(Color::srgb(1.0, 0.2, 0.2)),
        ))
        .id();

    commands.entity(root).add_child(text);
}

pub(crate) fn material_shader_notice_system(
    shaders: Query<(&BuiId, &BuiMaterialShader), Added<BuiMaterialShader>>,
) {
    for (id, shader) in &shaders {
        info!(
            "BUI node '{}' requested custom UI material shader '{}'.",
            id.0, shader.path
        );
    }
}
