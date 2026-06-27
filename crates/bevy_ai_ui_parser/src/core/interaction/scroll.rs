use bevy_ecs::prelude::*;
use bevy_input::{
    gamepad::{Gamepad, GamepadButton},
    keyboard::KeyCode,
    mouse::{MouseScrollUnit, MouseWheel},
    ButtonInput,
};
use bevy_input_focus::InputFocus;
use bevy_math::Vec2;
use bevy_picking::hover::HoverMap;
use bevy_ui::{ComputedNode, Node, OverflowAxis, ScrollPosition};

use crate::core::{
    interaction::{
        components::{BuiActions, BuiScrollView},
        types::{BuiActionTrigger, BuiActionTriggered, BuiBindingValue, BuiStateSet},
    },
    runtime::components::BuiId,
};

const SCROLL_LINE_HEIGHT: f32 = 21.0;
const KEYBOARD_SCROLL_LINES: f32 = 3.0;

pub(crate) fn scroll_view_wheel_input_system(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Option<Res<HoverMap>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_views: Query<(&mut ScrollPosition, &Node, &ComputedNode), With<BuiScrollView>>,
) {
    let Some(hover_map) = hover_map else {
        return;
    };

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
                let Ok((mut scroll_position, node, computed)) = scroll_views.get_mut(entity) else {
                    continue;
                };
                if apply_scroll_delta(&mut scroll_position, node, computed, delta) {
                    break;
                }
            }
        }
    }
}

pub(crate) fn dispatch_scroll_view_changed_system(
    scroll_views: Query<
        (
            Entity,
            &BuiId,
            &ScrollPosition,
            &BuiScrollView,
            Option<&BuiActions>,
        ),
        Changed<ScrollPosition>,
    >,
    mut action_writer: MessageWriter<BuiActionTriggered>,
    mut state_writer: MessageWriter<BuiStateSet>,
) {
    for (entity, id, position, scroll_view, actions) in &scroll_views {
        write_scroll_state(&mut state_writer, &id.0, position.0.x, position.0.y);
        if let Some(source) = &scroll_view.binding_source {
            write_scroll_state(&mut state_writer, source, position.0.x, position.0.y);
        }

        let Some(actions) = actions else {
            continue;
        };
        for action in &actions.0 {
            if !matches!(action.event.as_str(), "scroll" | "scrolled") {
                continue;
            }
            action_writer.write(BuiActionTriggered {
                entity,
                id: id.0.clone(),
                action: action.emit.clone(),
                trigger: BuiActionTrigger::Scroll,
            });
        }
    }
}

pub(crate) fn focused_scroll_view_keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    input_focus: Res<InputFocus>,
    mut scroll_views: Query<(&mut ScrollPosition, &Node, &ComputedNode), With<BuiScrollView>>,
) {
    let mut delta = Vec2::ZERO;
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        delta.y += SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        delta.y -= SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        delta.x += SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        delta.x -= SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
    }
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            delta.y += SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
        }
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            delta.y -= SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
        }
        if gamepad.just_pressed(GamepadButton::DPadRight) {
            delta.x += SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
        }
        if gamepad.just_pressed(GamepadButton::DPadLeft) {
            delta.x -= SCROLL_LINE_HEIGHT * KEYBOARD_SCROLL_LINES;
        }
    }
    if delta == Vec2::ZERO {
        return;
    }

    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok((mut scroll_position, node, computed)) = scroll_views.get_mut(focused) else {
        return;
    };
    apply_scroll_delta(&mut scroll_position, node, computed, delta);
}

pub(crate) fn apply_scroll_delta(
    scroll_position: &mut ScrollPosition,
    node: &Node,
    computed: &ComputedNode,
    delta: Vec2,
) -> bool {
    let max_offset = ((computed.content_size() - computed.size()) * computed.inverse_scale_factor)
        .max(Vec2::ZERO);
    let mut consumed = false;

    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let next = (scroll_position.0.x + delta.x).clamp(0.0, max_offset.x);
        if next != scroll_position.0.x {
            scroll_position.0.x = next;
            consumed = true;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let next = (scroll_position.0.y + delta.y).clamp(0.0, max_offset.y);
        if next != scroll_position.0.y {
            scroll_position.0.y = next;
            consumed = true;
        }
    }

    consumed
}

fn write_scroll_state(state_writer: &mut MessageWriter<BuiStateSet>, prefix: &str, x: f32, y: f32) {
    state_writer.write(BuiStateSet {
        key: format!("{prefix}.scroll_x"),
        value: BuiBindingValue::Number(x),
    });
    state_writer.write(BuiStateSet {
        key: format!("{prefix}.scroll_y"),
        value: BuiBindingValue::Number(y),
    });
}
