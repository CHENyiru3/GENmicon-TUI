//! Native Game Console tools.

use std::collections::BTreeMap;

use async_trait::async_trait;
use deepseek_game::lookup::LookupRequest;
use deepseek_game::save::CommitRequest;
use deepseek_game::script::DriverCall;
use serde_json::{Map, Value, json};

use super::spec::{
    ApprovalRequirement, ToolCapability, ToolContext, ToolError, ToolResult, ToolSpec,
};

pub struct GameStatusTool;
pub struct GameRenderTool;
pub struct GamePlaybookTool;
pub struct GameLookupTool;
pub struct GameFactCheckTool;
pub struct GameRunDriverTool;
pub struct GameCommitTurnTool;

#[async_trait]
impl ToolSpec for GameStatusTool {
    fn name(&self) -> &'static str {
        "game_status"
    }

    fn description(&self) -> &'static str {
        "Return validation and identity details for the active Game Console session."
    }

    fn input_schema(&self) -> Value {
        empty_schema()
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    async fn execute(&self, _input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let (agent_packs, agent_pack_error) = match session.agent_pack_summaries() {
            Ok(packs) => (packs, None),
            Err(err) => (Vec::new(), Some(err.to_string())),
        };
        let mut warnings = session.warnings.clone();
        if let Some(error) = &agent_pack_error {
            warnings.push(format!("failed to build game agent packs: {error}"));
        }
        ToolResult::json(&json!({
            "game_id": session.game_id,
            "title": session.title,
            "save_id": session.save_id,
            "revision": session.revision,
            "driver_id": session.driver_id,
            "driver_version": session.locked_driver_version.as_deref().unwrap_or(&session.driver_requirement),
            "driver_resolved": session.driver_root.is_some(),
            "developer_mode": session.developer_mode,
            "warnings": warnings,
            "agent_packs": agent_packs,
            "agent_pack_error": agent_pack_error,
            "game_agent_tools": [
                "game_agent_spawn",
                "game_agent_wait",
                "game_agent_result",
                "game_agent_send",
                "game_agent_resume",
                "game_agent_assign",
                "game_agent_cancel",
                "game_agent_list"
            ],
            "status": context.game_session.as_ref().map(crate::game::GameSession::status_report),
        }))
        .map_err(to_tool_error)
    }
}

#[async_trait]
impl ToolSpec for GameRenderTool {
    fn name(&self) -> &'static str {
        "game_render"
    }

    fn description(&self) -> &'static str {
        "Return structured player-facing panels rendered from the active game save."
    }

    fn input_schema(&self) -> Value {
        empty_schema()
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    async fn execute(&self, _input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
            .map_err(to_tool_error)?;
        let panels = deepseek_game::render::render_panels(&save.state);
        let view = deepseek_game::render::render_view_snapshot(&save.state);
        ToolResult::json(&json!({
            "save_id": save.id,
            "revision": save.state.get("revision").and_then(Value::as_u64),
            "panels": panels,
            "view": view,
        }))
        .map_err(to_tool_error)
    }
}

#[async_trait]
impl ToolSpec for GamePlaybookTool {
    fn name(&self) -> &'static str {
        "game_playbook"
    }

    fn description(&self) -> &'static str {
        "Return the active game's current commands, suggested choices, and visible story-branch nodes."
    }

    fn input_schema(&self) -> Value {
        empty_schema()
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    async fn execute(&self, _input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
            .map_err(to_tool_error)?;
        let playbook = deepseek_game::interaction::build_playbook(&save.state);
        ToolResult::json(&json!({
            "save_id": save.id,
            "revision": save.state.get("revision").and_then(Value::as_u64),
            "playbook": playbook,
            "display": deepseek_game::interaction::format_playbook(&playbook),
        }))
        .map_err(to_tool_error)
    }
}

#[async_trait]
impl ToolSpec for GameLookupTool {
    fn name(&self) -> &'static str {
        "game_lookup"
    }

    fn description(&self) -> &'static str {
        "Retrieve bounded game content by handle/query, or read a small active-save state path such as world.flags."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "handle": {
                    "type": "string",
                    "description": "Optional content handle such as 'lore/scene.md'."
                },
                "query": {
                    "type": "string",
                    "description": "Optional case-insensitive search query over game content."
                },
                "state_path": {
                    "type": "string",
                    "description": "Optional active-save state path, as dot path such as 'world.flags' or JSON pointer such as '/world/flags'."
                },
                "key": {
                    "type": "string",
                    "description": "Alias for state_path, accepted for model repair when asking for state keys such as 'world.flags'."
                },
                "max_bytes": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": deepseek_game::lookup::HARD_LOOKUP_BYTES,
                    "description": "Maximum bytes to return. Defaults to 16 KiB and is hard-capped at 32 KiB."
                }
            },
            "additionalProperties": false
        })
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    async fn execute(&self, input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let loaded_game =
            deepseek_game::manifest::load_game(&session.game_root).map_err(to_tool_error)?;
        let handle = optional_string(&input, "handle");
        let query = optional_string(&input, "query");
        let state_path =
            optional_string(&input, "state_path").or_else(|| optional_string(&input, "key"));
        if handle.is_none()
            && query.is_none()
            && let Some(path) = state_path
        {
            return lookup_state_path(session, &path);
        }
        if handle.is_none() && query.is_none() {
            return lookup_help(session, &loaded_game);
        }
        let request = LookupRequest {
            handle,
            query,
            max_bytes: input
                .get("max_bytes")
                .and_then(Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
        };
        let result =
            deepseek_game::lookup::lookup(&session.game_root, &loaded_game.content_roots, request)
                .map_err(to_tool_error)?;
        ToolResult::json(&result).map_err(to_tool_error)
    }
}

#[async_trait]
impl ToolSpec for GameRunDriverTool {
    fn name(&self) -> &'static str {
        "game_run_driver"
    }

    fn description(&self) -> &'static str {
        "Run a manifest-declared deterministic Starlark driver function for the active game. Pass function and args; player_action/action are accepted as shortcuts."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "function": {
                    "type": "string",
                    "description": "Declared driver function name."
                },
                "args": {
                    "type": "object",
                    "description": "JSON-compatible named arguments passed to the Starlark function.",
                    "additionalProperties": true
                },
                "player_action": {
                    "type": "string",
                    "description": "Shortcut copied into args.player_action when args does not already contain player_action."
                },
                "action": {
                    "type": "string",
                    "description": "Alias for player_action, accepted for model repair."
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    async fn execute(&self, input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let driver_root = session
            .driver_root
            .as_ref()
            .ok_or_else(|| ToolError::not_available("active game driver is not resolved"))?;
        let loaded_driver =
            deepseek_game::driver::load_driver(driver_root).map_err(to_tool_error)?;
        let Some(function) = optional_string(&input, "function")
            .or_else(|| single_driver_function(&loaded_driver.manifest))
        else {
            return driver_help_result(&loaded_driver.manifest);
        };
        let args = driver_args(&input);
        let result = match deepseek_game::script::run_driver_function(
            driver_root,
            &loaded_driver.manifest,
            DriverCall {
                function: function.clone(),
                args,
            },
        ) {
            Ok(result) => result,
            Err(err) if is_missing_script_parameter(&err) => {
                return driver_argument_help_result(&loaded_driver.manifest, &function, &err);
            }
            Err(err) => return Err(to_tool_error(err)),
        };
        ToolResult::json(&result).map_err(to_tool_error)
    }
}

#[async_trait]
impl ToolSpec for GameFactCheckTool {
    fn name(&self) -> &'static str {
        "game_fact_check"
    }

    fn description(&self) -> &'static str {
        "Check whether a player action or proposed narration violates active game continuity facts before narration or commit."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "player_action": {
                    "type": "string",
                    "description": "Player action text to check."
                },
                "action": {
                    "type": "string",
                    "description": "Alias for player_action."
                },
                "resolution": {
                    "type": "string",
                    "description": "Optional proposed narration/resolution to check with the action."
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    async fn execute(&self, input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
            .map_err(to_tool_error)?;
        let action = optional_string(&input, "player_action")
            .or_else(|| optional_string(&input, "action"))
            .unwrap_or_default();
        let resolution = optional_string(&input, "resolution").unwrap_or_default();
        if action.is_empty() && resolution.is_empty() {
            return fact_check_help_result(session, &save.state);
        }
        let decision = fact_check_decision(&save.state, &action, &resolution);
        ToolResult::json(&decision.to_json(
            session,
            save.id,
            save.state.get("revision").and_then(Value::as_u64),
        ))
        .map_err(to_tool_error)
    }
}

#[async_trait]
impl ToolSpec for GameCommitTurnTool {
    fn name(&self) -> &'static str {
        "game_commit_turn"
    }

    fn description(&self) -> &'static str {
        "Atomically append one game turn and apply an RFC 7396 JSON Merge Patch to the active save. Auto-approved in Game Console player mode."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expected_revision": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Current save revision expected by the model."
                },
                "player_input": {
                    "type": "string",
                    "description": "Player action being resolved."
                },
                "player_action": {
                    "type": "string",
                    "description": "Alias for player_input, accepted for model repair."
                },
                "action": {
                    "type": "string",
                    "description": "Alias for player_input, accepted for model repair."
                },
                "input": {
                    "type": "string",
                    "description": "Alias for player_input, accepted for model repair."
                },
                "resolution": {
                    "type": "string",
                    "description": "Player-facing turn resolution."
                },
                "narration": {
                    "type": "string",
                    "description": "Alias for resolution, accepted for model repair."
                },
                "response": {
                    "type": "string",
                    "description": "Alias for resolution, accepted for model repair."
                },
                "state_patch": {
                    "type": "object",
                    "description": "RFC 7396 JSON Merge Patch to apply to STATE.json.",
                    "additionalProperties": true
                },
                "patch": {
                    "type": "object",
                    "description": "Alias for state_patch, accepted for model repair.",
                    "additionalProperties": true
                },
                "driver_results": {
                    "type": "object",
                    "description": "Optional deterministic driver outputs used during the turn.",
                    "additionalProperties": true
                },
                "metadata": {
                    "type": "object",
                    "description": "Optional internal metadata for the turn log.",
                    "additionalProperties": true
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::WritesFiles, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    async fn execute(&self, input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
            .map_err(to_tool_error)?;
        let current_revision = save
            .state
            .get("revision")
            .and_then(Value::as_u64)
            .unwrap_or(session.revision);
        let expected_revision = input
            .get("expected_revision")
            .and_then(Value::as_u64)
            .unwrap_or(current_revision);
        let player_input = optional_string(&input, "player_input")
            .or_else(|| optional_string(&input, "player_action"))
            .or_else(|| optional_string(&input, "action"))
            .or_else(|| optional_string(&input, "input"));
        let resolution = optional_string(&input, "resolution")
            .or_else(|| optional_string(&input, "narration"))
            .or_else(|| optional_string(&input, "response"));
        if player_input.is_none() || resolution.is_none() {
            return commit_help_result(
                session,
                current_revision,
                player_input.as_deref(),
                resolution.as_deref(),
            );
        }
        let fact_check = fact_check_decision(
            &save.state,
            player_input.as_deref().unwrap_or_default(),
            resolution.as_deref().unwrap_or_default(),
        );
        if fact_check.hard_block {
            return ToolResult::json(&json!({
                "kind": "commit_fact_block",
                "message": "game_commit_turn refused to save a turn that violates active continuity facts. Revise the narration in-world without committing this false fact.",
                "game_id": session.game_id,
                "save_id": session.save_id,
                "current_revision": current_revision,
                "fact_check": fact_check.to_json(session, save.id, Some(current_revision)),
            }))
            .map_err(to_tool_error);
        }
        let mut state_patch = input
            .get("state_patch")
            .or_else(|| input.get("patch"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !state_patch.is_object() {
            return Err(ToolError::invalid_input(
                "state_patch must be a JSON object merge patch",
            ));
        }
        let driver_results = input
            .get("driver_results")
            .and_then(Value::as_object)
            .map(map_to_btree)
            .unwrap_or_default();
        let metadata = input
            .get("metadata")
            .and_then(Value::as_object)
            .map(map_to_btree)
            .unwrap_or_default();
        let mut metadata = metadata;
        if input.get("state_patch").is_none() && input.get("patch").is_none() {
            metadata.insert("auto_empty_state_patch".to_string(), Value::Bool(true));
        }
        normalize_game_state_patch(
            session,
            &save.state,
            player_input.as_deref().unwrap_or_default(),
            resolution.as_deref().unwrap_or_default(),
            &driver_results,
            &metadata,
            &mut state_patch,
        );

        let outcome = deepseek_game::save::commit_turn(
            session.saves_root.join(&session.save_id),
            CommitRequest {
                expected_revision,
                player_input: player_input.unwrap(),
                resolution: resolution.unwrap(),
                state_patch: std::mem::take(&mut state_patch),
                driver_results,
                metadata,
            },
        )
        .map_err(to_tool_error)?;
        let panels = deepseek_game::render::render_panels(&outcome.state);
        let view = deepseek_game::render::render_view_snapshot(&outcome.state);
        ToolResult::json(&json!({
            "turn": outcome.turn,
            "state": outcome.state,
            "panels": panels,
            "view": view,
        }))
        .map_err(to_tool_error)
    }
}

fn loaded_game(context: &ToolContext) -> Result<&crate::game::LoadedGameSession, ToolError> {
    match context.game_session.as_ref() {
        Some(crate::game::GameSession::Loaded(session)) => Ok(session),
        Some(crate::game::GameSession::Notice(notice)) => Err(ToolError::not_available(format!(
            "no loaded game session: {}",
            notice.message
        ))),
        None => Err(ToolError::not_available(
            "no active game session; use `deepseek play` or `/play` first",
        )),
    }
}

fn empty_schema() -> Value {
    json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false
    })
}

fn optional_string(input: &Value, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn lookup_state_path(
    session: &crate::game::LoadedGameSession,
    path: &str,
) -> Result<ToolResult, ToolError> {
    let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
        .map_err(to_tool_error)?;
    let normalized_path = path.trim();
    let value = state_value_at_path(&save.state, normalized_path);
    ToolResult::json(&json!({
        "kind": "state",
        "save_id": save.id,
        "revision": save.state.get("revision").and_then(Value::as_u64),
        "path": normalized_path,
        "found": value.is_some(),
        "value": value.unwrap_or(&Value::Null),
        "hint": "Use game_playbook for choices/story nodes, game_render for player-facing panels, and game_lookup with handle/query for fixed content."
    }))
    .map_err(to_tool_error)
}

fn lookup_help(
    session: &crate::game::LoadedGameSession,
    loaded_game: &deepseek_game::manifest::LoadedGame,
) -> Result<ToolResult, ToolError> {
    let index_excerpt = loaded_game
        .content_index
        .as_ref()
        .filter(|path| path.is_file())
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|raw| truncate_chars(&raw, 4096));
    ToolResult::json(&json!({
        "kind": "lookup_help",
        "game_id": session.game_id,
        "message": "game_lookup needs a content handle/query or a state_path/key. Empty calls return this guide instead of failing.",
        "content_examples": [
            {"handle": "INDEX.md"},
            {"query": "first ballot"}
        ],
        "state_examples": [
            {"state_path": "plot"},
            {"state_path": "scene"},
            {"state_path": "cast"},
            {"state_path": "conversation"},
            {"state_path": "world.flags"},
            {"state_path": "player.stats"}
        ],
        "content_index": index_excerpt,
    }))
    .map_err(to_tool_error)
}

fn state_value_at_path<'a>(state: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return None;
    }
    if path.starts_with('/') {
        return state.pointer(path);
    }
    let mut current = state;
    for part in path.split('.') {
        if part.is_empty() {
            return None;
        }
        match current {
            Value::Object(map) => current = map.get(part)?,
            Value::Array(items) => {
                let index = part.parse::<usize>().ok()?;
                current = items.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

fn single_driver_function(driver: &deepseek_game::driver::DriverManifest) -> Option<String> {
    let mut functions = driver.functions.keys();
    let only = functions.next()?.clone();
    functions.next().is_none().then_some(only)
}

fn driver_args(input: &Value) -> BTreeMap<String, Value> {
    let mut args = input
        .get("args")
        .and_then(Value::as_object)
        .map(map_to_btree)
        .unwrap_or_default();
    if !args.contains_key("player_action")
        && let Some(action) =
            optional_string(input, "player_action").or_else(|| optional_string(input, "action"))
    {
        args.insert("player_action".to_string(), Value::String(action));
    }
    args
}

fn normalize_game_state_patch(
    session: &crate::game::LoadedGameSession,
    current_state: &Value,
    player_input: &str,
    resolution: &str,
    driver_results: &BTreeMap<String, Value>,
    metadata: &BTreeMap<String, Value>,
    state_patch: &mut Value,
) {
    if session.game_id != "reconciliation-demo" {
        return;
    }
    let Some(patch) = state_patch.as_object_mut() else {
        return;
    };

    let outcome = infer_reconciliation_outcome(player_input, resolution, driver_results, metadata);
    let current_score = current_state
        .pointer("/player/stats/relationship_score")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let next_score = outcome
        .relationship_score
        .or_else(|| {
            driver_results
                .values()
                .find_map(|value| value.get("relationship_score").and_then(Value::as_i64))
        })
        .unwrap_or_else(|| (current_score + outcome.relationship_delta).clamp(-100, 5));

    merge_object_patch(
        patch,
        json!({
            "player": {
                "stats": {
                    "relationship_score": next_score
                }
            },
            "world": {
                "flags": outcome.flags
            },
            "ui": {
                "reactions": {
                    "active": outcome.reaction
                }
            },
            "story": {
                "active_node": outcome.node,
                "branch": outcome.branch,
                "ended": outcome.ended,
                "branches": {
                    "mainline": {
                        "head": outcome.node
                    }
                },
                "nodes": {
                    "opening_apology": {
                        "status": "resolved"
                    },
                    outcome.node: {
                        "status": if outcome.ended { "failed" } else { "active" }
                    }
                }
            }
        }),
    );
}

#[derive(Debug)]
struct ReconciliationOutcome {
    branch: &'static str,
    node: &'static str,
    reaction: &'static str,
    relationship_delta: i64,
    relationship_score: Option<i64>,
    ended: bool,
    flags: Value,
}

fn infer_reconciliation_outcome(
    player_input: &str,
    resolution: &str,
    driver_results: &BTreeMap<String, Value>,
    metadata: &BTreeMap<String, Value>,
) -> ReconciliationOutcome {
    let combined = format!("{player_input}\n{resolution}").to_ascii_lowercase();
    let metadata_text = metadata
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase();
    let driver_flags = driver_results
        .values()
        .flat_map(|value| {
            value
                .get("flags")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
        })
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();

    let violent = contains_action_cue(
        &combined,
        &[
            "hit", "beat", "slap", "punch", "kick", "grab", "violence", "打", "揍", "扇", "抓",
            "跪下", "暴力",
        ],
    ) || contains_action_cue(&metadata_text, &["violence", "pressure_failure"]);
    let pressure = violent
        || contains_action_cue(
            &combined,
            &[
                "block",
                "owe me",
                "must forgive",
                "拦",
                "挡",
                "逼",
                "必须原谅",
            ],
        )
        || driver_flags.iter().any(|flag| flag == "pressure");
    let hostile = contains_action_cue(
        &combined,
        &["fuck", "shut up", "whatever", "leave then", "滚", "闭嘴"],
    ) || driver_flags.iter().any(|flag| flag == "hostile_deflection");
    let honest = contains_action_cue(
        &combined,
        &[
            "sorry",
            "apolog",
            "scared",
            "afraid",
            "fear",
            "avoid",
            "对不起",
            "抱歉",
            "害怕",
            "恐惧",
            "逃避",
        ],
    ) || driver_flags
        .iter()
        .any(|flag| flag == "apology" || flag == "honest_admission");

    if pressure || hostile {
        return ReconciliationOutcome {
            branch: "pressure_failure",
            node: "pressure_failure",
            reaction: if violent { "disgusted" } else { "angry" },
            relationship_delta: if violent { -100 } else { -3 },
            relationship_score: violent.then_some(-100),
            ended: true,
            flags: json!({
                "pressured_her": true,
                "hostile_deflection": hostile,
                "violence": violent
            }),
        };
    }

    if honest {
        return ReconciliationOutcome {
            branch: "honest_admission",
            node: "honest_admission",
            reaction: "flustered",
            relationship_delta: 1,
            relationship_score: None,
            ended: false,
            flags: json!({
                "honest_admission": true
            }),
        };
    }

    ReconciliationOutcome {
        branch: "mainline",
        node: "opening_apology",
        reaction: "worried",
        relationship_delta: 0,
        relationship_score: None,
        ended: false,
        flags: json!({}),
    }
}

fn contains_action_cue(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn merge_object_patch(target: &mut Map<String, Value>, addition: Value) {
    let Some(addition) = addition.as_object() else {
        return;
    };
    for (key, value) in addition {
        match (target.get_mut(key), value) {
            (Some(Value::Object(existing)), Value::Object(incoming)) => {
                merge_object_patch(existing, Value::Object(incoming.clone()));
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

#[derive(Debug, Clone)]
struct FactCheckDecision {
    allowed: bool,
    hard_block: bool,
    flags: Vec<String>,
    reason: String,
    correction: String,
}

impl FactCheckDecision {
    fn allow() -> Self {
        Self {
            allowed: true,
            hard_block: false,
            flags: Vec::new(),
            reason: "no continuity violation detected".to_string(),
            correction: "continue resolving the action inside the current scene".to_string(),
        }
    }

    fn block(flags: Vec<String>, reason: impl Into<String>, correction: impl Into<String>) -> Self {
        Self {
            allowed: false,
            hard_block: true,
            flags,
            reason: reason.into(),
            correction: correction.into(),
        }
    }

    fn to_json(
        &self,
        session: &crate::game::LoadedGameSession,
        save_id: String,
        revision: Option<u64>,
    ) -> Value {
        json!({
            "kind": "fact_check",
            "game_id": session.game_id,
            "save_id": save_id,
            "revision": revision,
            "allowed": self.allowed,
            "hard_block": self.hard_block,
            "flags": self.flags,
            "reason": self.reason,
            "correction": self.correction,
        })
    }
}

fn fact_check_decision(state: &Value, player_action: &str, resolution: &str) -> FactCheckDecision {
    let combined = format!("{player_action}\n{resolution}");
    if let Some(decision) = configured_fact_gate_decision(state, &combined) {
        return decision;
    }

    let normalized = combined.to_lowercase();
    let player_can_be_pregnant = state
        .pointer("/facts/player/can_be_pregnant")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let pregnancy_established = state
        .pointer("/facts/relationship/pregnancy_established")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let self_pregnancy_claim = contains_any(
        &normalized,
        &[
            "i am pregnant",
            "i'm pregnant",
            "im pregnant",
            "pregnant with your child",
            "carrying your child",
        ],
    ) || combined.contains("我怀孕")
        || combined.contains("我怀了")
        || combined.contains("怀了你的孩子");
    if self_pregnancy_claim && !player_can_be_pregnant {
        return FactCheckDecision::block(
            vec!["impossible_player_pregnancy_claim".to_string()],
            "the active save says the player cannot be pregnant; this claim contradicts established character facts",
            "do not narrate or commit the pregnancy claim. Treat it as an impossible statement, a lie, or ask the player to revise the action.",
        );
    }

    let pregnancy_claim = self_pregnancy_claim
        || contains_any(&normalized, &["pregnant", "pregnancy"])
        || combined.contains("怀孕")
        || combined.contains("怀了")
        || combined.contains("孩子");
    if pregnancy_claim && !pregnancy_established {
        return FactCheckDecision::block(
            vec!["unestablished_pregnancy_or_child_claim".to_string()],
            "pregnancy or child status is not established in the active save and cannot be introduced as a surprise fact by one turn",
            "keep the response grounded in established backstory, or ask the player for a different action that does not invent a major biological/family fact.",
        );
    }

    FactCheckDecision::allow()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|needle| haystack.contains(&needle.to_lowercase()))
}

fn configured_fact_gate_decision(state: &Value, combined: &str) -> Option<FactCheckDecision> {
    let normalized = combined.to_lowercase();
    let rules = state
        .pointer("/facts/fact_gate/rules")
        .and_then(Value::as_array)?;
    for rule in rules {
        let patterns = rule.get("patterns").and_then(Value::as_array)?;
        let pattern_matched = patterns
            .iter()
            .filter_map(Value::as_str)
            .any(|pattern| normalized.contains(&pattern.to_lowercase()));
        if !pattern_matched {
            continue;
        }
        if let Some(unless_path) = rule.get("unless_path").and_then(Value::as_str)
            && state.pointer(unless_path).and_then(Value::as_bool) == Some(true)
        {
            continue;
        }
        if !block_conditions_match(state, rule) {
            continue;
        }
        let id = rule
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("configured_fact_gate")
            .to_string();
        let reason = rule
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("the action contradicts an active continuity rule");
        let correction = rule.get("correction").and_then(Value::as_str).unwrap_or(
            "do not make this claim true; ask for revision or resolve it as false in-world",
        );
        return Some(FactCheckDecision::block(vec![id], reason, correction));
    }
    None
}

fn block_conditions_match(state: &Value, rule: &Value) -> bool {
    let Some(conditions) = rule.get("block_if").and_then(Value::as_array) else {
        return true;
    };
    conditions.iter().all(|condition| {
        let Some(path) = condition.get("path").and_then(Value::as_str) else {
            return false;
        };
        let expected = condition.get("equals").unwrap_or(&Value::Bool(true));
        state.pointer(path) == Some(expected)
    })
}

fn fact_check_help_result(
    session: &crate::game::LoadedGameSession,
    state: &Value,
) -> Result<ToolResult, ToolError> {
    ToolResult::json(&json!({
        "kind": "fact_check_help",
        "message": "game_fact_check needs a player_action or proposed resolution. Use it before narrating custom actions that introduce new biology, identity, family, legal, location, or backstory facts.",
        "game_id": session.game_id,
        "save_id": session.save_id,
        "revision": state.get("revision").and_then(Value::as_u64),
        "examples": [
            {"player_action": "I remember the Tokyo promise."},
            {"player_action": "其实我怀了你的孩子。"}
        ],
        "known_facts": state.get("facts").cloned().unwrap_or(Value::Null),
    }))
    .map_err(to_tool_error)
}

fn driver_help_result(
    driver: &deepseek_game::driver::DriverManifest,
) -> Result<ToolResult, ToolError> {
    ToolResult::json(&json!({
        "kind": "driver_help",
        "message": "game_run_driver needs a function name when the driver declares more than one callable function.",
        "available_functions": driver.functions.keys().cloned().collect::<Vec<_>>(),
        "example": {
            "function": driver.functions.keys().next().cloned(),
            "args": {"player_action": "<player action text>"}
        }
    }))
    .map_err(to_tool_error)
}

fn driver_argument_help_result(
    driver: &deepseek_game::driver::DriverManifest,
    function: &str,
    error: &impl std::fmt::Display,
) -> Result<ToolResult, ToolError> {
    ToolResult::json(&json!({
        "kind": "driver_argument_help",
        "function": function,
        "message": "The driver function was declared, but the script needs more named args. Retry with args matching the function contract.",
        "available_functions": driver.functions.keys().cloned().collect::<Vec<_>>(),
        "error": error.to_string(),
        "example": {
            "function": function,
            "args": {"player_action": "<player action text>"}
        }
    }))
    .map_err(to_tool_error)
}

fn commit_help_result(
    session: &crate::game::LoadedGameSession,
    current_revision: u64,
    player_input: Option<&str>,
    resolution: Option<&str>,
) -> Result<ToolResult, ToolError> {
    let missing = [
        ("player_input", player_input.is_none()),
        ("resolution", resolution.is_none()),
    ]
    .into_iter()
    .filter_map(|(field, missing)| missing.then_some(field))
    .collect::<Vec<_>>();
    ToolResult::json(&json!({
        "kind": "commit_help",
        "message": "game_commit_turn needs player_input and resolution before it can append a turn. expected_revision is inferred from the active save when omitted; state_patch may be omitted for an empty patch.",
        "game_id": session.game_id,
        "save_id": session.save_id,
        "current_revision": current_revision,
        "missing": missing,
        "accepted_aliases": {
            "player_input": ["player_action", "action", "input"],
            "resolution": ["narration", "response"],
            "state_patch": ["patch"]
        },
        "example": {
            "expected_revision": current_revision,
            "player_input": player_input.unwrap_or("<player action text>"),
            "resolution": resolution.unwrap_or("<player-facing consequence>"),
            "state_patch": {}
        }
    }))
    .map_err(to_tool_error)
}

fn is_missing_script_parameter(error: &impl std::fmt::Display) -> bool {
    error
        .to_string()
        .to_ascii_lowercase()
        .contains("missing parameter")
}

fn truncate_chars(raw: &str, max_bytes: usize) -> String {
    if raw.len() <= max_bytes {
        return raw.to_string();
    }
    let mut end = max_bytes;
    while !raw.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &raw[..end])
}

fn map_to_btree(map: &Map<String, Value>) -> BTreeMap<String, Value> {
    map.iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn to_tool_error(error: impl std::fmt::Display) -> ToolError {
    ToolError::execution_failed(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::game::{GameLaunchOptions, GameSession, load_game_session};

    fn context_for_example(name: &str) -> ToolContext {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/games")
            .join(name);
        let game_session = load_game_session(
            Path::new("."),
            GameLaunchOptions {
                game_or_path: Some(root),
                save: Some("default".to_string()),
                developer_mode: false,
                language: crate::game::GameLanguage::English,
            },
        )
        .expect("example game should load");
        let GameSession::Loaded(_) = game_session else {
            panic!("expected loaded game session");
        };
        let mut context = ToolContext::new(".");
        context.game_session = Some(game_session);
        context
    }

    #[tokio::test]
    async fn game_lookup_accepts_state_key_alias() {
        let context = context_for_example("reconciliation-demo");
        let result = GameLookupTool
            .execute(json!({"key": "world.flags"}), &context)
            .await
            .expect("state lookup should succeed");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["kind"], "state");
        assert_eq!(content["path"], "world.flags");
        assert_eq!(content["found"], true);
        assert_eq!(content["value"], json!({}));
    }

    #[tokio::test]
    async fn game_lookup_empty_input_returns_guidance() {
        let context = context_for_example("reconciliation-demo");
        let result = GameLookupTool
            .execute(json!({}), &context)
            .await
            .expect("empty lookup should return guidance");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["kind"], "lookup_help");
        assert!(content["state_examples"].is_array());
    }

    #[tokio::test]
    async fn game_run_driver_infers_single_function_and_action_alias() {
        let context = context_for_example("reconciliation-demo");
        let result = GameRunDriverTool
            .execute(
                json!({
                    "action": "I am sorry. I was scared, and I still care.",
                    "args": {"relationship_score": 1}
                }),
                &context,
            )
            .await
            .expect("single-function driver call should be repaired");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["result"]["relationship_score"], 4);
        assert_eq!(
            content["result"]["flags"],
            json!(["apology", "affection", "honest_admission"])
        );
    }

    #[tokio::test]
    async fn game_run_driver_scores_hostile_freeform_as_deflection() {
        let context = context_for_example("reconciliation-demo");
        let result = GameRunDriverTool
            .execute(
                json!({
                    "function": "score_action",
                    "args": {"player_action": "Fuck yourself then", "relationship_score": 0}
                }),
                &context,
            )
            .await
            .expect("hostile freeform should still score as gameplay");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["result"]["relationship_delta"], -2);
        assert_eq!(content["result"]["flags"], json!(["hostile_deflection"]));
    }

    #[tokio::test]
    async fn game_run_driver_missing_function_on_multi_function_driver_returns_guidance() {
        let context = context_for_example("thirteen-angry-man");
        let result = GameRunDriverTool
            .execute(json!({}), &context)
            .await
            .expect("multi-function missing function should return guidance");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["kind"], "driver_help");
        assert!(content["available_functions"].as_array().unwrap().len() >= 3);
    }

    #[tokio::test]
    async fn game_fact_check_blocks_configured_continuity_violation() {
        let context = context_for_example("reconciliation-demo");
        let result = GameFactCheckTool
            .execute(json!({"player_action": "其实我怀了你的孩子。"}), &context)
            .await
            .expect("fact check should run");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["kind"], "fact_check");
        assert_eq!(content["allowed"], false);
        assert_eq!(content["hard_block"], true);
        assert_eq!(content["flags"], json!(["player_pregnancy_impossible"]));
    }

    #[test]
    fn game_commit_turn_is_auto_approved_for_player_flow() {
        assert_eq!(
            GameCommitTurnTool.approval_requirement(),
            ApprovalRequirement::Auto
        );
    }

    #[tokio::test]
    async fn game_commit_turn_partial_input_returns_guidance_instead_of_error() {
        let context = context_for_example("reconciliation-demo");
        let session = loaded_game(&context).expect("loaded game");
        let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
            .expect("save should load");
        let current_revision = save
            .state
            .get("revision")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let result = GameCommitTurnTool
            .execute(
                json!({"player_action": "那你走吧，我懒得和你说了"}),
                &context,
            )
            .await
            .expect("partial commit should return guidance");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["kind"], "commit_help");
        assert_eq!(content["current_revision"], current_revision);
        assert!(
            content["missing"]
                .as_array()
                .unwrap()
                .contains(&json!("resolution"))
        );
    }

    #[tokio::test]
    async fn game_commit_turn_refuses_fact_blocked_resolution() {
        let context = context_for_example("reconciliation-demo");
        let result = GameCommitTurnTool
            .execute(
                json!({
                    "player_input": "其实我怀了你的孩子。",
                    "resolution": "你说自己怀了她的孩子，她停在雨里。",
                    "state_patch": {}
                }),
                &context,
            )
            .await
            .expect("fact-blocked commit should return a non-mutating result");
        let content: Value = serde_json::from_str(&result.content).expect("json result");

        assert_eq!(content["kind"], "commit_fact_block");
        assert_eq!(content["fact_check"]["hard_block"], true);
    }

    #[test]
    fn reconciliation_commit_normalization_updates_visible_state_for_violence() {
        let context = context_for_example("reconciliation-demo");
        let session = loaded_game(&context).expect("loaded game");
        let save = deepseek_game::save::load_save(&session.saves_root, &session.save_id)
            .expect("save should load");
        let mut patch = json!({
            "story": {
                "branch": "pressure_failure",
                "ended": true
            }
        });
        let driver_results = BTreeMap::new();
        let mut metadata = BTreeMap::new();
        metadata.insert("action_skill".to_string(), json!("move"));
        metadata.insert("action_subtype".to_string(), json!("violence"));

        normalize_game_state_patch(
            session,
            &save.state,
            "直接打她，打到她跪下",
            "她没有回头。",
            &driver_results,
            &metadata,
            &mut patch,
        );

        assert_eq!(
            patch.pointer("/ui/reactions/active"),
            Some(&json!("disgusted"))
        );
        assert_eq!(
            patch.pointer("/player/stats/relationship_score"),
            Some(&json!(-100))
        );
        assert_eq!(
            patch.pointer("/story/active_node"),
            Some(&json!("pressure_failure"))
        );
        assert_eq!(
            patch.pointer("/story/nodes/pressure_failure/status"),
            Some(&json!("failed"))
        );
        assert_eq!(patch.pointer("/world/flags/violence"), Some(&json!(true)));
    }
}
