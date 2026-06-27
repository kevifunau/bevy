use bevy_ecs::prelude::*;
use bevy_ui::prelude::*;

use crate::core::{
    model::BuiNode, runtime::diagnostics::BuiMaterialShader, style::css_parser::parse_color,
};

use super::helpers::build_box_shadow;

pub(crate) fn insert_visual_components(
    entity_commands: &mut EntityCommands,
    node: &BuiNode,
) -> Result<(), String> {
    if let Some(color) = &node.style.visuals.background_color {
        entity_commands.insert(BackgroundColor(parse_color(color)?));
    }

    if let Some(color) = &node.style.visuals.border_color {
        entity_commands.insert(BorderColor::all(parse_color(color)?));
    }

    if let Some(box_shadow) = &node.style.visuals.box_shadow {
        entity_commands.insert(build_box_shadow(box_shadow)?);
    }

    if let Some(shader_path) = &node.style.visuals.material_shader {
        entity_commands.insert(BuiMaterialShader {
            path: shader_path.clone(),
        });
    }

    Ok(())
}
