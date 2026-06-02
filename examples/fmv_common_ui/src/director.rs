use bevy::prelude::*;
use std::collections::HashMap;

use crate::blackboard::BlackboardResource;
use crate::data_schema::*;
use crate::expression_eval::evaluate_condition;
use crate::FmvAppState;

pub struct DirectorPlugin;

impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UiRenderRequestMessage>()
            .add_message::<OnComponentClickedMessage>()
            .add_message::<TimeoutMessage>()
            .insert_resource(CurrentNodeResource::default())
            .add_systems(Startup, load_story_data)
            .add_systems(
                OnEnter(FmvAppState::InGameHud),
                (setup_current_node, setup_video_simulator),
            )
            .add_systems(
                Update,
                (
                    video_simulator_tick.run_if(in_state(FmvAppState::InGameHud)),
                    check_interaction_trigger.run_if(in_state(FmvAppState::InGameHud)),
                    handle_timeout.run_if(in_state(FmvAppState::InGameHud)),
                    handle_component_clicked.run_if(in_state(FmvAppState::InGameHud)),
                )
                    .chain(),
            )
            .add_systems(OnExit(FmvAppState::InGameHud), cleanup_current_node);
    }
}

#[derive(Resource, Default)]
pub struct CurrentNodeResource {
    pub node_id: Option<String>,
    pub node_data: Option<StoryNodeData>,
    pub interaction_shown: bool,
    pub ui_spawned: bool,
    pub timeout_started: bool,
}

#[derive(Message, Clone)]
#[allow(dead_code)]
pub struct UiRenderRequestMessage {
    pub node_id: String,
    pub components: Vec<InteractionComponentData>,
}

#[derive(Message, Clone)]
pub struct OnComponentClickedMessage {
    pub target_node: String,
    pub mutations: Vec<String>,
}

#[derive(Message, Clone)]
#[allow(dead_code)]
pub struct TimeoutMessage {
    pub target_node: Option<String>,
}

fn load_story_data(mut commands: Commands) {
    let json_str = include_str!("../assets/story/test_story.json");
    let story: StoryData = serde_json::from_str(json_str).unwrap_or_else(|e| {
        panic!("Failed to parse story JSON: {e}");
    });

    let mut nodes_map: HashMap<String, StoryNodeData> = HashMap::new();
    for node in &story.nodes {
        nodes_map.insert(node.node_id.clone(), node.clone());
    }

    let mut bb = BlackboardResource::from_initial(&story.initial_blackboard);
    for chapter in &story.chapters {
        let key = format!("chapters.{}.progress", chapter.id);
        bb.set(&key, chapter.progress);
        let status_key = format!("chapters.{}.status", chapter.id);
        let status_val = match chapter.status.as_str() {
            "completed" => 1.0,
            "in_progress" => 0.5,
            "locked" => 0.0,
            _ => 0.0,
        };
        bb.set(&status_key, status_val);
    }

    commands.insert_resource(story);
    commands.insert_resource(StoryNodesMap(nodes_map));
    commands.insert_resource(bb);
}

#[derive(Resource)]
pub struct StoryNodesMap(pub HashMap<String, StoryNodeData>);

fn setup_current_node(
    mut current: ResMut<CurrentNodeResource>,
    story: Res<StoryData>,
) {
    let start_node = story.nodes.first().unwrap().node_id.clone();
    current.node_id = Some(start_node.clone());
    current.node_data = story.nodes.first().cloned();
    current.interaction_shown = false;
    current.ui_spawned = false;
    current.timeout_started = false;
}

fn setup_video_simulator(mut commands: Commands, current: Res<CurrentNodeResource>) {
    if let Some(node_data) = &current.node_data {
        let mode = match node_data.playback_mode.as_str() {
            "ONCE_AND_STOP" => PlaybackMode::OnceAndStop,
            "ONCE_AND_LOOP" => PlaybackMode::OnceAndLoop,
            "INFINITE_LOOP" => PlaybackMode::InfiniteLoop,
            _ => PlaybackMode::OnceAndStop,
        };
        commands.spawn((
            VideoSimulatedPlayback {
                elapsed: 0.0,
                duration: 30.0,
                mode,
                finished: false,
            },
            CurrentNodeMarker,
        ));
    }
}

fn video_simulator_tick(
    time: Res<Time>,
    mut query: Query<&mut VideoSimulatedPlayback>,
    mut current: ResMut<CurrentNodeResource>,
) {
    for mut playback in &mut query {
        if !playback.finished {
            playback.elapsed += time.delta_secs() as f64;
            if playback.elapsed >= playback.duration {
                match playback.mode {
                    PlaybackMode::OnceAndStop => {
                        playback.finished = true;
                    }
                    PlaybackMode::OnceAndLoop | PlaybackMode::InfiniteLoop => {
                        playback.elapsed = 0.0;
                    }
                }
            }
            if playback.finished || playback.elapsed >= playback.duration {
                if !current.interaction_shown {
                    current.interaction_shown = true;
                }
            }
        }
    }
}

fn check_interaction_trigger(
    mut ev_render: MessageWriter<UiRenderRequestMessage>,
    mut current: ResMut<CurrentNodeResource>,
    bb: Res<BlackboardResource>,
    mut commands: Commands,
    query: Query<Entity, With<VideoSimulatedPlayback>>,
) {
    if !current.interaction_shown || current.ui_spawned {
        return;
    }
    let node_data = match &current.node_data {
        Some(data) => data.clone(),
        None => return,
    };
    let int_type = node_data.interaction.interaction_type.as_str();
    if int_type == "NONE" {
        current.ui_spawned = true;
        return;
    }

    let trigger_time = node_data.interaction.trigger_time;
    if trigger_time < 0.0 || current.interaction_shown {
        let mut filtered_components: Vec<InteractionComponentData> = Vec::new();
        for comp in &node_data.interaction.components {
            if let Some(cond) = &comp.behavior.condition {
                if !evaluate_condition(cond, &bb) {
                    continue;
                }
            }
            filtered_components.push(comp.clone());
        }

        if !filtered_components.is_empty() {
            ev_render.write(UiRenderRequestMessage {
                node_id: node_data.node_id.clone(),
                components: filtered_components,
            });

            if node_data.interaction.timeout > 0.0 && !current.timeout_started {
                current.timeout_started = true;
                for entity in &query {
                    commands.entity(entity).insert(InteractionTimeoutTimer {
                        remaining: node_data.interaction.timeout,
                        target_node: node_data.interaction.timeout_target_node.clone(),
                    });
                }
            }
        }
        current.ui_spawned = true;
    }
}

fn handle_timeout(
    time: Res<Time>,
    mut query: Query<(Entity, &mut InteractionTimeoutTimer)>,
    mut ev_timeout: MessageWriter<TimeoutMessage>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut query {
        timer.remaining -= time.delta_secs() as f64;
        if timer.remaining <= 0.0 {
            ev_timeout.write(TimeoutMessage {
                target_node: timer.target_node.clone(),
            });
            commands.entity(entity).remove::<InteractionTimeoutTimer>();
        }
    }
}

fn handle_component_clicked(
    mut ev_click: MessageReader<OnComponentClickedMessage>,
    mut bb: ResMut<BlackboardResource>,
    mut current: ResMut<CurrentNodeResource>,
    nodes_map: Res<StoryNodesMap>,
) {
    for event in ev_click.read() {
        bb.apply_mutations(&event.mutations);

        let target = event.target_node.clone();
        if let Some(node_data) = nodes_map.0.get(&target) {
            current.node_id = Some(target);
            current.node_data = Some(node_data.clone());
            current.interaction_shown = false;
            current.ui_spawned = false;
            current.timeout_started = false;
        }
    }
}

fn cleanup_current_node(
    mut commands: Commands,
    query: Query<Entity, With<CurrentNodeMarker>>,
    mut current: ResMut<CurrentNodeResource>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    current.node_id = None;
    current.node_data = None;
    current.interaction_shown = false;
    current.ui_spawned = false;
    current.timeout_started = false;
}