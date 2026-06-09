use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct StoryNodeComponent {
    pub node_id: String,
    pub node_title: String,
    pub video_uri: String,
    pub thumbnail_uri: String,
    pub playback_mode: PlaybackMode,
    pub interaction_type: InteractionType,
    pub trigger_time: f64,
    pub timeout: f64,
    pub timeout_target_node: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Deserialize, Reflect)]
#[reflect(Deserialize)]
pub enum PlaybackMode {
    #[default]
    OnceAndStop,
    OnceAndLoop,
    InfiniteLoop,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Deserialize, Reflect)]
#[reflect(Deserialize)]
pub enum InteractionType {
    #[default]
    None,
    Selection,
    Qte,
    InputSlider,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InteractionButtonComponent {
    pub component_id: String,
    pub visual_style: String,
    pub text: String,
    pub condition: Option<String>,
    pub on_click_target: String,
    pub mutations: Vec<String>,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct ActiveUI;

#[derive(Component)]
pub struct VideoSimulatedPlayback {
    pub elapsed: f64,
    pub duration: f64,
    pub mode: PlaybackMode,
    pub finished: bool,
}

#[derive(Resource, Deserialize)]
pub struct StoryData {
    pub nodes: Vec<StoryNodeData>,
    pub chapters: Vec<ChapterData>,
    #[serde(default)]
    pub initial_blackboard: serde_json::Value,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct StoryNodeData {
    pub node_id: String,
    pub node_title: String,
    pub asset_manifest: AssetManifestData,
    pub playback_mode: String,
    pub interaction: InteractionData,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct AssetManifestData {
    pub video_uri: String,
    pub thumbnail_uri: String,
}

#[derive(Deserialize, Clone)]
pub struct InteractionData {
    #[serde(rename = "type")]
    pub interaction_type: String,
    pub trigger_time: f64,
    pub timeout: f64,
    #[serde(default)]
    pub timeout_target_node: Option<String>,
    #[serde(default)]
    pub components: Vec<InteractionComponentData>,
}

#[derive(Deserialize, Clone)]
pub struct InteractionComponentData {
    pub component_id: String,
    #[serde(default)]
    pub visual_style: String,
    pub content: ContentData,
    pub behavior: BehaviorData,
}

#[derive(Deserialize, Clone)]
pub struct ContentData {
    pub text: String,
}

#[derive(Deserialize, Clone)]
pub struct BehaviorData {
    #[serde(default)]
    pub condition: Option<String>,
    pub on_click_target: String,
    #[serde(default)]
    pub mutations: Vec<String>,
}

#[derive(Resource, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ChapterData {
    pub id: String,
    pub title: String,
    pub start_node: String,
    pub status: String,
    pub progress: f64,
    pub thumbnail_uri: String,
}

#[derive(Component)]
pub struct CurrentNodeMarker;

#[derive(Component)]
pub struct InteractionTimeoutTimer {
    pub remaining: f64,
    pub target_node: Option<String>,
}
