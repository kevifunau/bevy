use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::*;
use bevy_math::Vec2;
use bevy_ui::ComputedNode;
use bevy_ui::UiTransform;

use crate::core::runtime::components::{BuiStageFit, BuiStageFitMode};

pub(crate) fn sync_stage_fit_system(
    parents: Query<&ComputedNode>,
    mut stages: Query<(&BuiStageFit, &ChildOf, &mut UiTransform)>,
) {
    for (fit, child_of, mut transform) in &mut stages {
        let Ok(parent) = parents.get(child_of.parent()) else {
            continue;
        };

        let parent_size = parent.size() * parent.inverse_scale_factor();
        if parent_size.x <= 0.0 || parent_size.y <= 0.0 {
            continue;
        }

        let scale = stage_fit_scale(*fit, parent_size);
        if scale.x <= 0.0 || scale.y <= 0.0 || !scale.is_finite() {
            continue;
        }

        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}

pub(crate) fn stage_fit_scale(fit: BuiStageFit, parent_size: Vec2) -> Vec2 {
    let scale_x = parent_size.x / fit.design_width;
    let scale_y = parent_size.y / fit.design_height;
    match fit.mode {
        BuiStageFitMode::Contain => Vec2::splat(scale_x.min(scale_y)),
        BuiStageFitMode::Cover => Vec2::splat(scale_x.max(scale_y)),
        BuiStageFitMode::Fill => Vec2::new(scale_x, scale_y),
        BuiStageFitMode::None => Vec2::ONE,
        BuiStageFitMode::ScaleDown => Vec2::splat(scale_x.min(scale_y).min(1.0)),
    }
}
