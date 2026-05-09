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
        ToolResult::json(&json!({
            "game_id": session.game_id,
            "title": session.title,
            "save_id": session.save_id,
            "revision": session.revision,
            "driver_id": session.driver_id,
            "driver_version": session.locked_driver_version.as_deref().unwrap_or(&session.driver_requirement),
            "driver_resolved": session.driver_root.is_some(),
            "developer_mode": session.developer_mode,
            "warnings": session.warnings,
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
        ToolResult::json(&json!({
            "save_id": save.id,
            "revision": save.state.get("revision").and_then(Value::as_u64),
            "panels": panels,
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
impl ToolSpec for GameCommitTurnTool {
    fn name(&self) -> &'static str {
        "game_commit_turn"
    }

    fn description(&self) -> &'static str {
        "Atomically append one game turn and apply an RFC 7396 JSON Merge Patch to the active save."
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
                "resolution": {
                    "type": "string",
                    "description": "Player-facing turn resolution."
                },
                "state_patch": {
                    "type": "object",
                    "description": "RFC 7396 JSON Merge Patch to apply to STATE.json.",
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
            "required": ["expected_revision", "player_input", "resolution", "state_patch"],
            "additionalProperties": false
        })
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::WritesFiles, ToolCapability::Sandboxable]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Required
    }

    async fn execute(&self, input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let session = loaded_game(context)?;
        let expected_revision = input
            .get("expected_revision")
            .and_then(Value::as_u64)
            .ok_or_else(|| ToolError::missing_field("expected_revision"))?;
        let player_input = required_string(&input, "player_input")?;
        let resolution = required_string(&input, "resolution")?;
        let state_patch = input
            .get("state_patch")
            .cloned()
            .ok_or_else(|| ToolError::missing_field("state_patch"))?;
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

        let outcome = deepseek_game::save::commit_turn(
            session.saves_root.join(&session.save_id),
            CommitRequest {
                expected_revision,
                player_input,
                resolution,
                state_patch,
                driver_results,
                metadata,
            },
        )
        .map_err(to_tool_error)?;
        let panels = deepseek_game::render::render_panels(&outcome.state);
        ToolResult::json(&json!({
            "turn": outcome.turn,
            "state": outcome.state,
            "panels": panels,
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

fn required_string(input: &Value, key: &str) -> Result<String, ToolError> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ToolError::missing_field(key))
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
}
