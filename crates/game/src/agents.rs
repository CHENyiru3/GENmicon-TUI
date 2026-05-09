use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::driver::DriverManifest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentPack {
    pub role: String,
    pub output_contract: String,
    pub allowed_files: Vec<PathBuf>,
    pub current_scene: Value,
    pub relevant_save_slice: Value,
    pub callable_driver_functions: Vec<String>,
    pub assigned_skills: Vec<PathBuf>,
}

pub fn build_agent_packs(driver: &DriverManifest, state: &Value) -> Vec<AgentPack> {
    let max_active = driver.subagents.max_active;
    let functions = driver.functions.keys().cloned().collect::<Vec<_>>();
    let scene = state.get("scene").cloned().unwrap_or(Value::Null);

    driver
        .subagents
        .default_roles
        .iter()
        .take(max_active)
        .map(|role| AgentPack {
            role: role.clone(),
            output_contract: output_contract_for(role),
            allowed_files: allowed_files_for(role, &driver.subagents.templates),
            current_scene: scene.clone(),
            relevant_save_slice: save_slice_for(role, state),
            callable_driver_functions: functions.clone(),
            assigned_skills: driver.skills.entry.iter().cloned().collect(),
        })
        .collect()
}

fn output_contract_for(role: &str) -> String {
    format!(
        "{role} proposes scoped game state, plot, or dialogue updates; authoritative commits stay with game_commit_turn."
    )
}

fn allowed_files_for(role: &str, templates: &BTreeMap<String, PathBuf>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(path) = templates.get(role) {
        files.push(path.clone());
    }
    if role.starts_with("npc_manager")
        && let Some(path) = templates.get("npc_manager")
    {
        files.push(path.clone());
    }
    files
}

fn save_slice_for(role: &str, state: &Value) -> Value {
    if role == "state_manager" {
        return serde_json::json!({
            "scene": state.get("scene").cloned().unwrap_or(Value::Null),
            "player": state.get("player").cloned().unwrap_or(Value::Null),
            "world": state.get("world").cloned().unwrap_or(Value::Null),
            "agents": state.get("agents").cloned().unwrap_or(Value::Null)
        });
    }
    if role == "plot_manager" {
        return serde_json::json!({
            "scene": state.get("scene").cloned().unwrap_or(Value::Null),
            "world": {
                "flags": state.pointer("/world/flags").cloned().unwrap_or(Value::Null),
                "quests": state.pointer("/world/quests").cloned().unwrap_or(Value::Null)
            }
        });
    }
    serde_json::json!({
        "scene": state.get("scene").cloned().unwrap_or(Value::Null),
        "actors": state.pointer("/world/actors").cloned().unwrap_or(Value::Null)
    })
}
