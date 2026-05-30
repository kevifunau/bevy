//! Renders the DIY Ultraman character BUI JSON test case.
//!
//! Run with:
//! `cargo run --example ultraman_ui`

#[path = "../common.rs"]
mod common;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::Checked;
use bevy_ai_ui_parser::{BuiActionTriggered, BuiBindingValue, BuiId, BuiStateSet, BuiStateStore, BuiToggle, BuiVisualState};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum UltramanTab {
    Attribute,
    Skill,
    Unique,
    Core,
    Amplifier,
}

#[derive(Resource, Debug, Default)]
struct FollowState {
    followed: bool,
}

#[derive(Resource, Debug)]
struct UltramanUiState {
    selected_tab: UltramanTab,
}

impl Default for UltramanUiState {
    fn default() -> Self {
        Self {
            selected_tab: UltramanTab::Amplifier,
        }
    }
}

fn main() {
    let mut app = App::new();
    common::configure_app_with_json(&mut app, "diy/ultraman_ui.json", false);
    app.init_resource::<UltramanUiState>()
        .init_resource::<FollowState>()
        .add_systems(Startup, seed_initial_bui_state_system)
        .add_systems(
            Update,
            (
                ultraman_button_press_system,
                sync_ultraman_tab_from_state_store_system,
                sync_follow_button_system,
                sync_share_note_toggle_system,
                emit_tab_binding_updates_system,
            ),
        )
        .run();
}

fn seed_initial_bui_state_system(mut state_writer: MessageWriter<BuiStateSet>) {
    state_writer.write(BuiStateSet {
        key: "ui.selected_tab".to_string(),
        value: BuiBindingValue::Text("amplifier".to_string()),
    });
}

fn ultraman_button_press_system(
    mut actions: MessageReader<BuiActionTriggered>,
    mut follow_state: ResMut<FollowState>,
) {
    for action in actions.read() {
        if action.action == "ui.hero.follow_toggle" {
            follow_state.followed = !follow_state.followed;
            info!("Ultraman UI follow toggled: {}", follow_state.followed);
            continue;
        }

        info!("Ultraman UI action on '{}': {}", action.id, action.action);
    }
}

fn sync_ultraman_tab_from_state_store_system(
    state_store: Res<BuiStateStore>,
    mut ui_state: ResMut<UltramanUiState>,
) {
    if !state_store.is_changed() {
        return;
    }

    let Some(BuiBindingValue::Text(value)) = state_store.0.get("ui.selected_tab") else {
        return;
    };

    let next = match value.as_str() {
        "attribute" => UltramanTab::Attribute,
        "skill" => UltramanTab::Skill,
        "unique" => UltramanTab::Unique,
        "core" => UltramanTab::Core,
        "amplifier" => UltramanTab::Amplifier,
        _ => return,
    };

    if ui_state.selected_tab != next {
        ui_state.selected_tab = next;
        info!("Ultraman UI selected tab: {:?}", next);
    }
}

fn sync_follow_button_system(
    follow_state: Res<FollowState>,
    ids: Query<(Entity, &BuiId)>,
    mut commands: Commands,
    mut state_writer: MessageWriter<BuiStateSet>,
    mut initialized: Local<bool>,
) {
    if !follow_state.is_changed() && *initialized {
        return;
    }
    *initialized = true;

    state_writer.write(BuiStateSet {
        key: "hero.follow_label".to_string(),
        value: BuiBindingValue::Text(if follow_state.followed {
            "♥已关注".to_string()
        } else {
            "♥关注".to_string()
        }),
    });
    state_writer.write(BuiStateSet {
        key: "hero.follow_bg".to_string(),
        value: BuiBindingValue::Color(if follow_state.followed {
            "#4a92e3F2".to_string()
        } else {
            "#8e9aa6CC".to_string()
        }),
    });
    state_writer.write(BuiStateSet {
        key: "hero.follow_border_width".to_string(),
        value: BuiBindingValue::Text(if follow_state.followed {
            "2px".to_string()
        } else {
            "1px".to_string()
        }),
    });

    for (entity, id) in &ids {
        if id.0 == "btn_follow" || id.0 == "txt_follow" {
            if follow_state.followed {
                commands
                    .entity(entity)
                    .insert(BuiVisualState("followed".to_string()));
            } else {
                commands.entity(entity).remove::<BuiVisualState>();
            }
        }
    }
}

fn sync_share_note_toggle_system(
    toggles: Query<(&BuiId, Has<Checked>), With<BuiToggle>>,
    mut state_writer: MessageWriter<BuiStateSet>,
    mut last_checked: Local<Option<bool>>,
) {
    for (id, checked) in &toggles {
        if id.0 != "toggle_share_note" {
            continue;
        }

        if *last_checked == Some(checked) {
            continue;
        }

        *last_checked = Some(checked);
        state_writer.write(BuiStateSet {
            key: "ui.shared_note_visible".to_string(),
            value: BuiBindingValue::Bool(checked),
        });
    }
}

fn emit_tab_binding_updates_system(
    ui_state: Res<UltramanUiState>,
    mut state_writer: MessageWriter<BuiStateSet>,
    mut initialized: Local<bool>,
) {
    if !ui_state.is_changed() && *initialized {
        return;
    }
    *initialized = true;

    let detail_title = match ui_state.selected_tab {
        UltramanTab::Attribute => "属性详情",
        UltramanTab::Skill => "技能详情",
        UltramanTab::Unique => "专属详情",
        UltramanTab::Core => "核心详情",
        UltramanTab::Amplifier => "增幅器详情",
    };

    let subtitle = match ui_state.selected_tab {
        UltramanTab::Attribute => "光辉赛罗",
        UltramanTab::Skill => "技能面板",
        UltramanTab::Unique => "专属回路",
        UltramanTab::Core => "核心配置",
        UltramanTab::Amplifier => "增幅器",
    };

    let rank_justify = match ui_state.selected_tab {
        UltramanTab::Attribute => "center",
        UltramanTab::Skill => "left",
        UltramanTab::Unique => "right",
        UltramanTab::Core => "center",
        UltramanTab::Amplifier => "center",
    };

    let rank_bounds_width = match ui_state.selected_tab {
        UltramanTab::Attribute => 28.0,
        UltramanTab::Skill => 30.0,
        UltramanTab::Unique => 32.0,
        UltramanTab::Core => 30.0,
        UltramanTab::Amplifier => 34.0,
    };

    let rank_bounds_height = match ui_state.selected_tab {
        UltramanTab::Attribute => 40.0,
        UltramanTab::Skill => 44.0,
        UltramanTab::Unique => 46.0,
        UltramanTab::Core => 42.0,
        UltramanTab::Amplifier => 48.0,
    };

    let subtitle_color = match ui_state.selected_tab {
        UltramanTab::Attribute => "#efaa18FF",
        UltramanTab::Skill => "#4f7fe0FF",
        UltramanTab::Unique => "#8a4fd9FF",
        UltramanTab::Core => "#2da38bFF",
        UltramanTab::Amplifier => "#efaa18FF",
    };

    let subtitle_size = match ui_state.selected_tab {
        UltramanTab::Attribute => 19.0,
        UltramanTab::Skill => 18.0,
        UltramanTab::Unique => 20.0,
        UltramanTab::Core => 18.0,
        UltramanTab::Amplifier => 22.0,
    };

    let subtitle_line_height = match ui_state.selected_tab {
        UltramanTab::Attribute => 1.15,
        UltramanTab::Skill => 1.05,
        UltramanTab::Unique => 1.25,
        UltramanTab::Core => 1.10,
        UltramanTab::Amplifier => 1.20,
    };

    let subtitle_letter_spacing = match ui_state.selected_tab {
        UltramanTab::Attribute => 0.0,
        UltramanTab::Skill => 0.5,
        UltramanTab::Unique => 1.5,
        UltramanTab::Core => 0.2,
        UltramanTab::Amplifier => 2.0,
    };

    let subtitle_shadow_color = match ui_state.selected_tab {
        UltramanTab::Attribute => "#7A3C00AA",
        UltramanTab::Skill => "#163F87AA",
        UltramanTab::Unique => "#4A2677AA",
        UltramanTab::Core => "#155947AA",
        UltramanTab::Amplifier => "#8B4C12CC",
    };

    let subtitle_shadow_offset_x = match ui_state.selected_tab {
        UltramanTab::Attribute => 2.0,
        UltramanTab::Skill => 1.0,
        UltramanTab::Unique => 3.0,
        UltramanTab::Core => 1.5,
        UltramanTab::Amplifier => 2.5,
    };

    let subtitle_shadow_offset_y = match ui_state.selected_tab {
        UltramanTab::Attribute => 2.0,
        UltramanTab::Skill => 1.0,
        UltramanTab::Unique => 3.0,
        UltramanTab::Core => 1.5,
        UltramanTab::Amplifier => 2.5,
    };

    let form_icon_tint = match ui_state.selected_tab {
        UltramanTab::Attribute => "#efaa18FF",
        UltramanTab::Skill => "#4f7fe0FF",
        UltramanTab::Unique => "#8a4fd9FF",
        UltramanTab::Core => "#2da38bFF",
        UltramanTab::Amplifier => "#c65f1bFF",
    };

    let power_rotation = match ui_state.selected_tab {
        UltramanTab::Attribute => "-9deg",
        UltramanTab::Skill => "-5deg",
        UltramanTab::Unique => "-18deg",
        UltramanTab::Core => "-2deg",
        UltramanTab::Amplifier => "-13deg",
    };

    let power_scale = match ui_state.selected_tab {
        UltramanTab::Attribute => "0.96 0.96",
        UltramanTab::Skill => "1.02 1.02",
        UltramanTab::Unique => "1.08 1.08",
        UltramanTab::Core => "0.94 0.94",
        UltramanTab::Amplifier => "1.00 1.00",
    };

    let power_translation = match ui_state.selected_tab {
        UltramanTab::Attribute => "-1% 0px",
        UltramanTab::Skill => "1% -2px",
        UltramanTab::Unique => "2% 2px",
        UltramanTab::Core => "-2% -1px",
        UltramanTab::Amplifier => "0px 0px",
    };

    let power_value = match ui_state.selected_tab {
        UltramanTab::Attribute => "10006325",
        UltramanTab::Skill => "10018880",
        UltramanTab::Unique => "10025470",
        UltramanTab::Core => "10031720",
        UltramanTab::Amplifier => "10040250",
    };

    let amplifier_modules: Vec<HashMap<String, String>> = match ui_state.selected_tab {
        UltramanTab::Attribute => vec![
            module_item("基础属性增幅", "力量提升 8%"),
            module_item("体魄增幅回路", "生命提升 5%"),
        ],
        UltramanTab::Skill => vec![
            module_item("技能充能缩短", "冷却减少 0.8 秒"),
            module_item("连携倍率提升", "终结技伤害 +12%"),
        ],
        UltramanTab::Unique => vec![
            module_item("专属回路共鸣", "专属效果激活"),
            module_item("特殊词条激活", "暴击词条 +1"),
        ],
        UltramanTab::Core => vec![
            module_item("核心回路优化", "核心效率 +6%"),
            module_item("能量损耗下降", "能量消耗 -4%"),
        ],
        UltramanTab::Amplifier => vec![
            module_item("增幅器面板内容", "当前槽位已激活"),
            module_item("炽热扩容模块", "攻击提升 +18"),
            module_item("暴击强化模块", "暴击率提升 +3%"),
        ],
    };

    let exp_progress_ratio = match ui_state.selected_tab {
        UltramanTab::Attribute => 0.15,
        UltramanTab::Skill => 0.42,
        UltramanTab::Unique => 0.68,
        UltramanTab::Core => 0.33,
        UltramanTab::Amplifier => 0.0,
    };

    state_writer.write(BuiStateSet {
        key: "ui.detail_title".to_string(),
        value: BuiBindingValue::Text(detail_title.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle".to_string(),
        value: BuiBindingValue::Text(subtitle.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.rank_justify".to_string(),
        value: BuiBindingValue::Text(rank_justify.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.rank_bounds_width".to_string(),
        value: BuiBindingValue::Number(rank_bounds_width),
    });
    state_writer.write(BuiStateSet {
        key: "hero.rank_bounds_height".to_string(),
        value: BuiBindingValue::Number(rank_bounds_height),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_color".to_string(),
        value: BuiBindingValue::Color(subtitle_color.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_size".to_string(),
        value: BuiBindingValue::Number(subtitle_size),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_line_height".to_string(),
        value: BuiBindingValue::Number(subtitle_line_height),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_letter_spacing".to_string(),
        value: BuiBindingValue::Number(subtitle_letter_spacing),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_shadow_color".to_string(),
        value: BuiBindingValue::Color(subtitle_shadow_color.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_shadow_offset_x".to_string(),
        value: BuiBindingValue::Number(subtitle_shadow_offset_x),
    });
    state_writer.write(BuiStateSet {
        key: "hero.subtitle_shadow_offset_y".to_string(),
        value: BuiBindingValue::Number(subtitle_shadow_offset_y),
    });
    state_writer.write(BuiStateSet {
        key: "hero.form_icon_tint".to_string(),
        value: BuiBindingValue::Color(form_icon_tint.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "ui.power_translation".to_string(),
        value: BuiBindingValue::Text(power_translation.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "ui.power_rotation".to_string(),
        value: BuiBindingValue::Text(power_rotation.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "ui.power_scale".to_string(),
        value: BuiBindingValue::Text(power_scale.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.power_value".to_string(),
        value: BuiBindingValue::Text(power_value.to_string()),
    });
    state_writer.write(BuiStateSet {
        key: "hero.amplifier_modules".to_string(),
        value: BuiBindingValue::ObjectList(amplifier_modules),
    });
    state_writer.write(BuiStateSet {
        key: "hero.exp_progress_ratio".to_string(),
        value: BuiBindingValue::Number(exp_progress_ratio),
    });
    for (key, tab) in [
        ("ui.panel.attribute_display", UltramanTab::Attribute),
        ("ui.panel.skill_display", UltramanTab::Skill),
        ("ui.panel.unique_display", UltramanTab::Unique),
        ("ui.panel.core_display", UltramanTab::Core),
        ("ui.panel.amplifier_display", UltramanTab::Amplifier),
    ] {
        state_writer.write(BuiStateSet {
            key: key.to_string(),
            value: BuiBindingValue::Text(if ui_state.selected_tab == tab {
                "flex".to_string()
            } else {
                "none".to_string()
            }),
        });
    }
}

fn module_item(title: &str, desc: &str) -> HashMap<String, String> {
    HashMap::from([
        ("title".to_string(), title.to_string()),
        ("desc".to_string(), desc.to_string()),
    ])
}
