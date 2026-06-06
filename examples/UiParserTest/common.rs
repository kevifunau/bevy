use std::path::{Path, PathBuf};
use std::{fs, io};

use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::image::Image;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::HoverMap;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::text::EditableText;
use bevy::{asset::io::AssetSourceBuilder, prelude::*};
use bevy_ai_ui_parser::{AiUiPlugin, BuiId, BuiLogicTags, BuiRootEntity, BuiTextInput};

#[allow(dead_code)]
pub fn run_with_json(file_name: &str) {
    let mut app = App::new();
    configure_app_with_plugin(&mut app, AiUiPlugin::from_path(bui_path(file_name)), true);
    app.run();
}

#[allow(dead_code)]
pub fn run_with_json_without_button_feedback(file_name: &str) {
    let mut app = App::new();
    configure_app_with_plugin(&mut app, AiUiPlugin::from_path(bui_path(file_name)), false);
    app.run();
}

#[allow(dead_code)]
pub fn run_with_bui_file_without_button_feedback(file_name: &str) {
    let mut app = App::new();
    configure_app_with_plugin(&mut app, AiUiPlugin::from_path(bui_path(file_name)), false);
    app.run();
}

#[allow(dead_code)]
pub fn run_with_html(file_name: &str) {
    let mut app = App::new();
    configure_app_with_plugin(
        &mut app,
        AiUiPlugin::from_html_path(bui_path(file_name)),
        true,
    );
    app.run();
}

#[allow(dead_code)]
pub fn run_with_html_without_button_feedback(file_name: &str) {
    let mut app = App::new();
    configure_app_with_plugin(
        &mut app,
        AiUiPlugin::from_html_path(bui_path(file_name)),
        false,
    );
    app.run();
}

#[allow(dead_code)]
pub fn configure_app_with_json(app: &mut App, file_name: &str, button_feedback_enabled: bool) {
    configure_app_with_plugin(
        app,
        AiUiPlugin::from_path(bui_path(file_name)),
        button_feedback_enabled,
    );
}

fn configure_app_with_plugin(app: &mut App, plugin: AiUiPlugin, button_feedback_enabled: bool) {
    register_optional_windows_fonts_source(app);
    register_optional_macos_fonts_source(app);
    register_optional_auto_screenshot(app);

    app.add_plugins(DefaultPlugins)
        .add_plugins(plugin)
        .insert_resource(ClearColor(Color::srgb_u8(59, 40, 24)))
        .add_systems(Startup, (setup_camera, setup_auto_screenshot_target));

    if button_feedback_enabled {
        app.add_systems(Update, button_feedback_system);
    }

    app.add_systems(
        Update,
        (
            send_scroll_events_system,
            log_bui_root_system,
            route_bui_root_to_auto_screenshot_target_system,
            log_text_input_focus_system,
            log_text_input_value_system,
            auto_capture_screenshot_system,
        ),
    );

    app.add_observer(on_scroll_handler);
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

fn bui_path(file_name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest");

    let nested = std::env::current_exe().ok().and_then(|path| {
        path.file_stem().map(|stem| {
            let stem = stem.to_string_lossy();
            root.join(&*stem).join(file_name)
        })
    });

    let nested_in_testset = std::env::current_exe().ok().and_then(|path| {
        path.file_stem().map(|stem| {
            let stem = stem.to_string_lossy();
            root.join("uiParse_TestSet").join(&*stem).join(file_name)
        })
    });

    nested
        .filter(|path| path.exists())
        .or_else(|| nested_in_testset.filter(|path| path.exists()))
        .unwrap_or_else(|| root.join(file_name))
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

const SCROLL_LINE_HEIGHT: f32 = 21.0;

#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct BuiScroll {
    entity: Entity,
    delta: Vec2,
}

fn send_scroll_events_system(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= SCROLL_LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(BuiScroll { entity, delta });
            }
        }
    }
}

fn on_scroll_handler(
    mut scroll: On<BuiScroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let at_limit = if delta.x > 0.0 {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.0
        };

        if !at_limit {
            scroll_position.x = (scroll_position.x + delta.x).clamp(0.0, max_offset.x);
            delta.x = 0.0;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let at_limit = if delta.y > 0.0 {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.0
        };

        if !at_limit {
            scroll_position.y = (scroll_position.y + delta.y).clamp(0.0, max_offset.y);
            delta.y = 0.0;
        }
    }

    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
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

    // Match the browser-reference composition more closely:
    // 1680x786 game-stage centered inside a 1728x888 logical viewport.
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

fn button_feedback_system(
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&BuiId>,
            Option<&BuiLogicTags>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background, id, tags) in &mut buttons {
        if let Some(id) = id {
            info!(
                "Button interaction on BUI node '{}': {:?}",
                id.0, interaction
            );
        }
        if let Some(tags) = tags {
            info!("Button tags: {:?}", tags.0);
        }

        background.0 = match *interaction {
            Interaction::Pressed => Color::srgb(0.17, 0.46, 0.2),
            Interaction::Hovered => Color::srgb(0.36, 0.77, 0.38),
            Interaction::None => Color::srgb(0.30, 0.69, 0.31),
        };
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
    mut state: Local<AutoScreenshotTargetRoutingState>,
) {
    if state.routed {
        return;
    }

    let (Some(screenshot_target), Some(root)) = (screenshot_target, root) else {
        return;
    };

    commands
        .entity(screenshot_target.container)
        .insert(UiTargetCamera(screenshot_target.camera));
    commands.entity(screenshot_target.container).add_child(root.0);
    state.routed = true;
}

fn auto_capture_screenshot_system(
    mut commands: Commands,
    screenshot_path: Option<Res<AutoScreenshotPath>>,
    screenshot_target: Option<Res<AutoScreenshotTarget>>,
    root: Option<Res<BuiRootEntity>>,
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
    if state.frames_after_layout < 30 {
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
