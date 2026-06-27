use bevy_color::{Alpha, Color};
use bevy_ecs::prelude::*;
use bevy_input::keyboard::KeyCode;
use bevy_input::prelude::*;
use bevy_picking::hover::HoverMap;
use bevy_text::{FontSize, FontSource, TextColor, TextFont};
use bevy_ui::prelude::*;

use crate::core::editor::state::{
    BuiEditorDialogPanel, BuiEditorDiscardButton, BuiEditorSaveButton, BuiEditorState, EditorMode,
};
use crate::core::runtime::components::BuiRootEntity;

pub(crate) fn editor_dialog_system(
    mut editor_state: ResMut<BuiEditorState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    root_entity: Option<Res<BuiRootEntity>>,
    target_camera_query: Query<&UiTargetCamera>,
    dialog_query: Query<Entity, With<BuiEditorDialogPanel>>,
    save_btn_query: Query<Entity, With<BuiEditorSaveButton>>,
    discard_btn_query: Query<Entity, With<BuiEditorDiscardButton>>,
    hover_map: Res<HoverMap>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if editor_state.mode != EditorMode::AwaitingSaveDialog {
        if !dialog_query.is_empty() {
            for entity in dialog_query.iter() {
                commands.entity(entity).despawn();
            }
            editor_state.dialog_entity = None;
        }
        return;
    }

    if editor_state.dialog_entity.is_none() {
        let inherited_target_camera =
            resolve_ui_target_camera(root_entity.as_deref(), &target_camera_query);
        spawn_dialog(&mut editor_state, &mut commands, inherited_target_camera);
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        for entity in save_btn_query.iter() {
            if is_hovered(&hover_map, &entity) {
                editor_state.save_requested = true;
                return;
            }
        }

        for entity in discard_btn_query.iter() {
            if is_hovered(&hover_map, &entity) {
                editor_state.discard_requested = true;
                return;
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        editor_state.mode = EditorMode::Active;
        despawn_dialog(&mut editor_state, &dialog_query, &mut commands);
    }
}

fn is_hovered(hover_map: &HoverMap, entity: &Entity) -> bool {
    hover_map
        .iter()
        .any(|(_, hovered)| hovered.contains_key(entity))
}

fn spawn_dialog(
    editor_state: &mut BuiEditorState,
    commands: &mut Commands,
    inherited_target_camera: Option<UiTargetCamera>,
) {
    let dialog = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            BackgroundColor(Color::srgb(0.0, 0.0, 0.0).with_alpha(0.5)),
            GlobalZIndex(9999),
            BuiEditorDialogPanel,
        ))
        .id();
    if let Some(target_camera) = inherited_target_camera.clone() {
        commands.entity(dialog).insert(target_camera);
    }

    let panel = commands
        .spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Px(150.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..Node::default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
        ))
        .id();
    if let Some(target_camera) = inherited_target_camera.clone() {
        commands.entity(panel).insert(target_camera);
    }

    let title = commands
        .spawn((
            Text::new("Save Changes?"),
            TextFont {
                font: FontSource::default(),
                font_size: FontSize::Px(18.0),
                ..TextFont::default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..Node::default()
            },
        ))
        .id();
    if let Some(target_camera) = inherited_target_camera.clone() {
        commands.entity(title).insert(target_camera);
    }

    let save_btn = commands
        .spawn((
            Text::new("Save"),
            TextFont {
                font: FontSource::default(),
                font_size: FontSize::Px(14.0),
                ..TextFont::default()
            },
            TextColor(Color::WHITE),
            Node {
                width: Val::Px(120.0),
                height: Val::Px(36.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..Node::default()
            },
            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            BorderColor::all(Color::WHITE),
            BuiEditorSaveButton,
        ))
        .id();
    if let Some(target_camera) = inherited_target_camera.clone() {
        commands.entity(save_btn).insert(target_camera);
    }

    let discard_btn = commands
        .spawn((
            Text::new("Discard"),
            TextFont {
                font: FontSource::default(),
                font_size: FontSize::Px(14.0),
                ..TextFont::default()
            },
            TextColor(Color::WHITE),
            Node {
                width: Val::Px(120.0),
                height: Val::Px(36.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..Node::default()
            },
            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
            BorderColor::all(Color::WHITE),
            BuiEditorDiscardButton,
        ))
        .id();
    if let Some(target_camera) = inherited_target_camera {
        commands.entity(discard_btn).insert(target_camera);
    }

    commands.entity(dialog).add_children(&[panel]);
    commands
        .entity(panel)
        .add_children(&[title, save_btn, discard_btn]);

    editor_state.dialog_entity = Some(dialog);
}

fn resolve_ui_target_camera(
    root_entity: Option<&BuiRootEntity>,
    target_camera_query: &Query<&UiTargetCamera>,
) -> Option<UiTargetCamera> {
    root_entity
        .and_then(|root| target_camera_query.get(root.0).ok())
        .cloned()
}

fn despawn_dialog(
    editor_state: &mut BuiEditorState,
    dialog_query: &Query<Entity, With<BuiEditorDialogPanel>>,
    commands: &mut Commands,
) {
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }
    editor_state.dialog_entity = None;
}
