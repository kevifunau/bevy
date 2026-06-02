use bevy::prelude::*;

use crate::blackboard::BlackboardResource;
use crate::data_schema::VideoSimulatedPlayback;
use crate::director::CurrentNodeResource;
use crate::FmvAppState;

const HUD_BG: Color = Color::srgb(0.02, 0.02, 0.04);
const BACK_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
const BACK_HOVER: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const TEXT_COLOR: Color = Color::srgb(0.7, 0.7, 0.85);
const NODE_TITLE_COLOR: Color = Color::srgb(0.85, 0.85, 0.95);
const BLACKBOARD_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);

#[derive(Component)]
pub struct InGameHudRoot;

#[derive(Component)]
pub struct BackButtonTag;

#[derive(Component)]
pub struct NodeTitleText;

#[derive(Component)]
pub struct VideoTimeText;

#[derive(Component)]
pub struct BlackboardText;

#[derive(Resource)]
pub struct InGameHudEntities {
    pub root: Entity,
}

pub fn setup_in_game_hud(mut commands: Commands, current: Res<CurrentNodeResource>) {
    let title = current
        .node_data
        .as_ref()
        .map(|n| n.node_title.clone())
        .unwrap_or("Loading...".into());

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(HUD_BG),
            InGameHudRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(45),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::left(px(20)),
                        ..default()
                    },
                ))
                .with_children(|top_bar| {
                    top_bar
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::all(px(10)),
                                border_radius: BorderRadius::all(px(12)),
                                ..default()
                            },
                            BackgroundColor(BACK_BG),
                            BackButtonTag,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("← BACK"),
                                TextFont {
                                    font_size: FontSize::Px(14.0),
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });

                    top_bar.spawn((
                        Node {
                            margin: UiRect::left(px(20)),
                            ..default()
                        },
                    ))
                    .with_children(|area| {
                        area.spawn((
                            Text::new(title),
                            TextFont {
                                font_size: FontSize::Px(18.0),
                                ..default()
                            },
                            TextColor(NODE_TITLE_COLOR),
                            NodeTitleText,
                        ));
                    });
                });

            parent
                .spawn((
                    Node {
                        width: percent(100),
                        flex_grow: 1.0,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.03, 0.03, 0.05)),
                ))
                .with_children(|center| {
                    center
                        .spawn((
                            Node {
                                padding: UiRect::all(px(40)),
                                border: UiRect::all(px(2)),
                                border_radius: BorderRadius::all(px(15)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            BorderColor::all(Color::srgb(0.15, 0.15, 0.25)),
                        ))
                        .with_children(|video_box| {
                            video_box.spawn((
                                Text::new("🎬 FMV Video Player"),
                                TextFont {
                                    font_size: FontSize::Px(28.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.65)),
                            ));
                            video_box.spawn((
                                Text::new("0:00 / 0:30"),
                                TextFont {
                                    font_size: FontSize::Px(16.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.4, 0.4, 0.5)),
                                VideoTimeText,
                            ));
                        });
                });

            parent
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(50),
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(px(20)),
                        ..default()
                    },
                    BackgroundColor(BLACKBOARD_BG),
                ))
                .with_children(|bottom| {
                    bottom.spawn((
                        Text::new("Gold: 100 | Rep: 5 | Empress: 0"),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.65)),
                        BlackboardText,
                    ));
                });
        })
        .id();

    commands.insert_resource(InGameHudEntities { root });
}

pub fn in_game_hud_interaction(
    mut next_state: ResMut<NextState<FmvAppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&BackButtonTag>),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text, Or<(With<VideoTimeText>, With<NodeTitleText>, With<BlackboardText>)>>,
    current: Res<CurrentNodeResource>,
    playback_query: Query<&VideoSimulatedPlayback>,
    bb: Res<BlackboardResource>,
) {
    for (interaction, mut color, back_tag) in &mut interaction_query {
        if back_tag.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    next_state.set(FmvAppState::MainMenu);
                }
                Interaction::Hovered => {
                    *color = BACK_HOVER.into();
                }
                Interaction::None => {
                    *color = BACK_BG.into();
                }
            }
        }
    }

    if current.is_changed() {
        if let Some(node_data) = &current.node_data {
            for mut text in &mut text_query {
                if text.0.contains("Loading") || text.0.contains("初入宫廷") || text.0.contains("女帝选秀") || text.0.contains("深宫谍影") || text.0.contains("太子之路") || text.0.contains("暗中观望") || text.0.contains("皇后之路") {
                    **text = node_data.node_title.clone();
                }
            }
        }
    }

    for playback in &playback_query {
        let elapsed_min = (playback.elapsed / 60.0) as i32;
        let elapsed_sec = (playback.elapsed % 60.0) as i32;
        let duration_min = (playback.duration / 60.0) as i32;
        let duration_sec = (playback.duration % 60.0) as i32;
        let time_str = format!("{elapsed_min}:{elapsed_sec:02} / {duration_min}:{duration_sec:02}");
        for mut text in &mut text_query {
            if text.0.contains("/") {
                **text = time_str.clone();
            }
        }
    }

    let gold = bb.get("player.gold").unwrap_or(0.0) as i32;
    let rep = bb.get("player.reputation").unwrap_or(0.0) as i32;
    let empress = bb.get("npc.empress_favor").unwrap_or(0.0) as i32;
    let bb_str = format!("Gold: {gold} | Rep: {rep} | Empress: {empress}");
    for mut text in &mut text_query {
        if text.0.contains("Gold:") {
            **text = bb_str.clone();
        }
    }
}

pub fn cleanup_in_game_hud(
    mut commands: Commands,
    entities: Res<InGameHudEntities>,
) {
    commands.entity(entities.root).despawn();
    commands.remove_resource::<InGameHudEntities>();
}