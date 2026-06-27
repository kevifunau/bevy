use super::shared::{find_bui_node, find_bui_node_optional, BEVY_UI_CAR_HTML};
use crate::core::model::BuiNode;
use crate::core::opendesign::html::opendesign_html_to_bui_document;

fn texture_paths(node: &BuiNode, paths: &mut Vec<String>) {
    if let Some(image) = &node.content.image {
        paths.push(image.texture_path.clone());
    }
    for state in node.state_visuals.values() {
        if let Some(image) = &state.image {
            paths.push(image.texture_path.clone());
        }
    }
    for child in &node.children {
        texture_paths(child, paths);
    }
}

fn assert_image(node: &BuiNode, node_id: &str, expected_path: &str) {
    let node = find_bui_node(node, node_id);

    assert_eq!(
        node.content
            .image
            .as_ref()
            .map(|image| image.texture_path.as_str()),
        Some(expected_path),
        "{node_id} should keep its OD-authored asset reference"
    );
}

#[test]
fn bevy_ui_car_compiles_prompt_regions_and_asset_backed_controls() {
    let document = opendesign_html_to_bui_document(BEVY_UI_CAR_HTML).expect("HTML should compile");

    for id in [
        "top_bar",
        "left_main_menu",
        "race_scene",
        "car_showcase",
        "bottom_bar",
        "social_shortcuts",
        "car_stats",
        "main_play",
    ] {
        find_bui_node(&document.root, id);
    }

    let root = find_bui_node(&document.root, "racing_main_ui");
    assert_eq!(
        root.content
            .image
            .as_ref()
            .map(|image| image.texture_path.as_str()),
        Some("Asset/background-clean.png")
    );

    let play = find_bui_node(&document.root, "main_play");
    assert_eq!(play.kind, "button");
    assert_eq!(
        play.actions
            .iter()
            .find(|action| action.event == "press")
            .map(|action| action.emit.as_str()),
        Some("start-race")
    );
    assert_eq!(
        play.content
            .image
            .as_ref()
            .map(|image| image.texture_path.as_str()),
        Some("Asset/play-button.png")
    );

    for (id, action) in [
        ("nav_garage", "open-garage"),
        ("nav_events", "open-events"),
        ("nav_multiplayer", "open-multiplayer"),
        ("nav_club", "open-club"),
        ("nav_profile", "open-profile"),
        ("nav_settings", "open-settings"),
    ] {
        let node = find_bui_node(&document.root, id);
        assert_eq!(node.kind, "button");
        assert_eq!(
            node.actions
                .iter()
                .find(|binding| binding.event == "press")
                .map(|binding| binding.emit.as_str()),
            Some(action),
            "{id} should dispatch its data-action"
        );
    }

    let status_text = find_bui_node(&document.root, "race_status_text_text_1");
    assert_eq!(
        status_text
            .content
            .text
            .as_ref()
            .map(|text| text.content.as_str()),
        Some("READY TO RACE")
    );
    assert_eq!(
        status_text
            .bindings
            .iter()
            .find(|binding| binding.source == "race.status")
            .map(|binding| binding.target.as_str()),
        Some("text.content")
    );
}

#[test]
fn bevy_ui_car_keeps_icon_and_hero_asset_references() {
    let document = opendesign_html_to_bui_document(BEVY_UI_CAR_HTML).expect("HTML should compile");
    assert_image(&document.root, "player_avatar", "Asset/avatar-frame.png");
    assert_image(&document.root, "play_light_ring", "Asset/play-ring.png");
    assert_image(&document.root, "car_preview", "Asset/car-hero.png");

    let mut paths = Vec::new();
    texture_paths(&document.root, &mut paths);

    for path in [
        "Asset/icon-garage.png",
        "Asset/icon-events.png",
        "Asset/icon-multiplayer.png",
        "Asset/icon-club.png",
        "Asset/icon-profile.png",
        "Asset/icon-settings.png",
        "Asset/icon-notification.png",
        "Asset/icon-chat.png",
        "Asset/icon-friends.png",
        "Asset/icon-mail.png",
        "Asset/icon-speed.png",
        "Asset/icon-accel.png",
        "Asset/icon-handling.png",
        "Asset/icon-nitro.png",
    ] {
        assert!(
            paths.iter().any(|candidate| candidate == path),
            "{path} should be preserved in the compiled IR"
        );
    }

    assert!(
        find_bui_node_optional(&document.root, "nav_icon").is_some(),
        "OD-authored img nodes without ids should still be retained with stable generated ids"
    );
}

#[test]
fn bevy_ui_car_builds_static_action_contract_from_od_output() {
    let document = opendesign_html_to_bui_document(BEVY_UI_CAR_HTML).expect("HTML should compile");

    for action in [
        "open-garage",
        "open-events",
        "open-multiplayer",
        "open-club",
        "open-profile",
        "open-settings",
        "open-notification",
        "open-chat",
        "open-friends",
        "open-mail",
        "start-race",
    ] {
        assert!(
            document.interaction_model.actions.contains_key(action),
            "{action} should be exposed through data-bui-actions"
        );
    }

    let open_events = document
        .interaction_model
        .actions
        .get("open-events")
        .expect("open-events should exist");
    assert_eq!(open_events[0].op, "run-action");
    assert_eq!(open_events[0].target.as_deref(), Some("select-events"));
    assert_eq!(open_events[1].op, "set-text");
    assert_eq!(
        open_events[1].node.as_deref(),
        Some("race_status_text_text_1")
    );
    assert_eq!(open_events[1].value.as_deref(), Some("EVENTS OPENED"));

    let start_race = document
        .interaction_model
        .actions
        .get("start-race")
        .expect("start-race should exist");
    assert_eq!(start_race.len(), 5);
    assert_eq!(start_race[0].value.as_deref(), Some("MATCHMAKING..."));
    assert_eq!(start_race[1].op, "delay");
    assert_eq!(start_race[1].ms, Some(900));
    assert_eq!(start_race[2].value.as_deref(), Some("RACE FOUND"));
    assert_eq!(start_race[4].value.as_deref(), Some("READY TO RACE"));
}
