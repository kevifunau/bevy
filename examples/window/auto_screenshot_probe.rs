//! Captures a simple offscreen UI scene automatically to debug screenshot readback.

use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};

#[derive(Resource)]
struct ScreenshotPath(String);

#[derive(Resource, Clone)]
struct ProbeTarget {
    image: Handle<Image>,
    camera: Entity,
}

#[derive(Default)]
struct ProbeState {
    frames: u8,
    requested: bool,
}

fn main() {
    let path = std::env::var("AUTO_SCREENSHOT_PROBE_PATH")
        .unwrap_or_else(|_| "/tmp/auto_screenshot_probe.png".to_string());

    App::new()
        .insert_resource(ScreenshotPath(path))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (route_root_to_probe_target, capture_when_ready))
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_target_texture(2560, 1440, TextureFormat::Rgba8UnormSrgb, None);
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let image_handle = images.add(image);
    let camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(image_handle.clone().into()),
        ))
        .id();
    commands.insert_resource(ProbeTarget {
        image: image_handle,
        camera,
    });

    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.18, 0.24)),
        children![(
            Node {
                width: px(420.0),
                height: px(220.0),
                border: UiRect::all(px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.92, 0.78, 0.44)),
            BorderColor::all(Color::srgb(0.38, 0.22, 0.08)),
            children![(
                Text::new("AUTO SCREENSHOT PROBE"),
                TextFont {
                    font_size: FontSize::Px(42.0),
                    ..default()
                },
                TextColor(Color::BLACK),
            )]
        )],
    ));
}

fn route_root_to_probe_target(
    probe_target: Res<ProbeTarget>,
    mut commands: Commands,
    roots: Query<Entity, (With<Node>, With<BackgroundColor>, Without<UiTargetCamera>)>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    let Some(root) = roots.iter().next() else {
        return;
    };

    commands
        .entity(root)
        .insert(UiTargetCamera(probe_target.camera));
    *done = true;
}

fn capture_when_ready(
    mut commands: Commands,
    screenshot_path: Res<ScreenshotPath>,
    probe_target: Res<ProbeTarget>,
    root_nodes: Query<&ComputedNode, With<BackgroundColor>>,
    mut state: Local<ProbeState>,
) {
    if state.requested {
        return;
    }

    let layout_ready = root_nodes
        .iter()
        .any(|node| node.size().x > 0.0 && node.size().y > 0.0);
    if !layout_ready {
        state.frames = 0;
        return;
    }

    state.frames += 1;
    if state.frames < 20 {
        return;
    }

    let path = screenshot_path.0.clone();
    let image = probe_target.image.clone();
    commands.spawn(Screenshot::image(image)).observe(
        move |captured: On<ScreenshotCaptured>, mut app_exit_writer: MessageWriter<AppExit>| {
            save_to_disk(path.clone())(captured);
            app_exit_writer.write(AppExit::Success);
        },
    );
    state.requested = true;
}
