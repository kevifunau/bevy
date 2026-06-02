use bevy::prelude::*;
use bevy::ui::InteractionDisabled;

use crate::blackboard::BlackboardResource;
use crate::data_schema::*;
use crate::director::{OnComponentClickedMessage, UiRenderRequestMessage};
use crate::expression_eval::evaluate_condition;

pub struct UiRendererPlugin;

impl Plugin for UiRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_interaction_ui.run_if(in_state(crate::FmvAppState::InGameHud)),
        )
        .add_systems(
            Update,
            handle_interaction_button_click.run_if(in_state(crate::FmvAppState::InGameHud)),
        );
    }
}

#[derive(Component)]
pub struct InteractionUIRoot;

#[derive(Component)]
pub struct InteractionButtonEntity;

const CAPSULE_BG: Color = Color::srgb(0.12, 0.12, 0.18);
const CAPSULE_BG_DISABLED: Color = Color::srgb(0.08, 0.08, 0.08);
const CAPSULE_BORDER: Color = Color::srgb(0.4, 0.6, 0.9);
const GHOST_LINE_BG: Color = Color::srgb(0.05, 0.05, 0.08);

fn spawn_interaction_ui(
    mut commands: Commands,
    mut ev_render: MessageReader<UiRenderRequestMessage>,
    existing: Query<Entity, With<InteractionUIRoot>>,
    bb: Res<BlackboardResource>,
) {
    for event in ev_render.read() {
        for entity in &existing {
            commands.entity(entity).despawn();
        }

        let mut button_entities: Vec<Entity> = Vec::new();

        let root = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(10),
                    bottom: percent(10),
                    width: percent(35),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(20)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                InteractionUIRoot,
            ))
            .id();

        for comp in &event.components {
            let is_condition_blocked = comp
                .behavior
                .condition
                .as_ref()
                .map(|cond| !evaluate_condition(cond, &bb))
                .unwrap_or(false);

            let bg_color = match comp.visual_style.as_str() {
                "GHOST_LINE" => GHOST_LINE_BG,
                _ => if is_condition_blocked {
                    CAPSULE_BG_DISABLED
                } else {
                    CAPSULE_BG
                },
            };
            let border_color = if is_condition_blocked {
                Color::srgb(0.2, 0.2, 0.2)
            } else {
                CAPSULE_BORDER
            };

            let mut btn = commands.spawn((
                Button,
                Node {
                    width: percent(100),
                    height: px(50),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(25)),
                    margin: UiRect::vertical(px(8)),
                    ..default()
                },
                BackgroundColor(bg_color),
                BorderColor::all(border_color),
                InteractionButtonEntity,
                InteractionButtonComponent {
                    component_id: comp.component_id.clone(),
                    visual_style: comp.visual_style.clone(),
                    text: comp.content.text.clone(),
                    condition: comp.behavior.condition.clone(),
                    on_click_target: comp.behavior.on_click_target.clone(),
                    mutations: comp.behavior.mutations.clone(),
                },
            ));

            if is_condition_blocked {
                btn.insert(InteractionDisabled);
            }

            let btn_entity = btn.with_children(|b| {
                b.spawn((
                    Text::new(if is_condition_blocked {
                        format!("{} [条件未满足]", comp.content.text)
                    } else {
                        comp.content.text.clone()
                    }),
                    TextFont {
                        font_size: FontSize::Px(22.0),
                        ..default()
                    },
                    TextColor(if is_condition_blocked {
                        Color::srgb(0.4, 0.4, 0.4)
                    } else {
                        Color::srgb(0.85, 0.85, 0.95)
                    }),
                ));
            }).id();

            button_entities.push(btn_entity);
        }

        for btn_entity in button_entities {
            commands.entity(root).add_child(btn_entity);
        }
    }
}

fn handle_interaction_button_click(
    mut interaction_query: Query<
        (&Interaction, &InteractionButtonComponent),
        (Changed<Interaction>, With<Button>, With<InteractionButtonEntity>),
    >,
    mut ev_click: MessageWriter<OnComponentClickedMessage>,
    existing: Query<Entity, With<InteractionUIRoot>>,
    mut commands: Commands,
) {
    for (interaction, comp) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            ev_click.write(OnComponentClickedMessage {
                target_node: comp.on_click_target.clone(),
                mutations: comp.mutations.clone(),
            });
            for entity in &existing {
                commands.entity(entity).despawn();
            }
        }
    }
}