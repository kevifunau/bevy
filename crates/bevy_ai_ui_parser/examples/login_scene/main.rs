#![allow(missing_docs)]

//! Login scene game code — business logic only.
//!
//! The plugin handles all UI loading/unloading (IR JSON = prefab).
//! This file only contains:
//! - Panel registration (name → IR path)
//! - Action handlers (button clicks → business logic)
//! - Data binding (pushing data to UI)
//!
//! This is the Bevy equivalent of Unity's LoginPanel.cs / RegisterPanel.cs / ServerPanel.cs.

use bevy::asset::io::AssetSourceBuilder;
use bevy::asset::{AssetPlugin, UnapprovedPathMode};
use bevy::prelude::*;
use bevy_ai_ui_parser::{
    BuiActionAppExt, BuiActionTriggered, BuiPanelAppExt, BuiPanelSwitch, BuiStateSet,
    BuiStateStore, BuiBindingValue,
};
use bevy_camera::visibility::Visibility;

const WEBGAMEUI_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/login_scene/webgameui"
);
const PREFABS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/login_scene/prefabs"
);

/// Persisted login data (equivalent to Unity's LoginData / LoginMgr).
#[derive(Resource)]
struct LoginData {
    username: String,
    password: String,
    remember_pw: bool,
    auto_login: bool,
    /// Server ID selected by the player.
    #[allow(dead_code)]
    front_server_id: i32,
    front_server_name: String,
}

impl Default for LoginData {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            remember_pw: false,
            auto_login: false,
            front_server_id: -1,
            front_server_name: String::new(),
        }
    }
}

fn main() {
    let mut app = App::new();

    // macOS font sources
    let macos_fonts = std::path::Path::new("/System/Library/Fonts");
    if macos_fonts.exists() {
        app.register_asset_source(
            "macos_fonts",
            AssetSourceBuilder::platform_default("/System/Library/Fonts", None),
        );
    }
    let macos_supplemental = std::path::Path::new("/System/Library/Fonts/Supplemental");
    if macos_supplemental.exists() {
        app.register_asset_source(
            "macos_supplemental_fonts",
            AssetSourceBuilder::platform_default("/System/Library/Fonts/Supplemental", None),
        );
    }

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: WEBGAMEUI_DIR.to_string(),
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Login Scene".to_string(),
                    resolution: bevy::window::WindowResolution::new(1365, 768),
                    ..default()
                }),
                ..default()
            }),
    )
    // Plugin loads the initial panel (index = login) + registers all interaction systems
    .add_plugins(bevy_ai_ui_parser::AiUiPlugin::from_path(format!("{PREFABS_DIR}/index.ir.json")))
    // Register all panels (plugin handles loading/unloading)
    .register_bui_panel("login", format!("{PREFABS_DIR}/index.ir.json"))
    .register_bui_panel("register", format!("{PREFABS_DIR}/register.ir.json"))
    .register_bui_panel("server_select", format!("{PREFABS_DIR}/server_select.ir.json"))
    .register_bui_panel("server_list", format!("{PREFABS_DIR}/server_list.ir.json"))
    .insert_resource(LoginData::default())
    .insert_resource(ClearColor(Color::srgb_u8(26, 26, 46)))
    .add_systems(Startup, setup_camera)
    // Action handlers — only business logic (the JS that was discarded from HTML)
    .add_bui_action_handler("login.sure", handle_login_sure)
    .add_bui_action_handler("login.register", handle_login_register)
    .add_bui_action_handler("login.toggleRemember", handle_toggle_remember)
    .add_bui_action_handler("login.toggleAuto", handle_toggle_auto)
    .add_bui_action_handler("register.sure", handle_register_sure)
    .add_bui_action_handler("register.cancel", handle_register_cancel)
    .add_bui_action_handler("server.change", handle_server_change)
    .add_bui_action_handler("server.start", handle_server_start)
    .add_bui_action_handler("server.back", handle_server_back)
    .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ===== Action Handlers (business logic only — equivalent to Unity's Panel.cs) =====

/// Handle "login.sure" — validate, show tip on error, switch to server_select on success.
fn handle_login_sure(world: &mut World, _event: &BuiActionTriggered) {
    let (username, password, remember, auto) = {
        let store = &world.resource::<BuiStateStore>().0;
        let get_text = |k: &str| store.get(k).and_then(|v| match v { BuiBindingValue::Text(t) => Some(t.clone()), _ => None }).unwrap_or_default();
        let get_bool = |k: &str| store.get(k).and_then(|v| match v { BuiBindingValue::Bool(b) => Some(*b), _ => None }).unwrap_or(false);
        (get_text("login.username"), get_text("login.password"), get_bool("login.rememberPw"), get_bool("login.autoLogin"))
    };

    if username.len() <= 6 || password.len() <= 6 {
        info!("Login failed: too short");
        let mut w = world.resource_mut::<Messages<BuiStateSet>>();
        w.write(BuiStateSet { key: "tip.info".into(), value: BuiBindingValue::Text("账号和密码都必须大于6位".into()) });
        show_node(world, "tip_panel");
        return;
    }

    info!("Login success: user={}", username);
    let mut data = world.resource_mut::<LoginData>();
    data.username = username;
    data.password = password;
    data.remember_pw = remember;
    data.auto_login = auto;

    // Switch to server_select panel (plugin handles load/unload)
    world.resource_mut::<BuiPanelSwitch>().show("server_select");
}

/// Handle "login.register" — switch to register panel.
fn handle_login_register(world: &mut World, _event: &BuiActionTriggered) {
    world.resource_mut::<BuiPanelSwitch>().show("register");
}

/// Handle "login.toggleRemember" — when remember unchecked, uncheck auto login.
fn handle_toggle_remember(world: &mut World, _event: &BuiActionTriggered) {
    let remember = world.resource::<BuiStateStore>().0.get("login.rememberPw")
        .and_then(|v| match v { BuiBindingValue::Bool(b) => Some(*b), _ => None }).unwrap_or(false);
    if !remember {
        let mut w = world.resource_mut::<Messages<BuiStateSet>>();
        w.write(BuiStateSet { key: "login.autoLogin".into(), value: BuiBindingValue::Bool(false) });
    }
}

/// Handle "login.toggleAuto" — when auto checked, check remember.
fn handle_toggle_auto(world: &mut World, _event: &BuiActionTriggered) {
    let auto = world.resource::<BuiStateStore>().0.get("login.autoLogin")
        .and_then(|v| match v { BuiBindingValue::Bool(b) => Some(*b), _ => None }).unwrap_or(false);
    if auto {
        let mut w = world.resource_mut::<Messages<BuiStateSet>>();
        w.write(BuiStateSet { key: "login.rememberPw".into(), value: BuiBindingValue::Bool(true) });
    }
}

/// Handle "register.sure" — validate, return to login on success.
fn handle_register_sure(world: &mut World, _event: &BuiActionTriggered) {
    let (username, password) = {
        let store = &world.resource::<BuiStateStore>().0;
        let get_text = |k: &str| store.get(k).and_then(|v| match v { BuiBindingValue::Text(t) => Some(t.clone()), _ => None }).unwrap_or_default();
        (get_text("register.username"), get_text("register.password"))
    };

    if username.len() <= 6 || password.len() <= 6 {
        info!("Register failed: too short");
        let mut w = world.resource_mut::<Messages<BuiStateSet>>();
        w.write(BuiStateSet { key: "tip.info".into(), value: BuiBindingValue::Text("账号和密码都必须大于6位".into()) });
        show_node(world, "tip_panel");
        return;
    }

    info!("Register success: {}", username);
    world.resource_mut::<LoginData>().username = username;
    world.resource_mut::<BuiPanelSwitch>().show("login");
}

/// Handle "register.cancel" — return to login.
fn handle_register_cancel(world: &mut World, _event: &BuiActionTriggered) {
    world.resource_mut::<BuiPanelSwitch>().show("login");
}

/// Handle "server.change" — switch to server_select.
fn handle_server_change(world: &mut World, _event: &BuiActionTriggered) {
    world.resource_mut::<BuiPanelSwitch>().show("server_select");
}

/// Handle "server.start" — enter game (exit for demo).
fn handle_server_start(world: &mut World, _event: &BuiActionTriggered) {
    let data = world.resource::<LoginData>();
    info!("Entering game! Server: {}", data.front_server_name);
    let mut w = world.resource_mut::<Messages<AppExit>>();
    w.write(AppExit::Success);
}

/// Handle "server.back" — return to login.
fn handle_server_back(world: &mut World, _event: &BuiActionTriggered) {
    world.resource_mut::<LoginData>().auto_login = false;
    world.resource_mut::<BuiPanelSwitch>().show("login");
}

// ===== Helper =====

/// Show a node by its Bui id.
fn show_node(world: &mut World, node_id: &str) {
    use bevy_ai_ui_parser::BuiId;
    let mut query = world.query::<(&BuiId, &mut Visibility)>();
    for (id, mut vis) in query.iter_mut(world) {
        if id.0 == node_id {
            *vis = Visibility::Inherited;
        }
    }
}
