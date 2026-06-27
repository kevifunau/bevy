use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, ButtonInput};
use bevy_input_focus::{FocusCause, InputFocus};
use bevy_math::Vec2;
use bevy_text::EditableText;
use bevy_ui::{Checked, ComputedNode, Interaction, Node, Overflow, ScrollPosition};
use bevy_ui_widgets::{SliderRange, SliderStep, SliderValue};

use crate::core::{
    interaction::{
        action_registry::BuiActionAppExt,
        components::{
            BuiActions, BuiBindings, BuiDropdownGroupDefinition, BuiDropdownItem, BuiFocusOrder,
            BuiScrollView, BuiTextInput, BuiToggle,
        },
        dropdown::{dispatch_bui_dropdown_selection_system, focused_dropdown_confirm_system},
        keyboard::{
            focused_control_confirm_system, keyboard_focus_navigation_system, pointer_focus_system,
        },
        scroll::{
            apply_scroll_delta, dispatch_scroll_view_changed_system,
            focused_scroll_view_keyboard_input_system,
        },
        slider::{dispatch_slider_value_changed_system, focused_slider_keyboard_input_system},
        state::apply_bui_state_updates_system,
        text_input::dispatch_text_input_value_changed_system,
        toggle::toggle_interaction_system,
        types::{
            BuiActionTrigger, BuiActionTriggered, BuiBindingUpdate, BuiBindingValue, BuiStateSet,
            BuiStateStore,
        },
    },
    model::{BuiActionBinding, BuiBinding},
    runtime::components::BuiId,
};

#[derive(Resource, Default)]
struct HandledActions(Vec<String>);

#[test]
fn focused_button_enter_emits_press_action() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<HandledActions>()
        .add_systems(Update, focused_control_confirm_system)
        .add_bui_action_handler("menu.confirm", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    let button = app
        .world_mut()
        .spawn((
            BuiId("confirm_button".to_string()),
            BuiActions(vec![BuiActionBinding {
                event: "press".to_string(),
                emit: "menu.confirm".to_string(),
            }]),
        ))
        .id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(button, FocusCause::Navigated);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);

    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["Press:confirm_button".to_string()]
    );
}

#[test]
fn focused_text_input_enter_does_not_emit_press_action() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<HandledActions>()
        .add_systems(Update, focused_control_confirm_system)
        .add_bui_action_handler("input.press", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    let input = app
        .world_mut()
        .spawn((
            BuiId("account_input".to_string()),
            BuiTextInput,
            BuiActions(vec![BuiActionBinding {
                event: "press".to_string(),
                emit: "input.press".to_string(),
            }]),
        ))
        .id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(input, FocusCause::Navigated);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);

    app.update();
    app.update();

    assert!(
        app.world().resource::<HandledActions>().0.is_empty(),
        "text input Enter should be reserved for submit handling"
    );
}

#[test]
fn tab_navigation_cycles_focusable_bui_controls() {
    let mut app = App::new();
    app.init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, keyboard_focus_navigation_system);

    let first = app
        .world_mut()
        .spawn((
            BuiId("garage_button".to_string()),
            BuiActions(vec![BuiActionBinding {
                event: "press".to_string(),
                emit: "garage.open".to_string(),
            }]),
            BuiFocusOrder(0),
        ))
        .id();
    let second = app
        .world_mut()
        .spawn((
            BuiId("events_button".to_string()),
            BuiActions(vec![BuiActionBinding {
                event: "press".to_string(),
                emit: "events.open".to_string(),
            }]),
            BuiFocusOrder(1),
        ))
        .id();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(first));

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(KeyCode::Tab);
        input.clear();
    }
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(second));
}

#[test]
fn pointer_press_moves_focus_to_pressed_control() {
    let mut app = App::new();
    app.init_resource::<InputFocus>()
        .add_systems(Update, pointer_focus_system);

    let button = app
        .world_mut()
        .spawn((
            BuiId("play_button".to_string()),
            Interaction::None,
            BuiActions(vec![BuiActionBinding {
                event: "press".to_string(),
                emit: "race.start".to_string(),
            }]),
        ))
        .id();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();

    assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
}

#[test]
fn focused_slider_arrow_key_changes_value() {
    let mut app = App::new();
    app.init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, focused_slider_keyboard_input_system);

    let slider = app
        .world_mut()
        .spawn((
            BuiId("nitro_slider".to_string()),
            SliderValue(5.0),
            SliderRange::new(0.0, 10.0),
            SliderStep(2.0),
        ))
        .id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(slider, FocusCause::Navigated);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowRight);
    app.update();

    assert_eq!(
        app.world()
            .entity(slider)
            .get::<SliderValue>()
            .expect("slider should keep a value")
            .0,
        7.0
    );
}

#[test]
fn focused_scroll_view_arrow_key_changes_scroll_position() {
    let mut app = App::new();
    app.init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, focused_scroll_view_keyboard_input_system);

    let scroll = app
        .world_mut()
        .spawn((
            BuiId("garage_scroll".to_string()),
            BuiScrollView {
                binding_source: None,
            },
            ScrollPosition::default(),
            Node {
                overflow: Overflow::scroll_y(),
                ..Default::default()
            },
            ComputedNode {
                size: Vec2::new(100.0, 100.0),
                content_size: Vec2::new(100.0, 400.0),
                inverse_scale_factor: 1.0,
                ..Default::default()
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(scroll, FocusCause::Navigated);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowDown);
    app.update();

    assert!(
        app.world()
            .entity(scroll)
            .get::<ScrollPosition>()
            .expect("scroll view should keep position")
            .0
            .y
            > 0.0
    );
}

#[test]
fn bui_action_registry_routes_action_messages_to_registered_handlers() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .init_resource::<HandledActions>()
        .add_bui_action_handler("start-race", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{}:{}", event.id, event.action));
        });

    app.world_mut()
        .resource_mut::<Messages<BuiActionTriggered>>()
        .write(BuiActionTriggered {
            entity: Entity::PLACEHOLDER,
            id: "play_button".to_string(),
            action: "start-race".to_string(),
            trigger: BuiActionTrigger::Press,
        });
    app.world_mut()
        .resource_mut::<Messages<BuiActionTriggered>>()
        .write(BuiActionTriggered {
            entity: Entity::PLACEHOLDER,
            id: "settings_button".to_string(),
            action: "open-settings".to_string(),
            trigger: BuiActionTrigger::Press,
        });

    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["play_button:start-race".to_string()]
    );
}

#[test]
fn scroll_delta_clamps_to_scrollable_content_size() {
    let mut position = ScrollPosition(Vec2::new(10.0, 20.0));
    let node = Node {
        overflow: Overflow::scroll_y(),
        ..Default::default()
    };
    let computed = ComputedNode {
        size: Vec2::new(100.0, 100.0),
        content_size: Vec2::new(100.0, 260.0),
        inverse_scale_factor: 1.0,
        ..Default::default()
    };

    assert!(apply_scroll_delta(
        &mut position,
        &node,
        &computed,
        Vec2::new(50.0, 400.0)
    ));
    assert_eq!(position.0, Vec2::new(10.0, 160.0));

    assert!(apply_scroll_delta(
        &mut position,
        &node,
        &computed,
        Vec2::new(0.0, -500.0)
    ));
    assert_eq!(position.0, Vec2::new(10.0, 0.0));
}

#[test]
fn toggle_press_emits_action_and_updates_checked_state() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .add_message::<BuiStateSet>()
        .add_message::<BuiBindingUpdate>()
        .init_resource::<BuiStateStore>()
        .init_resource::<HandledActions>()
        .add_systems(
            Update,
            (
                toggle_interaction_system,
                apply_bui_state_updates_system.after(toggle_interaction_system),
            ),
        )
        .add_bui_action_handler("audio.changed", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    let toggle = app
        .world_mut()
        .spawn((
            BuiId("audio_toggle".to_string()),
            Interaction::None,
            BuiActions(vec![BuiActionBinding {
                event: "value_changed".to_string(),
                emit: "audio.changed".to_string(),
            }]),
            BuiBindings(vec![BuiBinding {
                target: "checked".to_string(),
                source: "settings.audio_enabled".to_string(),
            }]),
            BuiToggle,
            Checked,
        ))
        .id();

    app.world_mut()
        .entity_mut(toggle)
        .insert(Interaction::Pressed);
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["ValueChanged:audio_toggle".to_string()]
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("settings.audio_enabled"),
        Some(&BuiBindingValue::Bool(false))
    );
    assert!(
        !app.world().entity(toggle).contains::<Checked>(),
        "checked marker should be removed after pressing an initially checked toggle"
    );
}

#[test]
fn dropdown_selection_emits_action_and_updates_bound_state() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .add_message::<BuiStateSet>()
        .add_message::<BuiBindingUpdate>()
        .init_resource::<BuiStateStore>()
        .init_resource::<HandledActions>()
        .add_systems(
            Update,
            (
                dispatch_bui_dropdown_selection_system,
                apply_bui_state_updates_system.after(dispatch_bui_dropdown_selection_system),
            ),
        )
        .add_bui_action_handler("difficulty.changed", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    app.world_mut().spawn(BuiDropdownGroupDefinition {
        group: "difficulty".to_string(),
        source: "settings.difficulty".to_string(),
    });
    let hard = app
        .world_mut()
        .spawn((
            BuiId("difficulty_hard".to_string()),
            Interaction::None,
            BuiDropdownItem {
                group: "difficulty".to_string(),
                value: "hard".to_string(),
            },
            BuiActions(vec![BuiActionBinding {
                event: "selection_changed".to_string(),
                emit: "difficulty.changed".to_string(),
            }]),
        ))
        .id();

    app.world_mut()
        .entity_mut(hard)
        .insert(Interaction::Pressed);
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["SelectionChanged:difficulty_hard".to_string()]
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("settings.difficulty"),
        Some(&BuiBindingValue::Text("hard".to_string()))
    );
}

#[test]
fn focused_dropdown_enter_selects_option() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .add_message::<BuiStateSet>()
        .add_message::<BuiBindingUpdate>()
        .init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<BuiStateStore>()
        .init_resource::<HandledActions>()
        .add_systems(
            Update,
            (
                focused_dropdown_confirm_system,
                apply_bui_state_updates_system.after(focused_dropdown_confirm_system),
            ),
        )
        .add_bui_action_handler("garage.filter.changed", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    app.world_mut().spawn(BuiDropdownGroupDefinition {
        group: "garage_filter".to_string(),
        source: "garage.filter".to_string(),
    });
    let option = app
        .world_mut()
        .spawn((
            BuiId("filter_elite".to_string()),
            BuiDropdownItem {
                group: "garage_filter".to_string(),
                value: "elite".to_string(),
            },
            BuiActions(vec![BuiActionBinding {
                event: "selection_changed".to_string(),
                emit: "garage.filter.changed".to_string(),
            }]),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(option, FocusCause::Navigated);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["SelectionChanged:filter_elite".to_string()]
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("garage.filter"),
        Some(&BuiBindingValue::Text("elite".to_string()))
    );
}

#[test]
fn scroll_position_changed_emits_action_and_updates_bound_state() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .add_message::<BuiStateSet>()
        .add_message::<BuiBindingUpdate>()
        .init_resource::<BuiStateStore>()
        .init_resource::<HandledActions>()
        .add_systems(
            Update,
            (
                dispatch_scroll_view_changed_system,
                apply_bui_state_updates_system.after(dispatch_scroll_view_changed_system),
            ),
        )
        .add_bui_action_handler("inventory.scrolled", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    let scroll = app
        .world_mut()
        .spawn((
            BuiId("inventory_scroll".to_string()),
            ScrollPosition::default(),
            BuiScrollView {
                binding_source: Some("inventory.list".to_string()),
            },
            BuiActions(vec![BuiActionBinding {
                event: "scroll".to_string(),
                emit: "inventory.scrolled".to_string(),
            }]),
        ))
        .id();

    app.world_mut()
        .entity_mut(scroll)
        .insert(ScrollPosition(Vec2::new(12.0, 48.0)));
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["Scroll:inventory_scroll".to_string()]
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("inventory.list.scroll_x"),
        Some(&BuiBindingValue::Number(12.0))
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("inventory.list.scroll_y"),
        Some(&BuiBindingValue::Number(48.0))
    );
}

#[test]
fn slider_value_changed_emits_action_and_updates_bound_state() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .add_message::<BuiStateSet>()
        .add_message::<BuiBindingUpdate>()
        .init_resource::<BuiStateStore>()
        .init_resource::<HandledActions>()
        .add_systems(
            Update,
            (
                dispatch_slider_value_changed_system,
                apply_bui_state_updates_system.after(dispatch_slider_value_changed_system),
            ),
        )
        .add_bui_action_handler("volume.changed", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    let slider = app
        .world_mut()
        .spawn((
            BuiId("volume_slider".to_string()),
            SliderValue(0.25),
            BuiActions(vec![BuiActionBinding {
                event: "value_changed".to_string(),
                emit: "volume.changed".to_string(),
            }]),
            BuiBindings(vec![BuiBinding {
                target: "value".to_string(),
                source: "settings.volume".to_string(),
            }]),
        ))
        .id();

    app.world_mut().entity_mut(slider).insert(SliderValue(0.75));
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["ValueChanged:volume_slider".to_string()]
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("settings.volume"),
        Some(&BuiBindingValue::Number(0.75))
    );
}

#[test]
fn text_input_value_changed_emits_action_and_updates_bound_state() {
    let mut app = App::new();
    app.add_message::<BuiActionTriggered>()
        .add_message::<BuiStateSet>()
        .add_message::<BuiBindingUpdate>()
        .init_resource::<BuiStateStore>()
        .init_resource::<HandledActions>()
        .add_systems(
            Update,
            (
                dispatch_text_input_value_changed_system,
                apply_bui_state_updates_system.after(dispatch_text_input_value_changed_system),
            ),
        )
        .add_bui_action_handler("account.changed", |world, event| {
            world
                .resource_mut::<HandledActions>()
                .0
                .push(format!("{:?}:{}", event.trigger, event.id));
        });

    let input = app
        .world_mut()
        .spawn((
            BuiId("account_input".to_string()),
            BuiTextInput,
            EditableText::new("Racer"),
            BuiActions(vec![BuiActionBinding {
                event: "value_changed".to_string(),
                emit: "account.changed".to_string(),
            }]),
            BuiBindings(vec![BuiBinding {
                target: "text.content".to_string(),
                source: "login.account".to_string(),
            }]),
        ))
        .id();

    app.world_mut()
        .entity_mut(input)
        .get_mut::<EditableText>()
        .expect("input should have editable text")
        .editor_mut()
        .set_text("Ace");
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HandledActions>().0,
        vec!["ValueChanged:account_input".to_string()]
    );
    assert_eq!(
        app.world()
            .resource::<BuiStateStore>()
            .0
            .get("login.account"),
        Some(&BuiBindingValue::Text("Ace".to_string()))
    );
}
