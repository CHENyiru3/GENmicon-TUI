use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

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
    let active_npcs = active_npc_actors(state);
    let mut packs = Vec::new();

    for role in &driver.subagents.default_roles {
        if packs.len() >= max_active {
            break;
        }
        if expands_to_individual_npcs(role) && !active_npcs.is_empty() {
            for npc in &active_npcs {
                if packs.len() >= max_active {
                    break;
                }
                let generated_role = format!("{}_{}", role, safe_role_segment(&npc.id));
                packs.push(build_pack(
                    driver,
                    state,
                    &scene,
                    &functions,
                    &generated_role,
                    role,
                    Some(npc),
                ));
            }
            continue;
        }
        packs.push(build_pack(
            driver, state, &scene, &functions, role, role, None,
        ));
    }

    packs
}

fn build_pack(
    driver: &DriverManifest,
    state: &Value,
    scene: &Value,
    functions: &[String],
    role: &str,
    base_role: &str,
    npc: Option<&ActorDescriptor>,
) -> AgentPack {
    AgentPack {
        role: role.to_string(),
        output_contract: output_contract_for(role, npc),
        allowed_files: allowed_files_for(role, base_role, npc, &driver.subagents.templates),
        current_scene: scene.clone(),
        relevant_save_slice: save_slice_for(base_role, state, npc),
        callable_driver_functions: functions.to_vec(),
        assigned_skills: assigned_skills_for(driver, npc),
    }
}

fn output_contract_for(role: &str, npc: Option<&ActorDescriptor>) -> String {
    if let Some(npc) = npc {
        let name = npc.name.as_deref().unwrap_or(&npc.id);
        return format!(
            "{role} proposes dialogue, reactions, memories, and visible actions only for NPC {name} ({}); authoritative commits stay with game_commit_turn.",
            npc.id
        );
    }
    format!(
        "{role} proposes scoped game state, plot, or dialogue updates; authoritative commits stay with game_commit_turn."
    )
}

fn allowed_files_for(
    role: &str,
    base_role: &str,
    npc: Option<&ActorDescriptor>,
    templates: &BTreeMap<String, PathBuf>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(path) = templates.get(role) {
        push_unique_path(&mut files, path.clone());
    }
    if role != base_role
        && let Some(path) = templates.get(base_role)
    {
        push_unique_path(&mut files, path.clone());
    }
    if (base_role.starts_with("npc_manager") || expands_to_individual_npcs(base_role))
        && let Some(path) = templates.get("npc_manager")
    {
        push_unique_path(&mut files, path.clone());
    }
    if let Some(npc) = npc {
        push_unique_path(&mut files, PathBuf::from("content/scene.md"));
        push_unique_path(&mut files, PathBuf::from("content/backstory.md"));
        push_unique_path(
            &mut files,
            PathBuf::from("skills")
                .join("npc")
                .join(&npc.id)
                .join("SKILL.md"),
        );
    }
    files
}

fn assigned_skills_for(driver: &DriverManifest, npc: Option<&ActorDescriptor>) -> Vec<PathBuf> {
    let mut skills = driver.skills.entry.iter().cloned().collect::<Vec<_>>();
    if let Some(npc) = npc {
        push_unique_path(
            &mut skills,
            PathBuf::from("skills")
                .join("npc")
                .join(&npc.id)
                .join("SKILL.md"),
        );
    }
    skills
}

fn save_slice_for(role: &str, state: &Value, npc: Option<&ActorDescriptor>) -> Value {
    if let Some(npc) = npc {
        return npc_save_slice_for(npc, state);
    }
    if role == "state_manager" || role == "state" {
        return serde_json::json!({
            "scene": state.get("scene").cloned().unwrap_or(Value::Null),
            "player": state.get("player").cloned().unwrap_or(Value::Null),
            "world": state.get("world").cloned().unwrap_or(Value::Null),
            "agents": state.get("agents").cloned().unwrap_or(Value::Null)
        });
    }
    if role == "plot_manager" || role == "plot" {
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

fn npc_save_slice_for(npc: &ActorDescriptor, state: &Value) -> Value {
    json!({
        "scene": state.get("scene").cloned().unwrap_or(Value::Null),
        "conversation": state.get("conversation").cloned().unwrap_or(Value::Null),
        "npc": npc.cast.clone(),
        "player": state.get("player").cloned().unwrap_or(Value::Null),
        "backstory": state.get("backstory").cloned().unwrap_or(Value::Null),
        "facts": actor_fact_slice(npc, state),
        "world": {
            "flags": state.pointer("/world/flags").cloned().unwrap_or(Value::Null),
            "actors": [npc.id.clone()]
        },
        "story": {
            "style": state.pointer("/story/style").cloned().unwrap_or(Value::Null),
            "active_branch": state.pointer("/story/active_branch").cloned().unwrap_or(Value::Null),
            "active_node": state.pointer("/story/active_node").cloned().unwrap_or(Value::Null)
        }
    })
}

#[derive(Debug, Clone)]
struct ActorDescriptor {
    id: String,
    name: Option<String>,
    cast: Value,
}

fn active_npc_actors(state: &Value) -> Vec<ActorDescriptor> {
    let cast = state
        .get("cast")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let cast_by_id = cast
        .iter()
        .filter_map(|entry| Some((entry.get("id")?.as_str()?.to_string(), entry)))
        .collect::<BTreeMap<_, _>>();
    let mut ids = state
        .pointer("/world/actors")
        .and_then(Value::as_array)
        .map(|actors| {
            actors
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if ids.is_empty() {
        ids = cast_by_id.keys().cloned().collect();
    }

    let mut actors = Vec::new();
    for id in ids {
        let cast = cast_by_id
            .get(&id)
            .map(|entry| (*entry).clone())
            .unwrap_or_else(|| json!({ "id": id.clone() }));
        if is_player_actor(&id, &cast) {
            continue;
        }
        let name = cast.get("name").and_then(Value::as_str).map(str::to_string);
        actors.push(ActorDescriptor { id, name, cast });
    }
    actors
}

fn is_player_actor(id: &str, actor: &Value) -> bool {
    if id == "player" {
        return true;
    }
    actor
        .get("role")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .is_some_and(|role| role.contains("player character"))
}

fn expands_to_individual_npcs(role: &str) -> bool {
    matches!(role, "dialogue" | "npc" | "npc_manager")
}

fn actor_fact_slice(npc: &ActorDescriptor, state: &Value) -> Value {
    let Some(all_facts) = state.get("facts").and_then(Value::as_object) else {
        return Value::Null;
    };
    let mut facts = Map::new();
    copy_fact_key(all_facts, &mut facts, "relationship");
    copy_fact_key(all_facts, &mut facts, &npc.id);
    if let Some(key) = npc.cast.get("fact_key").and_then(Value::as_str) {
        copy_fact_key(all_facts, &mut facts, key);
    }
    if let Some(name) = &npc.name {
        copy_fact_key(all_facts, &mut facts, &fact_key(name));
    }
    Value::Object(facts)
}

fn copy_fact_key(all_facts: &Map<String, Value>, facts: &mut Map<String, Value>, key: &str) {
    if let Some(value) = all_facts.get(key) {
        facts.insert(key.to_string(), value.clone());
    }
}

fn fact_key(raw: &str) -> String {
    raw.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                byte.to_ascii_lowercase() as char
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn safe_role_segment(raw: &str) -> String {
    raw.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
                byte as char
            } else {
                '_'
            }
        })
        .collect()
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}
