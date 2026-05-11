use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::paths;
use crate::{GameError, Result};

pub const STATE_FILE: &str = "STATE.json";
pub const TURN_LOG_FILE: &str = "TURN_LOG.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveState {
    #[serde(flatten)]
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriverLock {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnRecord {
    pub turn_id: String,
    pub revision_before: u64,
    pub revision_after: u64,
    pub player_input: String,
    pub resolution: String,
    pub state_patch: Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub driver_results: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedSave {
    pub id: String,
    pub root: PathBuf,
    pub state: Value,
    pub turns: Vec<TurnRecord>,
    pub summary: Option<String>,
    pub agents: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitRequest {
    pub expected_revision: u64,
    pub player_input: String,
    pub resolution: String,
    pub state_patch: Value,
    #[serde(default)]
    pub driver_results: BTreeMap<String, Value>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommitOutcome {
    pub turn: TurnRecord,
    pub state: Value,
}

pub fn load_save(saves_root: impl AsRef<Path>, save_id: &str) -> Result<LoadedSave> {
    validate_save_id(save_id)?;
    let saves_root = paths::canonicalize_root(saves_root, "saves root")?;
    let root = paths::resolve_under(&saves_root, save_id, "save id")?;
    let state = read_state(root.join(STATE_FILE))?;
    validate_state(&state)?;
    let turns = read_turn_log(root.join(TURN_LOG_FILE))?;
    let summary = read_optional_string(&root.join("SUMMARY.md"))?;
    let agents = read_optional_json(&root.join("AGENTS.json"))?;
    Ok(LoadedSave {
        id: save_id.to_string(),
        root,
        state,
        turns,
        summary,
        agents,
    })
}

pub fn create_save_from_template(
    saves_root: impl AsRef<Path>,
    save_id: &str,
    template_save_id: &str,
) -> Result<()> {
    create_save_from_template_root(
        saves_root.as_ref(),
        save_id,
        saves_root.as_ref(),
        template_save_id,
    )
}

pub fn create_save_from_template_root(
    saves_root: impl AsRef<Path>,
    save_id: &str,
    template_saves_root: impl AsRef<Path>,
    template_save_id: &str,
) -> Result<()> {
    validate_save_id(save_id)?;
    validate_save_id(template_save_id)?;
    let saves_root = paths::canonicalize_root(saves_root, "saves root")?;
    let template_saves_root = paths::canonicalize_root(template_saves_root, "save template root")?;
    let target_root = paths::resolve_under(&saves_root, save_id, "save id")?;
    let template = load_save(&template_saves_root, template_save_id)?;

    prepare_new_save_dir(&target_root)?;
    write_json_atomic(&target_root.join(STATE_FILE), &template.state)?;
    write_bytes_atomic(&target_root.join(TURN_LOG_FILE), b"")?;
    if let Some(summary) = template.summary {
        write_bytes_atomic(&target_root.join("SUMMARY.md"), summary.as_bytes())?;
    }
    if let Some(mut agents) = template.agents {
        rewrite_template_save_references(&mut agents, save_id, template_save_id);
        write_json_atomic(&target_root.join("AGENTS.json"), &agents)?;
    }
    Ok(())
}

fn rewrite_template_save_references(value: &mut Value, save_id: &str, template_save_id: &str) {
    match value {
        Value::String(text) => {
            for (from, to) in [
                (
                    format!("saves/{template_save_id}/"),
                    format!("saves/{save_id}/"),
                ),
                (
                    format!("save_templates/{template_save_id}/"),
                    format!("saves/{save_id}/"),
                ),
            ] {
                if text.contains(&from) {
                    *text = text.replace(&from, &to);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_template_save_references(value, save_id, template_save_id);
            }
        }
        Value::Object(map) => {
            for value in map.values_mut() {
                rewrite_template_save_references(value, save_id, template_save_id);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn prepare_new_save_dir(target_root: &Path) -> Result<()> {
    match std::fs::create_dir(target_root) {
        Ok(()) => Ok(()),
        Err(source)
            if source.kind() == std::io::ErrorKind::AlreadyExists && target_root.is_dir() =>
        {
            let mut entries = std::fs::read_dir(target_root).map_err(|source| GameError::Read {
                path: target_root.to_path_buf(),
                source,
            })?;
            if entries.next().is_none() {
                Ok(())
            } else {
                Err(GameError::Write {
                    path: target_root.to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "save directory is not empty",
                    ),
                })
            }
        }
        Err(source) => Err(GameError::Write {
            path: target_root.to_path_buf(),
            source,
        }),
    }
}

pub fn read_state(path: impl AsRef<Path>) -> Result<Value> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path).map_err(|source| GameError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|source| GameError::Json {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_turn_log(path: impl AsRef<Path>) -> Result<Vec<TurnRecord>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path).map_err(|source| GameError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<TurnRecord>(line).map_err(|source| GameError::Json {
                path: path.to_path_buf(),
                source,
            })
        })
        .collect()
}

pub fn commit_turn(save_root: impl AsRef<Path>, request: CommitRequest) -> Result<CommitOutcome> {
    let save_root = paths::canonicalize_root(save_root, "save root")?;
    let state_path = save_root.join(STATE_FILE);
    let turn_log_path = save_root.join(TURN_LOG_FILE);
    let mut state = read_state(&state_path)?;
    validate_state(&state)?;
    let revision_before = revision(&state)?;
    if revision_before != request.expected_revision {
        return Err(GameError::RevisionConflict {
            expected: request.expected_revision,
            actual: revision_before,
        });
    }

    merge_patch(&mut state, &request.state_patch);
    set_revision(&mut state, revision_before + 1)?;
    validate_state(&state)?;

    let turn_count = read_turn_log(&turn_log_path)?.len();
    let turn = TurnRecord {
        turn_id: format!("{:06}", turn_count + 1),
        revision_before,
        revision_after: revision_before + 1,
        player_input: request.player_input,
        resolution: request.resolution,
        state_patch: request.state_patch,
        driver_results: request.driver_results,
        metadata: request.metadata,
        created_at: Utc::now(),
    };

    let existing_log = if turn_log_path.exists() {
        std::fs::read_to_string(&turn_log_path).map_err(|source| GameError::Read {
            path: turn_log_path.clone(),
            source,
        })?
    } else {
        String::new()
    };
    let mut next_log = existing_log;
    if !next_log.is_empty() && !next_log.ends_with('\n') {
        next_log.push('\n');
    }
    let turn_line = serde_json::to_string(&turn).map_err(|source| GameError::Json {
        path: turn_log_path.clone(),
        source,
    })?;
    next_log.push_str(&turn_line);
    next_log.push('\n');

    write_json_atomic(&state_path, &state)?;
    write_bytes_atomic(&turn_log_path, next_log.as_bytes())?;

    Ok(CommitOutcome { turn, state })
}

pub fn validate_state(state: &Value) -> Result<()> {
    let object = state.as_object().ok_or_else(|| {
        GameError::SaveValidation("STATE.json root must be an object".to_string())
    })?;
    required_u64(object, "schema_version")?;
    required_u64(object, "revision")?;
    let driver = object
        .get("driver")
        .and_then(Value::as_object)
        .ok_or_else(|| GameError::SaveValidation("driver must be an object".to_string()))?;
    required_string(driver, "id")?;
    required_string(driver, "version")?;
    Ok(())
}

pub fn revision(state: &Value) -> Result<u64> {
    state
        .get("revision")
        .and_then(Value::as_u64)
        .ok_or_else(|| GameError::SaveValidation("revision must be a number".to_string()))
}

pub fn driver_lock(state: &Value) -> Result<DriverLock> {
    let driver = state
        .get("driver")
        .and_then(Value::as_object)
        .ok_or_else(|| GameError::SaveValidation("driver must be an object".to_string()))?;
    Ok(DriverLock {
        id: required_string(driver, "id")?.to_string(),
        version: required_string(driver, "version")?.to_string(),
    })
}

pub fn merge_patch(target: &mut Value, patch: &Value) {
    let Some(patch_object) = patch.as_object() else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let target_object = target.as_object_mut().expect("target was forced to object");
    for (key, value) in patch_object {
        if value.is_null() {
            target_object.remove(key);
        } else if let Some(existing) = target_object.get_mut(key) {
            merge_patch(existing, value);
        } else {
            target_object.insert(key.clone(), value.clone());
        }
    }
}

fn set_revision(state: &mut Value, revision: u64) -> Result<()> {
    let object = state.as_object_mut().ok_or_else(|| {
        GameError::SaveValidation("STATE.json root must be an object".to_string())
    })?;
    object.insert("revision".to_string(), Value::from(revision));
    Ok(())
}

fn required_u64<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a Value> {
    let value = object
        .get(key)
        .ok_or_else(|| GameError::SaveValidation(format!("{key} is required")))?;
    if value.as_u64().is_none() {
        return Err(GameError::SaveValidation(format!(
            "{key} must be an unsigned integer"
        )));
    }
    Ok(value)
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| GameError::SaveValidation(format!("{key} must be a non-empty string")))
}

fn read_optional_string(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(value) => Ok(Some(value)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(GameError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn read_optional_json(path: &Path) -> Result<Option<Value>> {
    let Some(raw) = read_optional_string(path)? else {
        return Ok(None);
    };
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|source| GameError::Json {
            path: path.to_path_buf(),
            source,
        })
}

fn validate_save_id(save_id: &str) -> Result<()> {
    if !save_id.is_empty()
        && save_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        Ok(())
    } else {
        Err(GameError::InvalidPath {
            label: "save id".to_string(),
            path: PathBuf::from(save_id),
        })
    }
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|source| GameError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    write_bytes_atomic(path, &bytes)
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| GameError::InvalidPath {
        label: "atomic write".to_string(),
        path: path.to_path_buf(),
    })?;
    std::fs::create_dir_all(parent).map_err(|source| GameError::Write {
        path: parent.to_path_buf(),
        source,
    })?;
    let tmp = parent.join(format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("game-save"),
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|source| GameError::Write {
                path: tmp.clone(),
                source,
            })?;
        file.write_all(bytes).map_err(|source| GameError::Write {
            path: tmp.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| GameError::Write {
            path: tmp.clone(),
            source,
        })?;
    }
    std::fs::rename(&tmp, path).map_err(|source| GameError::Write {
        path: path.to_path_buf(),
        source,
    })
}
