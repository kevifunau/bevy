use bevy::prelude::*;
use bevy::picking::hover::Hovered;
use bevy::ui_widgets::{
    observe, Slider, SliderDragState, SliderRange, SliderThumb, SliderValue, TrackClick,
    ValueChange,
};

use crate::blackboard::SettingsResource;
use crate::FmvAppState;

const DIALOG_BG: Color = Color::srgb(0.08, 0.08, 0.14);
const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
const LABEL_COLOR: Color = Color::srgb(0.7, 0.7, 0.85);
const SLIDER_TRACK: Color = Color::srgb(0.06, 0.06, 0.10);
const SLIDER_THUMB: Color = Color::srgb(0.35, 0.55, 0.85);
const BORDER_COLOR: Color = Color::srgb(0.3, 0.5, 0.8);
const APPLY_BG: Color = Color::srgb(0.15, 0.25, 0.45);
const APPLY_HOVER: Color = Color::srgb(0.25, 0.35, 0.55);
const CLOSE_BG: Color = Color::srgb(0.08, 0.08, 0.14);
const CLOSE_HOVER: Color = Color::srgb(0.15, 0.15, 0.22);

#[derive(Component)]
pub struct SettingsRoot;

#[derive(Component)]
pub struct CloseButtonTag;

#[derive(Component)]
pub struct ApplyButtonTag;

#[derive(Component, Default)]
pub struct MasterVolumeSliderMarker;

#[derive(Component, Default)]
pub struct MusicVolumeSliderMarker;

#[derive(Component, Default)]
pub struct SfxVolumeSliderMarker;

#[derive(Component, Default)]
pub struct DemoSliderThumb;

#[derive(Component)]
pub struct LanguageButtonTag;

#[derive(Resource)]
pub struct SettingsEntities {
    pub overlay: Entity,
}

fn make_slider(val: f32) -> impl Bundle {
    (
        Hovered::default(),
        Slider {
            track_click: TrackClick::Snap,
            ..Default::default()
        },
        SliderValue(val),
        SliderRange::new(0.0, 100.0),
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            height: px(12),
            width: percent(55),
            ..default()
        },
        Children::spawn((
            Spawn((
                Node {
                    height: px(6),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(SLIDER_TRACK),
            )),
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(12),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                children![(
                    DemoSliderThumb,
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: px(18),
                        height: px(18),
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(SLIDER_THUMB),
                )],
            )),
        )),
    )
}

pub fn setup_settings(mut commands: Commands, settings: Res<SettingsResource>) {
    let master_val = settings.master_volume;
    let music_val = settings.music_volume;
    let sfx_val = settings.sfx_volume;
    let current_lang = settings
        .languages
        .get(settings.language_index)
        .cloned()
        .unwrap_or("English".into());

    let master_slider = commands
        .spawn((
            make_slider(master_val),
            MasterVolumeSliderMarker,
            observe(
                |value_change: On<ValueChange<f32>>,
                 mut settings: ResMut<SettingsResource>| {
                    settings.master_volume = value_change.value;
                },
            ),
        ))
        .id();

    let music_slider = commands
        .spawn((
            make_slider(music_val),
            MusicVolumeSliderMarker,
            observe(
                |value_change: On<ValueChange<f32>>,
                 mut settings: ResMut<SettingsResource>| {
                    settings.music_volume = value_change.value;
                },
            ),
        ))
        .id();

    let sfx_slider = commands
        .spawn((
            make_slider(sfx_val),
            SfxVolumeSliderMarker,
            observe(
                |value_change: On<ValueChange<f32>>,
                 mut settings: ResMut<SettingsResource>| {
                    settings.sfx_volume = value_change.value;
                },
            ),
        ))
        .id();

    let mut master_row = Entity::PLACEHOLDER;
    let mut music_row = Entity::PLACEHOLDER;
    let mut sfx_row = Entity::PLACEHOLDER;

    let overlay = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(OVERLAY_BG),
            SettingsRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: px(500),
                        height: px(450),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(px(30)),
                        border: UiRect::all(px(4)),
                        border_radius: BorderRadius::all(px(20)),
                        ..default()
                    },
                    BackgroundColor(DIALOG_BG),
                    BorderColor::all(BORDER_COLOR),
                ))
                .with_children(|dialog| {
                    dialog
                        .spawn((
                            Node {
                                width: percent(100),
                                height: px(40),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(px(25)),
                                ..default()
                            },
                        ))
                        .with_children(|header| {
                            header.spawn((
                                Text::new("SETTINGS"),
                                TextFont {
                                    font_size: FontSize::Px(24.0),
                                    ..default()
                                },
                                TextColor(LABEL_COLOR),
                            ));

                            header
                                .spawn((
                                    Button,
                                    Node {
                                        width: px(30),
                                        height: px(30),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(px(15)),
                                        ..default()
                                    },
                                    BackgroundColor(CLOSE_BG),
                                    CloseButtonTag,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("✕"),
                                        TextFont {
                                            font_size: FontSize::Px(18.0),
                                            ..default()
                                        },
                                        TextColor(LABEL_COLOR),
                                    ));
                                });
                        });

                    dialog.spawn((
                        Node {
                            width: percent(100),
                            height: px(2),
                            margin: UiRect::bottom(px(25)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
                    ));

                    master_row = dialog
                        .spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::vertical(px(15)),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    width: percent(30),
                                    ..default()
                                },
                            ))
                            .with_children(|cell| {
                                cell.spawn((
                                    Text::new("MASTER VOLUME"),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    },
                                    TextColor(LABEL_COLOR),
                                ));
                            });
                        })
                        .id();

                    music_row = dialog
                        .spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::vertical(px(15)),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    width: percent(30),
                                    ..default()
                                },
                            ))
                            .with_children(|cell| {
                                cell.spawn((
                                    Text::new("MUSIC VOLUME"),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    },
                                    TextColor(LABEL_COLOR),
                                ));
                            });
                        })
                        .id();

                    sfx_row = dialog
                        .spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::vertical(px(15)),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    width: percent(30),
                                    ..default()
                                },
                            ))
                            .with_children(|cell| {
                                cell.spawn((
                                    Text::new("SFX VOLUME"),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    },
                                    TextColor(LABEL_COLOR),
                                ));
                            });
                        })
                        .id();

                    dialog
                        .spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::vertical(px(15)),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    width: percent(30),
                                    ..default()
                                },
                            ))
                            .with_children(|cell| {
                                cell.spawn((
                                    Text::new("LANGUAGE"),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    },
                                    TextColor(LABEL_COLOR),
                                ));
                            });

                            row
                                .spawn((
                                    Button,
                                    Node {
                                        width: percent(55),
                                        height: px(35),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(px(2)),
                                        border_radius: BorderRadius::all(px(8)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.06, 0.06, 0.10)),
                                    BorderColor::all(Color::srgb(0.2, 0.3, 0.5)),
                                    LanguageButtonTag,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(current_lang),
                                        TextFont {
                                            font_size: FontSize::Px(14.0),
                                            ..default()
                                        },
                                        TextColor(LABEL_COLOR),
                                    ));
                                    btn.spawn((
                                        Text::new("  [▼]"),
                                        TextFont {
                                            font_size: FontSize::Px(12.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.5, 0.5, 0.6)),
                                    ));
                                });
                        });

                    dialog.spawn((
                        Node {
                            width: percent(100),
                            height: px(2),
                            margin: UiRect::vertical(px(15)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
                    ));

                    dialog
                        .spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                            },
                        ))
                        .with_children(|footer| {
                            footer
                                .spawn((
                                    Button,
                                    Node {
                                        width: px(100),
                                        height: px(40),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(px(2)),
                                        border_radius: BorderRadius::all(px(20)),
                                        margin: UiRect::left(px(15)),
                                        ..default()
                                    },
                                    BackgroundColor(APPLY_BG),
                                    BorderColor::all(BORDER_COLOR),
                                    ApplyButtonTag,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("APPLY"),
                                        TextFont {
                                            font_size: FontSize::Px(16.0),
                                            ..default()
                                        },
                                        TextColor(LABEL_COLOR),
                                    ));
                                });
                        });
                });
        })
        .id();

    let master_label = commands
        .spawn((
            Node {
                width: percent(12),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            children![(
                Text::new(format!("{}%", master_val as i32)),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(LABEL_COLOR),
            )],
        ))
        .id();
    commands.entity(master_row).add_child(master_slider);
    commands.entity(master_row).add_child(master_label);

    let music_label = commands
        .spawn((
            Node {
                width: percent(12),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            children![(
                Text::new(format!("{}%", music_val as i32)),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(LABEL_COLOR),
            )],
        ))
        .id();
    commands.entity(music_row).add_child(music_slider);
    commands.entity(music_row).add_child(music_label);

    let sfx_label = commands
        .spawn((
            Node {
                width: percent(12),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            children![(
                Text::new(format!("{}%", sfx_val as i32)),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(LABEL_COLOR),
            )],
        ))
        .id();
    commands.entity(sfx_row).add_child(sfx_slider);
    commands.entity(sfx_row).add_child(sfx_label);

    commands.insert_resource(SettingsEntities { overlay });
}

pub fn settings_interaction(
    mut next_state: ResMut<NextState<FmvAppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&CloseButtonTag>, Option<&ApplyButtonTag>, Option<&LanguageButtonTag>),
        (Changed<Interaction>, With<Button>),
    >,
    slider_query: Query<
        (Entity, &SliderValue, &SliderRange, &Hovered, &SliderDragState),
        Or<(With<MasterVolumeSliderMarker>, With<MusicVolumeSliderMarker>, With<SfxVolumeSliderMarker>)>,
    >,
    children_query: Query<&Children>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor, Has<DemoSliderThumb>), (Without<MasterVolumeSliderMarker>, Without<Button>)>,
    mut settings: ResMut<SettingsResource>,
) {
    for (slider_ent, value, range, hovered, drag_state) in &slider_query {
        for child in children_query.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut thumb_bg, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                thumb_node.left = percent(range.thumb_position(value.0) * 100.0);
                thumb_bg.0 = if hovered.0 | drag_state.dragging {
                    SLIDER_THUMB.lighter(0.3)
                } else {
                    SLIDER_THUMB
                };
            }
        }
    }

    for (interaction, mut color, close_tag, apply_tag, lang_tag) in &mut interaction_query {
        if close_tag.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    next_state.set(FmvAppState::MainMenu);
                }
                Interaction::Hovered => {
                    *color = CLOSE_HOVER.into();
                }
                Interaction::None => {
                    *color = CLOSE_BG.into();
                }
            }
        }
        if apply_tag.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    next_state.set(FmvAppState::MainMenu);
                }
                Interaction::Hovered => {
                    *color = APPLY_HOVER.into();
                }
                Interaction::None => {
                    *color = APPLY_BG.into();
                }
            }
        }
        if lang_tag.is_some() {
            if *interaction == Interaction::Pressed {
                settings.language_index =
                    (settings.language_index + 1) % settings.languages.len();
            }
        }
    }
}

pub fn cleanup_settings(
    mut commands: Commands,
    entities: Res<SettingsEntities>,
) {
    commands.entity(entities.overlay).despawn();
    commands.remove_resource::<SettingsEntities>();
}