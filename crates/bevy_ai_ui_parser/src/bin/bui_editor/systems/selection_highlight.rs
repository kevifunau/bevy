use bevy::prelude::*;
use bevy::ui::ui_transform::UiGlobalTransform;
use bevy::ui::ComputedNode;

use bevy_ai_ui_parser::BuiId;

use crate::app_state::EditorState;

#[derive(Component)]
pub struct SelectionBorder;

pub fn update_selection_highlight(
    editor_state: Res<EditorState>,
    mut commands: Commands,
    bui_nodes: Query<(Entity, &BuiId, &ComputedNode, &UiGlobalTransform)>,
    existing_borders: Query<Entity, With<SelectionBorder>>,
) {
    if !editor_state.is_changed() && !existing_borders.is_empty() {
        return;
    }

    for entity in &existing_borders {
        commands.entity(entity).despawn();
    }

    let Some(selected_id) = &editor_state.selected_node_id else {
        return;
    };

    for (entity, bui_id, computed, transform) in &bui_nodes {
        if &bui_id.0 != selected_id {
            continue;
        }

        let size = computed.size();
        let pos = transform.affine().translation;

        commands.spawn((
            SelectionBorder,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x - size.x / 2.0 - 2.0),
                top: Val::Px(pos.y - size.y / 2.0 - 2.0),
                width: Val::Px(size.x + 4.0),
                height: Val::Px(size.y + 4.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.0, 1.0, 0.5)),
            GlobalZIndex(10000),
        ));
    }
}
