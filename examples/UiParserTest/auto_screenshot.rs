use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};

const SCREENSHOT_ENV: &str = "BUI_SCREENSHOT_PATH";

#[derive(Resource)]
struct AutoScreenshotPath(PathBuf);

#[derive(Clone, Copy)]
struct AutoScreenshotProfile {
    width: u32,
    height: u32,
}

impl AutoScreenshotProfile {
    const DEFAULT: Self = Self {
        width: 2048,
        height: 1152,
    };

    const HERO_GAME_UI: Self = Self {
        width: 1728,
        height: 888,
    };
}

#[derive(Resource, Clone, Copy)]
struct ActiveAutoScreenshotProfile(AutoScreenshotProfile);

#[derive(Resource, Clone)]
struct AutoScreenshotTarget {
    image: Handle<Image>,
    camera: Entity,
    container: Entity,
}

#[derive(Default)]
struct AutoScreenshotState {
    frames_after_layout: u8,
    requested: bool,
}

#[derive(Default)]
struct AutoScreenshotTargetRoutingState {
    routed: bool,
}

pub fn install(app: &mut App) {
    register_optional_auto_screenshot(app);
    app.add_systems(Startup, setup_auto_screenshot_target);
    app.add_systems(
        Update,
        (
            route_root_ui_to_auto_screenshot_target_system,
            auto_capture_screenshot_system,
        ),
    );
}

fn register_optional_auto_screenshot(app: &mut App) {
    if let Ok(path) = std::env::var(SCREENSHOT_ENV) {
        app.insert_resource(AutoScreenshotPath(PathBuf::from(path)));
        app.insert_resource(ActiveAutoScreenshotProfile(detect_auto_screenshot_profile()));
    }
}

fn detect_auto_screenshot_profile() -> AutoScreenshotProfile {
    let example_name = std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| stem.to_string_lossy().to_string()))
        .unwrap_or_default();

    if example_name.starts_with("hero_game_ui") {
        AutoScreenshotProfile::HERO_GAME_UI
    } else {
        AutoScreenshotProfile::DEFAULT
    }
}

fn setup_auto_screenshot_target(
    mut commands: Commands,
    screenshot_path: Option<Res<AutoScreenshotPath>>,
    screenshot_profile: Option<Res<ActiveAutoScreenshotProfile>>,
    mut images: ResMut<Assets<Image>>,
) {
    if screenshot_path.is_none() {
        return;
    }

    let profile = screenshot_profile
        .map(|profile| profile.0)
        .unwrap_or(AutoScreenshotProfile::DEFAULT);

    if profile.width == AutoScreenshotProfile::DEFAULT.width
        && profile.height == AutoScreenshotProfile::DEFAULT.height
    {
        return;
    }

    let mut image = Image::new_target_texture(
        profile.width,
        profile.height,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let image_handle = images.add(image);
    let container = commands
        .spawn((
            Name::new("bui_auto_screenshot_root"),
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(67, 41, 30, 255)),
        ))
        .id();
    let camera = commands
        .spawn((
            Name::new("bui_auto_screenshot_camera"),
            Camera2d,
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(image_handle.clone().into()),
        ))
        .id();

    commands.insert_resource(AutoScreenshotTarget {
        image: image_handle,
        camera,
        container,
    });
}

fn route_root_ui_to_auto_screenshot_target_system(
    mut commands: Commands,
    screenshot_target: Option<Res<AutoScreenshotTarget>>,
    root_nodes: Query<Entity, (With<Node>, Without<ChildOf>)>,
    mut state: Local<AutoScreenshotTargetRoutingState>,
) {
    if state.routed {
        return;
    }

    let Some(screenshot_target) = screenshot_target else {
        return;
    };

    let Some(root) = root_nodes
        .iter()
        .find(|entity| *entity != screenshot_target.container)
    else {
        return;
    };

    commands
        .entity(screenshot_target.container)
        .insert(UiTargetCamera(screenshot_target.camera));
    commands.entity(screenshot_target.container).add_child(root);
    state.routed = true;
}

fn auto_capture_screenshot_system(
    mut commands: Commands,
    screenshot_path: Option<Res<AutoScreenshotPath>>,
    screenshot_target: Option<Res<AutoScreenshotTarget>>,
    root_nodes: Query<(Entity, &ComputedNode), With<Node>>,
    mut state: Local<AutoScreenshotState>,
) {
    let Some(screenshot_path) = screenshot_path else {
        return;
    };

    if state.requested {
        return;
    }

    let Some((root, computed_root)) = root_nodes
        .iter()
        .find(|(entity, computed)| {
            computed.size().x > 0.0
                && computed.size().y > 0.0
                && screenshot_target
                    .as_ref()
                    .map(|target| *entity != target.container)
                    .unwrap_or(true)
        })
    else {
        state.frames_after_layout = 0;
        return;
    };

    let _ = root;

    if computed_root.size().x <= 0.0 || computed_root.size().y <= 0.0 {
        state.frames_after_layout = 0;
        return;
    }

    state.frames_after_layout += 1;
    if state.frames_after_layout < 30 {
        return;
    }

    let screenshot_path = screenshot_path.0.clone();
    if let Some(screenshot_target) = screenshot_target {
        let screenshot_image = screenshot_target.image.clone();
        commands.spawn(Screenshot::image(screenshot_image)).observe(
            move |captured: On<ScreenshotCaptured>, mut app_exit_writer: MessageWriter<AppExit>| {
                save_to_disk(screenshot_path.clone())(captured);
                app_exit_writer.write(AppExit::Success);
            },
        );
    } else {
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>, mut app_exit_writer: MessageWriter<AppExit>| {
                save_to_disk(screenshot_path.clone())(captured);
                app_exit_writer.write(AppExit::Success);
            },
        );
    }
    state.requested = true;
}
