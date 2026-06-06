//! Renders the official `examples/ui/ui_scaling.rs` UI through BUI JSON.
//!
//! Run with:
//! `cargo run --example uiParse_ui_scaling`

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::AiUiPlugin;
use core::time::Duration;

#[path = "../../auto_screenshot.rs"]
mod auto_screenshot;

const SCALE_TIME: u64 = 400;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(TargetScale {
            start_scale: 1.0,
            target_scale: 1.0,
            target_time: Timer::new(Duration::from_millis(SCALE_TIME), TimerMode::Once),
        })
        .add_plugins(AiUiPlugin::from_path(bui_json_path("ui_scaling.ir.json")))
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (change_scaling, apply_scaling.after(change_scaling)),
        );
    auto_screenshot::install(&mut app);
    app.run();
}

#[derive(Resource)]
struct TargetScale {
    start_scale: f32,
    target_scale: f32,
    target_time: Timer,
}

impl TargetScale {
    fn set_scale(&mut self, scale: f32) {
        self.start_scale = self.current_scale();
        self.target_scale = scale;
        self.target_time.reset();
    }

    fn current_scale(&self) -> f32 {
        let completion = self.target_time.fraction();
        let t = ease_in_expo(completion);
        self.start_scale.lerp(self.target_scale, t)
    }

    fn tick(&mut self, delta: Duration) -> &Self {
        self.target_time.tick(delta);
        self
    }

    fn already_completed(&self) -> bool {
        self.target_time.is_finished() && !self.target_time.just_finished()
    }
}

fn bui_json_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest")
        .join("uiParse_TestSet")
        .join("uiParse_ui_scaling")
        .join(file_name)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn change_scaling(input: Res<ButtonInput<KeyCode>>, mut ui_scale: ResMut<TargetScale>) {
    if input.just_pressed(KeyCode::ArrowUp) {
        let scale = (ui_scale.target_scale * 2.0).min(8.);
        ui_scale.set_scale(scale);
        info!("Scaling up! Scale: {}", ui_scale.target_scale);
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        let scale = (ui_scale.target_scale / 2.0).max(1. / 8.);
        ui_scale.set_scale(scale);
        info!("Scaling down! Scale: {}", ui_scale.target_scale);
    }
}

fn apply_scaling(
    time: Res<Time>,
    mut target_scale: ResMut<TargetScale>,
    mut ui_scale: ResMut<UiScale>,
) {
    if target_scale.tick(time.delta()).already_completed() {
        return;
    }

    ui_scale.0 = target_scale.current_scale();
}

fn ease_in_expo(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else {
        ops::powf(2.0f32, 5.0 * x - 5.0)
    }
}
