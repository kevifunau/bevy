use bevy::prelude::*;

use crate::FmvAppState;

const MENU_BG: Color = Color::srgb(0.04, 0.04, 0.08);
const CAPSULE_NORMAL: Color = Color::srgb(0.12, 0.12, 0.18);
const CAPSULE_HOVER: Color = Color::srgb(0.22, 0.22, 0.32);
const CAPSULE_BORDER: Color = Color::srgb(0.35, 0.55, 0.85);
const TITLE_COLOR: Color = Color::srgb(0.9, 0.85, 0.7);
const TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.95);
const SLOGAN_COLOR: Color = Color::srgb(0.6, 0.6, 0.75);

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MenuButtonTag(pub MenuAction);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MenuAction {
    NewGame,
    Continue,
    Chapters,
    Settings,
    Quit,
}

#[derive(Resource)]
pub struct MainMenuEntities {
    pub root: Entity,
}

pub fn setup_main_menu(mut commands: Commands) {
    let buttons_data = [
        ("NEW GAME", MenuAction::NewGame),
        ("CONTINUE", MenuAction::Continue),
        ("CHAPTER SELECT", MenuAction::Chapters),
        ("SETTINGS", MenuAction::Settings),
        ("QUIT", MenuAction::Quit),
    ];

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(MENU_BG),
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(35),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect {
                            left: percent(10),
                            top: percent(8),
                            bottom: percent(8),
                            right: percent(10),
                        },
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                ))
                .with_children(|left_panel| {
                    left_panel
                        .spawn((
                            Node {
                                margin: UiRect::bottom(px(40)),
                                ..default()
                            },
                        ))
                        .with_children(|title_area| {
                            title_area.spawn((
                                Text::new("《华君传》"),
                                TextFont {
                                    font_size: FontSize::Px(48.0),
                                    ..default()
                                },
                                TextColor(TITLE_COLOR),
                            ));
                        });

                    left_panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: percent(100),
                                ..default()
                            },
                        ))
                        .with_children(|menu_list| {
                            for (text, action) in &buttons_data {
                                menu_list
                                    .spawn((
                                        Button,
                                        Node {
                                            width: percent(100),
                                            height: px(55),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(px(3)),
                                            border_radius: BorderRadius::all(px(28)),
                                            margin: UiRect::vertical(px(12)),
                                            ..default()
                                        },
                                        BackgroundColor(CAPSULE_NORMAL),
                                        BorderColor::all(CAPSULE_BORDER),
                                        MenuButtonTag(*action),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(*text),
                                            TextFont {
                                                font_size: FontSize::Px(20.0),
                                                ..default()
                                            },
                                            TextColor(TEXT_COLOR),
                                        ));
                                    });
                            }
                        });
                });

            parent
                .spawn((
                    Node {
                        width: percent(65),
                        height: percent(100),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        padding: UiRect::bottom(px(30)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.06, 0.10)),
                ))
                .with_children(|right_panel| {
                    right_panel
                        .spawn((
                            Node {
                                padding: UiRect::all(px(20)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        ))
                        .with_children(|slogan_area| {
                            slogan_area.spawn((
                                Text::new("江山为棋，情为局"),
                                TextFont {
                                    font_size: FontSize::Px(24.0),
                                    ..default()
                                },
                                TextColor(SLOGAN_COLOR),
                            ));
                        });
                });
        })
        .id();

    commands.insert_resource(MainMenuEntities { root });
}

pub fn main_menu_interaction(
    mut next_state: ResMut<NextState<FmvAppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButtonTag),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, tag) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                *color = CAPSULE_HOVER.into();
            }
            Interaction::Pressed => {
                *color = CAPSULE_HOVER.into();
                match tag.0 {
                    MenuAction::NewGame => next_state.set(FmvAppState::InGameHud),
                    MenuAction::Continue => next_state.set(FmvAppState::InGameHud),
                    MenuAction::Chapters => next_state.set(FmvAppState::ChapterSelect),
                    MenuAction::Settings => next_state.set(FmvAppState::Settings),
                    MenuAction::Quit => std::process::exit(0),
                }
            }
            Interaction::None => {
                *color = CAPSULE_NORMAL.into();
            }
        }
    }
}

pub fn cleanup_main_menu(
    mut commands: Commands,
    entities: Res<MainMenuEntities>,
) {
    commands.entity(entities.root).despawn();
    commands.remove_resource::<MainMenuEntities>();
}