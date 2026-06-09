use bevy::prelude::*;
use bevy::ui::InteractionDisabled;

use crate::blackboard::BlackboardResource;
use crate::data_schema::StoryData;
use crate::FmvAppState;

const PANEL_BG: Color = Color::srgba(0.0, 0.0, 0.05, 0.85);
const CHAPTER_HIGHLIGHT: Color = Color::srgb(0.18, 0.22, 0.35);
const CHAPTER_NORMAL: Color = Color::srgb(0.08, 0.08, 0.14);
const CHAPTER_LOCKED: Color = Color::srgb(0.06, 0.06, 0.06);
const CHAPTER_LOCKED_TEXT: Color = Color::srgb(0.4, 0.4, 0.4);
const CHAPTER_TEXT: Color = Color::srgb(0.85, 0.85, 0.95);
const CTA_BG: Color = Color::srgb(0.15, 0.25, 0.45);
const CTA_HOVER: Color = Color::srgb(0.25, 0.35, 0.55);
const SECONDARY_BG: Color = Color::srgb(0.08, 0.12, 0.20);
#[allow(dead_code)]
const SECONDARY_HOVER: Color = Color::srgb(0.15, 0.20, 0.30);
const NAV_BG: Color = Color::srgb(0.06, 0.08, 0.14);
const PROGRESS_BG: Color = Color::srgb(0.08, 0.08, 0.12);
const PROGRESS_FILL: Color = Color::srgb(0.3, 0.55, 0.85);
const BORDER_COLOR: Color = Color::srgb(0.3, 0.5, 0.8);

#[derive(Component)]
pub struct ChapterSelectRoot;

#[derive(Component)]
pub struct ChapterItemTag(pub String);

#[derive(Component)]
pub struct BackButtonTag;

#[derive(Component)]
pub struct EnterChapterButtonTag;

#[derive(Component)]
pub struct ProgressBarFill;

#[derive(Component)]
pub struct ChapterPreviewText;

#[derive(Resource)]
pub struct ChapterSelectState {
    pub root: Entity,
    pub selected_chapter_id: String,
}

pub fn setup_chapter_select(
    mut commands: Commands,
    story: Res<StoryData>,
    _bb: Res<BlackboardResource>,
) {
    let first_unlocked = story
        .chapters
        .iter()
        .find(|c| c.status != "locked")
        .map(|c| c.id.clone())
        .unwrap_or_default();

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.03, 0.06)),
            ChapterSelectRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(50),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(px(20)),
                        ..default()
                    },
                    BackgroundColor(NAV_BG),
                ))
                .with_children(|nav| {
                    nav.spawn((
                        Button,
                        Node {
                            padding: UiRect::all(px(12)),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::all(px(8)),
                            ..default()
                        },
                        BackgroundColor(NAV_BG),
                        BorderColor::all(BORDER_COLOR),
                        BackButtonTag,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("← BACK"),
                            TextFont {
                                font_size: FontSize::Px(16.0),
                                ..default()
                            },
                            TextColor(CHAPTER_TEXT),
                        ));
                    });

                    nav.spawn((
                        Text::new("CHAPTER SELECT"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(CHAPTER_TEXT),
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Row,
                        flex_grow: 1.0,
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                ))
                .with_children(|content| {
                    content
                        .spawn((Node {
                            width: percent(35),
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(px(15)),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },))
                        .with_children(|list| {
                            for chapter in &story.chapters {
                                let is_locked = chapter.status == "locked";
                                let is_selected = chapter.id == first_unlocked;
                                let bg = if is_locked {
                                    CHAPTER_LOCKED
                                } else if is_selected {
                                    CHAPTER_HIGHLIGHT
                                } else {
                                    CHAPTER_NORMAL
                                };
                                let border = if is_selected {
                                    BORDER_COLOR
                                } else {
                                    Color::srgb(0.2, 0.2, 0.3)
                                };
                                let icon = if is_locked {
                                    "🔒"
                                } else if chapter.status == "completed" {
                                    "★"
                                } else {
                                    "▶"
                                };
                                let status_text = if is_locked {
                                    "Locked".to_string()
                                } else {
                                    format!("{}%", chapter.progress as i32)
                                };

                                let mut btn = list.spawn((
                                    Button,
                                    Node {
                                        width: percent(100),
                                        height: px(70),
                                        padding: UiRect::all(px(12)),
                                        border: UiRect::all(px(2)),
                                        border_radius: BorderRadius::all(px(10)),
                                        margin: UiRect::vertical(px(6)),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(bg),
                                    BorderColor::all(border),
                                    ChapterItemTag(chapter.id.clone()),
                                ));

                                if is_locked {
                                    btn.insert(InteractionDisabled);
                                }

                                btn.with_children(|item| {
                                    let ch_num = chapter.id.replace("chapter_", "");
                                    item.spawn((
                                        Text::new(format!(
                                            "{icon} 第{ch_num}章：{}",
                                            chapter.title
                                        )),
                                        TextFont {
                                            font_size: FontSize::Px(16.0),
                                            ..default()
                                        },
                                        TextColor(if is_locked {
                                            CHAPTER_LOCKED_TEXT
                                        } else {
                                            CHAPTER_TEXT
                                        }),
                                    ));
                                    item.spawn((
                                        Text::new(format!("Status: {status_text}")),
                                        TextFont {
                                            font_size: FontSize::Px(12.0),
                                            ..default()
                                        },
                                        TextColor(if is_locked {
                                            CHAPTER_LOCKED_TEXT
                                        } else {
                                            Color::srgb(0.6, 0.6, 0.7)
                                        }),
                                    ));
                                });
                            }
                        });

                    content
                        .spawn((Node {
                            width: percent(65),
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(px(20)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },))
                        .with_children(|right| {
                            right
                                .spawn((
                                    Node {
                                        width: percent(100),
                                        height: percent(55),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(px(3)),
                                        border_radius: BorderRadius::all(px(15)),
                                        margin: UiRect::bottom(px(20)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
                                    BorderColor::all(BORDER_COLOR),
                                    ChapterPreviewText,
                                ))
                                .with_children(|preview| {
                                    preview.spawn((
                                        Text::new("[ Chapter Preview ]"),
                                        TextFont {
                                            font_size: FontSize::Px(20.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.4, 0.4, 0.5)),
                                    ));
                                });

                            right
                                .spawn((
                                    Node {
                                        width: percent(100),
                                        height: px(20),
                                        border: UiRect::all(px(2)),
                                        border_radius: BorderRadius::all(px(10)),
                                        margin: UiRect::bottom(px(30)),
                                        ..default()
                                    },
                                    BackgroundColor(PROGRESS_BG),
                                    BorderColor::all(Color::srgb(0.2, 0.2, 0.3)),
                                    ProgressBarFill,
                                ))
                                .with_children(|bar| {
                                    let progress = story
                                        .chapters
                                        .iter()
                                        .find(|c| c.id == first_unlocked)
                                        .map(|c| c.progress)
                                        .unwrap_or(0.0);
                                    bar.spawn((
                                        Node {
                                            width: percent(progress),
                                            height: percent(100),
                                            border_radius: BorderRadius::all(px(10)),
                                            ..default()
                                        },
                                        BackgroundColor(PROGRESS_FILL),
                                    ));
                                });

                            right
                                .spawn((Node {
                                    width: percent(100),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },))
                                .with_children(|buttons| {
                                    buttons
                                        .spawn((
                                            Button,
                                            Node {
                                                width: px(160),
                                                height: px(50),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(px(3)),
                                                border_radius: BorderRadius::all(px(25)),
                                                margin: UiRect::horizontal(px(15)),
                                                ..default()
                                            },
                                            BackgroundColor(CTA_BG),
                                            BorderColor::all(BORDER_COLOR),
                                            EnterChapterButtonTag,
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new("▶ ENTER CHAPTER"),
                                                TextFont {
                                                    font_size: FontSize::Px(18.0),
                                                    ..default()
                                                },
                                                TextColor(CHAPTER_TEXT),
                                            ));
                                        });

                                    buttons
                                        .spawn((
                                            Button,
                                            Node {
                                                width: px(140),
                                                height: px(50),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(px(3)),
                                                border_radius: BorderRadius::all(px(25)),
                                                margin: UiRect::horizontal(px(15)),
                                                ..default()
                                            },
                                            BackgroundColor(SECONDARY_BG),
                                            BorderColor::all(Color::srgb(0.2, 0.3, 0.5)),
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new("➔ STORY TREE"),
                                                TextFont {
                                                    font_size: FontSize::Px(16.0),
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.6, 0.7, 0.85)),
                                            ));
                                        });
                                });
                        });
                });
        })
        .id();

    commands.insert_resource(ChapterSelectState {
        root,
        selected_chapter_id: first_unlocked,
    });
}

pub fn chapter_select_interaction(
    mut next_state: ResMut<NextState<FmvAppState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ChapterItemTag>,
            Option<&BackButtonTag>,
            Option<&EnterChapterButtonTag>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut chapter_state: ResMut<ChapterSelectState>,
    story: Res<StoryData>,
) {
    for (interaction, mut color, item_tag, back_tag, enter_tag) in &mut interaction_query {
        if let Some(tag) = item_tag {
            match *interaction {
                Interaction::Pressed => {
                    chapter_state.selected_chapter_id = tag.0.clone();
                }
                Interaction::Hovered => {
                    *color = CHAPTER_HIGHLIGHT.into();
                }
                Interaction::None => {
                    let is_selected = tag.0 == chapter_state.selected_chapter_id;
                    let chapter = story.chapters.iter().find(|c| c.id == tag.0);
                    let is_locked = chapter.map(|c| c.status == "locked").unwrap_or(true);
                    *color = if is_locked {
                        CHAPTER_LOCKED
                    } else if is_selected {
                        CHAPTER_HIGHLIGHT
                    } else {
                        CHAPTER_NORMAL
                    }
                    .into();
                }
            }
        }
        if back_tag.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    next_state.set(FmvAppState::MainMenu);
                }
                Interaction::Hovered => {
                    *color = Color::srgb(0.12, 0.15, 0.25).into();
                }
                Interaction::None => {
                    *color = NAV_BG.into();
                }
            }
        }
        if enter_tag.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    next_state.set(FmvAppState::InGameHud);
                }
                Interaction::Hovered => {
                    *color = CTA_HOVER.into();
                }
                Interaction::None => {
                    *color = CTA_BG.into();
                }
            }
        }
    }
}

pub fn update_selection_highlight(
    chapter_state: Res<ChapterSelectState>,
    mut query: Query<(&mut BackgroundColor, &mut BorderColor, &ChapterItemTag), With<Button>>,
    story: Res<StoryData>,
) {
    if chapter_state.is_changed() {
        for (mut bg, mut border, tag) in &mut query {
            let is_selected = tag.0 == chapter_state.selected_chapter_id;
            let chapter = story.chapters.iter().find(|c| c.id == tag.0);
            let is_locked = chapter.map(|c| c.status == "locked").unwrap_or(true);
            *bg = if is_locked {
                CHAPTER_LOCKED
            } else if is_selected {
                CHAPTER_HIGHLIGHT
            } else {
                CHAPTER_NORMAL
            }
            .into();
            *border = BorderColor::all(if is_selected {
                BORDER_COLOR
            } else {
                Color::srgb(0.2, 0.2, 0.3)
            });
        }
    }
}

pub fn cleanup_chapter_select(mut commands: Commands, state: Res<ChapterSelectState>) {
    commands.entity(state.root).despawn();
    commands.remove_resource::<ChapterSelectState>();
}
