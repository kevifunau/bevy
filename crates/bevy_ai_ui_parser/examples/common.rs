use std::path::{Path, PathBuf};
use std::{fs, io};

use bevy::app::{AppExit, PostUpdate};
use bevy::asset::{AssetPlugin, RenderAssetUsages, UnapprovedPathMode};
use bevy::camera::RenderTarget;
use bevy::ecs::hierarchy::ChildOf;
use bevy::image::Image;
use bevy::input_focus::InputFocus;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::text::EditableText;
use bevy::text::{FontSize, FontSource, TextColor, TextFont};
use bevy::{asset::io::AssetSourceBuilder, prelude::*};
use bevy_ai_ui_parser::{AiUiPlugin, BuiId, BuiLogicTags, BuiRootEntity, BuiTextInput};

/// Load and render a pre-compiled BUI IR JSON file with full interaction.
///
/// `ir_rel_path` is relative to `examples/`.
#[allow(dead_code)]
pub fn run(ir_rel_path: &str) {
    let mut app = App::new();
    let file_path = ui_test_path(ir_rel_path);
    configure_app(
        &mut app,
        AiUiPlugin::from_path(file_path.clone()),
        Some(asset_root_for_file(&file_path)),
    );
    app.run();
}

/// Load and render a BUI IR JSON file with the editor enabled (F8 to toggle).
///
/// `ir_rel_path` is relative to `examples/`.
#[allow(dead_code)]
pub fn run_with_editor(ir_rel_path: &str) {
    let mut app = App::new();
    let file_path = ui_test_path(ir_rel_path);
    configure_app(
        &mut app,
        AiUiPlugin::from_path_with_editor(file_path.clone()),
        Some(asset_root_for_file(&file_path)),
    );
    app.add_systems(Startup, spawn_editor_hotkey_overlay);
    app.run();
}

fn ui_test_path(rel_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(rel_path)
}

fn configure_app(app: &mut App, plugin: AiUiPlugin, asset_root: Option<PathBuf>) {
    register_optional_windows_fonts_source(app);
    register_optional_macos_fonts_source(app);
    register_optional_auto_screenshot(app);

    let mut default_plugins = DefaultPlugins.build();
    if let Some(asset_root) = asset_root {
        default_plugins = default_plugins.set(AssetPlugin {
            file_path: asset_root.to_string_lossy().to_string(),
            unapproved_path_mode: UnapprovedPathMode::Allow,
            ..Default::default()
        });
    }

    app.add_plugins(default_plugins)
        .add_plugins(plugin)
        .insert_resource(ClearColor(Color::srgb_u8(59, 40, 24)))
        .add_systems(Startup, (setup_camera, setup_auto_screenshot_target))
        .add_systems(
            Update,
            (
                button_feedback_system,
                log_bui_root_system,
                log_text_input_focus_system,
                log_text_input_value_system,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                route_bui_root_to_auto_screenshot_target_system,
                auto_capture_screenshot_system,
            ),
        );
}

fn register_optional_windows_fonts_source(app: &mut App) {
    const WINDOWS_FONTS: &str = "/mnt/c/Windows/Fonts";
    let windows_fonts = Path::new(WINDOWS_FONTS);

    if windows_fonts.exists() {
        app.register_asset_source(
            "windows_fonts",
            AssetSourceBuilder::platform_default(WINDOWS_FONTS, None),
        );
    }
}

fn register_optional_macos_fonts_source(app: &mut App) {
    const MACOS_FONTS: &str = "/System/Library/Fonts";
    const MACOS_SUPPLEMENTAL_FONTS: &str = "/System/Library/Fonts/Supplemental";

    let macos_fonts = Path::new(MACOS_FONTS);
    if macos_fonts.exists() {
        app.register_asset_source(
            "macos_fonts",
            AssetSourceBuilder::platform_default(MACOS_FONTS, None),
        );
    }

    let macos_supplemental_fonts = Path::new(MACOS_SUPPLEMENTAL_FONTS);
    if macos_supplemental_fonts.exists() {
        app.register_asset_source(
            "macos_supplemental_fonts",
            AssetSourceBuilder::platform_default(MACOS_SUPPLEMENTAL_FONTS, None),
        );
    }
}

fn register_optional_auto_screenshot(app: &mut App) {
    const SCREENSHOT_ENV: &str = "BUI_SCREENSHOT_PATH";

    if let Ok(path) = std::env::var(SCREENSHOT_ENV) {
        app.insert_resource(AutoScreenshotPath(PathBuf::from(path)));
        app.insert_resource(ActiveAutoScreenshotProfile(detect_auto_screenshot_profile()));
        app.insert_resource(AutoScreenshotFrames(detect_auto_screenshot_frames()));
    }
}

fn detect_auto_screenshot_frames() -> u8 {
    const FRAMES_ENV: &str = "BUI_SCREENSHOT_FRAMES";

    std::env::var(FRAMES_ENV)
        .ok()
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|frames| *frames > 0)
        .unwrap_or(30)
}

fn detect_auto_screenshot_profile() -> AutoScreenshotProfile {
    if let Some(profile) = auto_screenshot_profile_from_env() {
        return profile;
    }
    AutoScreenshotProfile::DEFAULT
}

fn auto_screenshot_profile_from_env() -> Option<AutoScreenshotProfile> {
    const WIDTH_ENV: &str = "BUI_SCREENSHOT_WIDTH";
    const HEIGHT_ENV: &str = "BUI_SCREENSHOT_HEIGHT";

    let width = std::env::var(WIDTH_ENV).ok()?.parse::<u32>().ok()?;
    let height = std::env::var(HEIGHT_ENV).ok()?.parse::<u32>().ok()?;
    if width == 0 || height == 0 {
        return None;
    }

    Some(AutoScreenshotProfile { width, height })
}

fn asset_root_for_file(file_path: &Path) -> PathBuf {
    file_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn log_bui_root_system(root: Option<Res<BuiRootEntity>>, mut logged: Local<bool>) {
    if *logged {
        return;
    }

    let Some(root) = root else {
        return;
    };

    info!("BUI root entity spawned: {:?}", root.0);
    *logged = true;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_editor_hotkey_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("Editor: F8 toggle | Hover border to reveal delete | Drag absolute nodes | F8 again to save/discard"),
        TextFont {
            font: FontSource::default(),
            font_size: FontSize::Px(16.0),
            ..TextFont::default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..Node::default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.82)),
        BorderColor::all(Color::srgba(0.92, 0.86, 0.65, 0.7)),
        GlobalZIndex(10001),
    ));
}

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
}

#[derive(Resource, Clone, Copy)]
struct ActiveAutoScreenshotProfile(AutoScreenshotProfile);

#[derive(Resource, Clone, Copy)]
struct AutoScreenshotFrames(u8);

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

fn button_feedback_system(
    buttons: Query<
        (
            &Interaction,
            Option<&BuiId>,
            Option<&BuiLogicTags>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, id, tags) in &buttons {
        if let Some(id) = id {
            info!(
                "Button interaction on BUI node '{}': {:?}",
                id.0, interaction
            );
        }
        if let Some(tags) = tags {
            info!("Button tags: {:?}", tags.0);
        }
    }
}

fn log_text_input_focus_system(
    input_focus: Res<InputFocus>,
    text_inputs: Query<(&BuiId, Option<&TextColor>), With<BuiTextInput>>,
) {
    if !input_focus.is_changed() {
        return;
    }

    let Some(focused) = input_focus.get() else {
        info!("Input focus cleared.");
        return;
    };

    if let Ok((id, text_color)) = text_inputs.get(focused) {
        info!(
            "Focused text input '{}', color: {:?}",
            id.0,
            text_color.map(|color| color.0)
        );
    }
}

fn log_text_input_value_system(
    text_inputs: Query<(&BuiId, &EditableText), (With<BuiTextInput>, Changed<EditableText>)>,
) {
    for (id, editable_text) in &text_inputs {
        info!("Text input '{}' value: '{}'", id.0, editable_text.value());
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

fn route_bui_root_to_auto_screenshot_target_system(
    mut commands: Commands,
    screenshot_target: Option<Res<AutoScreenshotTarget>>,
    root: Option<Res<BuiRootEntity>>,
    children_query: Query<&Children>,
    root_ui_nodes: Query<Entity, (With<Node>, Without<ChildOf>)>,
) {
    let (Some(screenshot_target), Some(root)) = (screenshot_target, root) else {
        return;
    };

    commands
        .entity(screenshot_target.container)
        .insert(UiTargetCamera(screenshot_target.camera));

    for entity in root_ui_nodes.iter() {
        if entity == screenshot_target.container {
            continue;
        }

        assign_ui_target_camera_recursive(
            &mut commands,
            &children_query,
            entity,
            screenshot_target.camera,
        );

        if entity != root.0 {
            commands
                .entity(screenshot_target.container)
                .add_child(entity);
            continue;
        }

        commands
            .entity(screenshot_target.container)
            .add_child(root.0);
    }

    assign_ui_target_camera_recursive(
        &mut commands,
        &children_query,
        screenshot_target.container,
        screenshot_target.camera,
    );
}

fn assign_ui_target_camera_recursive(
    commands: &mut Commands,
    children_query: &Query<&Children>,
    entity: Entity,
    camera: Entity,
) {
    commands.entity(entity).insert(UiTargetCamera(camera));

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        assign_ui_target_camera_recursive(commands, children_query, child, camera);
    }
}

fn auto_capture_screenshot_system(
    mut commands: Commands,
    screenshot_path: Option<Res<AutoScreenshotPath>>,
    screenshot_target: Option<Res<AutoScreenshotTarget>>,
    root: Option<Res<BuiRootEntity>>,
    screenshot_frames: Option<Res<AutoScreenshotFrames>>,
    computed_nodes: Query<&ComputedNode>,
    mut state: Local<AutoScreenshotState>,
) {
    let Some(screenshot_path) = screenshot_path else {
        return;
    };

    if state.requested {
        return;
    }

    let Some(root) = root else {
        return;
    };

    let Ok(computed_root) = computed_nodes.get(root.0) else {
        state.frames_after_layout = 0;
        return;
    };

    if computed_root.size().x <= 0.0 || computed_root.size().y <= 0.0 {
        state.frames_after_layout = 0;
        return;
    }

    state.frames_after_layout += 1;
    let target_frames = screenshot_frames
        .as_deref()
        .map(|frames| frames.0)
        .unwrap_or(30);
    if state.frames_after_layout < target_frames {
        return;
    }

    let screenshot_path = screenshot_path.0.clone();
    ensure_screenshot_parent_dir(&screenshot_path);
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

fn ensure_screenshot_parent_dir(path: &Path) {
    let Some(parent) = path.parent() else {
        return;
    };

    if let Err(error) = create_screenshot_parent_dir(parent) {
        warn!(
            "Failed to create screenshot output directory '{}': {error}",
            parent.display()
        );
    }
}

fn create_screenshot_parent_dir(parent: &Path) -> io::Result<()> {
    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    fs::create_dir_all(parent)
}
