use std::path::{Path, PathBuf};

use bevy::app::AppExit;
use bevy::input_focus::InputFocus;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::text::EditableText;
use bevy::{asset::io::AssetSourceBuilder, prelude::*};
use bevy_ai_ui_parser::{AiUiPlugin, BuiId, BuiLogicTags, BuiRootEntity, BuiTextInput};

#[allow(dead_code)]
pub fn run_with_json(file_name: &str) {
    let mut app = App::new();
    configure_app_with_json(&mut app, file_name, true);
    app.run();
}

#[allow(dead_code)]
pub fn run_with_json_without_button_feedback(file_name: &str) {
    let mut app = App::new();
    configure_app_with_json(&mut app, file_name, false);
    app.run();
}

pub fn configure_app_with_json(app: &mut App, file_name: &str, button_feedback_enabled: bool) {
    register_optional_windows_fonts_source(app);
    register_optional_auto_screenshot(app);

    app.add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path(bui_json_path(file_name)))
        .add_systems(Startup, setup_camera);

    if button_feedback_enabled {
        app.add_systems(Update, button_feedback_system);
    }

    app.add_systems(
        Update,
        (
            log_bui_root_system,
            log_text_input_focus_system,
            log_text_input_value_system,
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

fn register_optional_auto_screenshot(app: &mut App) {
    const SCREENSHOT_ENV: &str = "BUI_SCREENSHOT_PATH";

    if let Ok(path) = std::env::var(SCREENSHOT_ENV) {
        app.insert_resource(AutoScreenshotPath(PathBuf::from(path)));
    }
}

fn bui_json_path(file_name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("UiParserTest");

    let nested = std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| root.join(stem).join(file_name)));

    nested
        .filter(|path| path.exists())
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

#[derive(Resource)]
struct AutoScreenshotPath(PathBuf);

#[derive(Default)]
struct AutoScreenshotState {
    frames_after_root: u8,
    requested: bool,
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

fn auto_capture_screenshot_system(
    mut commands: Commands,
    screenshot_path: Option<Res<AutoScreenshotPath>>,
    root: Option<Res<BuiRootEntity>>,
    mut state: Local<AutoScreenshotState>,
) {
    let Some(screenshot_path) = screenshot_path else {
        return;
    };

    if state.requested || root.is_none() {
        return;
    }

    state.frames_after_root += 1;
    if state.frames_after_root < 3 {
        return;
    }

    let screenshot_path = screenshot_path.0.clone();
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut app_exit_writer: MessageWriter<AppExit>| {
            save_to_disk(screenshot_path.clone())(captured);
            app_exit_writer.write(AppExit::Success);
        },
    );
    state.requested = true;
}
