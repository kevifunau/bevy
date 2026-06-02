use bevy::prelude::*;
use std::collections::HashMap;

use crate::FmvAppState;

pub struct BlackboardPlugin;

impl Plugin for BlackboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlackboardResource::default())
            .insert_resource(SettingsResource::default())
            .add_systems(OnEnter(FmvAppState::Settings), load_settings_to_blackboard)
            .add_systems(OnExit(FmvAppState::Settings), save_settings_from_blackboard);
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlackboardResource {
    pub values: HashMap<String, f64>,
}

impl BlackboardResource {
    pub fn get(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied()
    }

    pub fn set(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn apply_mutations(&mut self, mutations: &[String]) {
        for expr in mutations {
            if let Some((key, op, val)) = parse_mutation(expr) {
                let current = self.values.get(&key).copied().unwrap_or(0.0);
                let new_val = match op {
                    MutationOp::Add => current + val,
                    MutationOp::Sub => current - val,
                    MutationOp::Set => val,
                };
                self.values.insert(key, new_val);
            }
        }
    }

    pub fn from_initial(data: &serde_json::Value) -> Self {
        let mut bb = Self::default();
        if let Some(obj) = data.as_object() {
            for (category, inner) in obj {
                if let Some(inner_obj) = inner.as_object() {
                    for (field, val) in inner_obj {
                        let key = format!("{category}.{field}");
                        if let Some(num) = val.as_f64() {
                            bb.values.insert(key, num);
                        }
                    }
                }
            }
        }
        bb
    }
}

enum MutationOp {
    Add,
    Sub,
    Set,
}

fn parse_mutation(expr: &str) -> Option<(String, MutationOp, f64)> {
    let expr = expr.trim();
    if expr.contains("+=") {
        let parts: Vec<&str> = expr.splitn(2, "+=").collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val: f64 = parts[1].trim().parse().ok()?;
            return Some((key.to_string(), MutationOp::Add, val));
        }
    }
    if expr.contains("-=") {
        let parts: Vec<&str> = expr.splitn(2, "-=").collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val: f64 = parts[1].trim().parse().ok()?;
            return Some((key.to_string(), MutationOp::Sub, val));
        }
    }
    if expr.contains("=") && !expr.contains("+=") && !expr.contains("-=") && !expr.contains(">=") && !expr.contains("<=") {
        let parts: Vec<&str> = expr.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val: f64 = parts[1].trim().parse().ok()?;
            return Some((key.to_string(), MutationOp::Set, val));
        }
    }
    None
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SettingsResource {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub language_index: usize,
    pub languages: Vec<String>,
}

impl SettingsResource {
    pub fn default_values() -> Self {
        Self {
            master_volume: 60.0,
            music_volume: 100.0,
            sfx_volume: 20.0,
            language_index: 1,
            languages: vec!["简体中文".into(), "English".into(), "日本語".into()],
        }
    }
}

fn load_settings_to_blackboard(mut settings: ResMut<SettingsResource>) {
    if settings.languages.is_empty() {
        *settings = SettingsResource::default_values();
    }
}

fn save_settings_from_blackboard(_settings: Res<SettingsResource>) {}