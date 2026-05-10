//! `load_skill` tool — fetch a `SKILL.md` body and its companion-file
//! list into the model's context (#434).
//!
//! ## Why a tool when skills already surface in the system prompt?
//!
//! `prompts.rs::system_prompt_for_mode_with_context_and_skills` injects
//! a one-line listing of every available skill (name + description +
//! file path) so the model knows what's in the catalogue at the start
//! of every turn. The full body of each skill is *not* loaded — that
//! would blow the prompt budget the moment a user has half a dozen
//! skills installed.
//!
//! Two paths exist for the model to actually read a skill:
//!
//! 1. The existing progressive-disclosure pattern: model spots a
//!    skill in the catalogue, calls `read_file <path>` from the
//!    listing.
//! 2. (this tool) `load_skill name=<id>` — single call, name-based
//!    lookup, also enumerates the sibling files in the skill's
//!    directory so the model sees the companion resources without
//!    a separate `list_dir`.
//!
//! Both are valid; the tool is the higher-level affordance and
//! avoids the two-call dance for skills that ship with multiple
//! resource files.

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::skills::{Skill, SkillRegistry, discover_in_workspace, skills_directories};

use super::spec::{
    ApprovalRequirement, ToolCapability, ToolContext, ToolError, ToolResult, ToolSpec,
};

pub struct LoadSkillTool;

#[async_trait]
impl ToolSpec for LoadSkillTool {
    fn name(&self) -> &'static str {
        "load_skill"
    }

    fn description(&self) -> &'static str {
        "Load a skill (SKILL.md body + companion file list) into the next turn's context. \
         Use this when the user names a skill or the task clearly matches a skill listed in the system prompt's `## Skills` section. Accepts name, skill, skill_name, or id. Faster than read_file + list_dir."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Skill id (the `name` field from the SKILL.md frontmatter, also shown in the `## Skills` listing)."
                },
                "skill": {
                    "type": "string",
                    "description": "Alias for name, accepted for model repair."
                },
                "skill_name": {
                    "type": "string",
                    "description": "Alias for name, accepted for model repair."
                },
                "id": {
                    "type": "string",
                    "description": "Alias for name, accepted for model repair."
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::ReadOnly]
    }

    fn approval_requirement(&self) -> ApprovalRequirement {
        ApprovalRequirement::Auto
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    async fn execute(&self, input: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        // #432: walk every candidate skill directory (workspace
        // .agents/skills, skills, .opencode/skills, .claude/skills,
        // .cursor/skills, ~/.agents/skills, global default), merging with
        // first-wins precedence. The
        // tool's lookup mirrors what the system-prompt skills block
        // already lists, so the model never asks for a name it
        // can't find.
        let registry = discover_in_workspace(&context.workspace);
        let Some(name) = requested_skill_name(&input) else {
            return skill_help_result(context, &registry, None);
        };
        let skill = registry
            .get(&name)
            .cloned()
            .or_else(|| find_game_skill(context, &name));
        let Some(skill) = skill else {
            return skill_help_result(context, &registry, Some(&name));
        };

        let body = format_skill_body(&skill);
        Ok(ToolResult::success(body).with_metadata(json!({
            "skill_name": skill.name,
            "skill_path": skill.path.display().to_string(),
            "companion_files": collect_companion_files(&skill)
                .into_iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<String>>(),
        })))
    }
}

fn requested_skill_name(input: &Value) -> Option<String> {
    ["name", "skill", "skill_name", "id"]
        .into_iter()
        .find_map(|key| {
            input
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

fn find_game_skill(context: &ToolContext, name: &str) -> Option<Skill> {
    let game_session = context.game_session.as_ref()?;
    for dir in game_session.skill_directories() {
        let registry = SkillRegistry::discover(&dir);
        if let Some(skill) = registry.get(name) {
            return Some(skill.clone());
        }
    }
    None
}

fn available_skill_names(context: &ToolContext, registry: &SkillRegistry) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    names.extend(registry.list().iter().map(|skill| skill.name.clone()));
    if let Some(game_session) = context.game_session.as_ref() {
        for dir in game_session.skill_directories() {
            let game_registry = SkillRegistry::discover(&dir);
            names.extend(game_registry.list().iter().map(|skill| skill.name.clone()));
        }
    }
    names.into_iter().collect()
}

fn skill_help_result(
    context: &ToolContext,
    registry: &SkillRegistry,
    missing: Option<&str>,
) -> Result<ToolResult, ToolError> {
    let available = available_skill_names(context, registry);
    let dirs: Vec<String> = skills_directories(&context.workspace)
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let message = match missing {
        Some(name) if available.is_empty() => {
            format!("skill `{name}` not found; no skills are currently installed")
        }
        Some(name) => format!("skill `{name}` not found"),
        None if available.is_empty() => {
            "load_skill needs a skill name; no skills are currently installed".to_string()
        }
        None => "load_skill needs a skill name".to_string(),
    };
    ToolResult::json(&json!({
        "kind": "skill_help",
        "message": message,
        "accepted_fields": ["name", "skill", "skill_name", "id"],
        "examples": [
            {"name": "game-action-router"},
            {"skill": "reconciliation"},
            {"skill_name": "game-storytelling-director"}
        ],
        "available_skills": available,
        "searched_dirs": dirs,
    }))
    .map_err(|err| ToolError::execution_failed(err.to_string()))
}

/// Render the skill body the model will see. Includes the description
/// up top so a single tool result is self-contained — no need to
/// cross-reference the system-prompt catalogue. Companion-file paths
/// land at the bottom under a clearly-named heading so the model can
/// open them with `read_file` if they're relevant to the task.
fn format_skill_body(skill: &Skill) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Skill: {}\n\n", skill.name));
    if !skill.description.trim().is_empty() {
        out.push_str(&format!("> {}\n\n", skill.description.trim()));
    }
    out.push_str(&format!("Source: `{}`\n\n", skill.path.display()));
    out.push_str("## SKILL.md\n\n");
    out.push_str(skill.body.trim());
    out.push('\n');

    let companions = collect_companion_files(skill);
    if !companions.is_empty() {
        out.push_str("\n## Companion files\n\n");
        out.push_str(
            "Sibling files in the skill directory. Use `read_file` to open them when the task requires.\n\n",
        );
        for path in &companions {
            out.push_str(&format!("- `{}`\n", path.display()));
        }
    }
    out
}

/// List sibling files of `SKILL.md` in the skill's own directory.
/// Skips the `SKILL.md` itself and any nested directories so the
/// listing stays focused on at-hand resources. Sorted lexically for
/// deterministic output (matters for transcript diffing in tests).
fn collect_companion_files(skill: &Skill) -> Vec<std::path::PathBuf> {
    let Some(dir) = skill.path.parent() else {
        return Vec::new();
    };
    let mut entries: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                let is_file = entry.file_type().is_ok_and(|ft| ft.is_file());
                let is_skill_md = path.file_name().and_then(|s| s.to_str()) == Some("SKILL.md");
                if is_file && !is_skill_md {
                    Some(path)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::SkillRegistry;
    use std::fs;
    use tempfile::tempdir;

    fn write_skill(dir: &std::path::Path, name: &str, description: &str, body: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {description}\n---\n{body}\n"),
        )
        .unwrap();
    }

    #[test]
    fn load_skill_returns_skill_body_with_description_header() {
        let tmp = tempdir().unwrap();
        write_skill(
            tmp.path(),
            "review-pr",
            "Run a focused PR review",
            "# Steps\n1. Read the diff.\n2. Comment.\n",
        );
        let skill = SkillRegistry::discover(tmp.path())
            .get("review-pr")
            .unwrap()
            .clone();
        let body = format_skill_body(&skill);
        assert!(body.contains("# Skill: review-pr"));
        assert!(body.contains("Run a focused PR review"));
        assert!(body.contains("# Steps"));
        assert!(body.contains("Read the diff."));
    }

    #[test]
    fn collect_companion_files_lists_siblings_excluding_skill_md() {
        let tmp = tempdir().unwrap();
        let skill_dir = tmp.path().join("rich-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: rich-skill\ndescription: x\n---\nbody\n",
        )
        .unwrap();
        fs::write(skill_dir.join("script.py"), "print('hi')").unwrap();
        fs::write(skill_dir.join("data.json"), "{}").unwrap();
        // Nested directory — skipped by collect_companion_files.
        fs::create_dir_all(skill_dir.join("subdir")).unwrap();

        let registry = SkillRegistry::discover(tmp.path());
        let skill = registry.get("rich-skill").unwrap();
        let files = collect_companion_files(skill);
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str().map(str::to_string)))
            .collect();
        assert_eq!(
            names,
            vec!["data.json".to_string(), "script.py".to_string()]
        );
    }

    #[test]
    fn collect_companion_files_returns_empty_for_solo_skill() {
        let tmp = tempdir().unwrap();
        write_skill(tmp.path(), "solo", "Just a skill", "body");
        let registry = SkillRegistry::discover(tmp.path());
        let skill = registry.get("solo").unwrap();
        assert!(collect_companion_files(skill).is_empty());
    }

    #[test]
    fn format_skill_body_emits_companion_files_section_when_present() {
        let tmp = tempdir().unwrap();
        let skill_dir = tmp.path().join("skill-with-friends");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: skill-with-friends\ndescription: x\n---\nbody\n",
        )
        .unwrap();
        fs::write(skill_dir.join("helper.sh"), "#!/bin/sh\necho hi").unwrap();

        let registry = SkillRegistry::discover(tmp.path());
        let skill = registry.get("skill-with-friends").unwrap();
        let body = format_skill_body(skill);
        assert!(body.contains("## Companion files"));
        assert!(body.contains("helper.sh"));
    }

    #[test]
    fn format_skill_body_skips_companion_section_when_solo() {
        let tmp = tempdir().unwrap();
        write_skill(tmp.path(), "solo", "x", "body");
        let registry = SkillRegistry::discover(tmp.path());
        let skill = registry.get("solo").unwrap();
        let body = format_skill_body(skill);
        assert!(
            !body.contains("## Companion files"),
            "solo skills shouldn't emit an empty Companion files section"
        );
    }

    #[tokio::test]
    async fn execute_finds_skills_in_opencode_dir_via_workspace_discovery() {
        let tmp = tempdir().unwrap();
        let workspace = tmp.path().to_path_buf();
        // Skill installed under workspace `.opencode/skills` (#432).
        let opencode_dir = workspace.join(".opencode").join("skills");
        std::fs::create_dir_all(&opencode_dir).unwrap();
        write_skill(
            &opencode_dir,
            "from-opencode",
            "Skill installed under .opencode/skills",
            "Body content marker.",
        );

        let mut context = ToolContext::new(workspace);
        // The skill tool reads $HOME for the global default; pin it to a
        // tempdir so the test is hermetic regardless of the host's
        // ~/.deepseek/skills.
        context.workspace = tmp.path().to_path_buf();

        let tool = LoadSkillTool;
        let result = tool
            .execute(json!({"name": "from-opencode"}), &context)
            .await
            .expect("load_skill should succeed");
        assert!(result.success);
        assert!(
            result.content.contains("# Skill: from-opencode"),
            "body header missing: {}",
            &result.content
        );
        assert!(result.content.contains("Body content marker."));

        let metadata = result.metadata.expect("metadata stamped");
        assert_eq!(
            metadata
                .get("skill_name")
                .and_then(serde_json::Value::as_str),
            Some("from-opencode")
        );
        let path_str = metadata
            .get("skill_path")
            .and_then(serde_json::Value::as_str)
            .expect("skill_path stamped");
        assert!(
            path_str.contains(".opencode"),
            "skill_path should point at the .opencode dir: {path_str}"
        );
    }

    #[tokio::test]
    async fn execute_finds_skills_from_active_game_session() {
        let tmp = tempdir().unwrap();
        let game_root = tmp.path().join("game");
        let game_skills = game_root.join("skills");
        write_skill(
            &game_skills,
            "game-only",
            "Only available through the active game",
            "Game skill body marker.",
        );

        let mut context = ToolContext::new(tmp.path().join("workspace"));
        context.game_session = Some(crate::game::GameSession::Loaded(
            crate::game::LoadedGameSession {
                game_root,
                saves_root: tmp.path().join("saves"),
                driver_root: None,
                game_id: "game".to_string(),
                title: "Game".to_string(),
                save_id: "default".to_string(),
                revision: 0,
                driver_id: "driver".to_string(),
                driver_requirement: "0.1.0".to_string(),
                locked_driver_version: Some("0.1.0".to_string()),
                panels: Vec::new(),
                view: deepseek_game::render::render_view_snapshot(&json!({
                    "revision": 0,
                    "scene": {"location": "Test", "summary": "Test scene."}
                })),
                action_skills: Vec::new(),
                skills: Vec::new(),
                warnings: Vec::new(),
                developer_mode: false,
                language: crate::game::GameLanguage::English,
            },
        ));

        let tool = LoadSkillTool;
        let result = tool
            .execute(json!({"name": "game-only"}), &context)
            .await
            .expect("load_skill should find active game skills");
        assert!(result.success);
        assert!(result.content.contains("Game skill body marker."));
    }

    #[tokio::test]
    async fn execute_accepts_skill_alias() {
        let tmp = tempdir().unwrap();
        let workspace = tmp.path().to_path_buf();
        write_skill(
            &workspace.join(".agents").join("skills"),
            "alias-skill",
            "x",
            "Alias body marker.",
        );

        let context = ToolContext::new(workspace);
        let tool = LoadSkillTool;
        let result = tool
            .execute(json!({"skill": "alias-skill"}), &context)
            .await
            .expect("load_skill should accept skill alias");
        assert!(result.success);
        assert!(result.content.contains("Alias body marker."));
    }

    #[tokio::test]
    async fn execute_returns_guidance_for_missing_name() {
        let tmp = tempdir().unwrap();
        let workspace = tmp.path().to_path_buf();
        write_skill(
            &workspace.join(".agents").join("skills"),
            "real-one",
            "x",
            "body",
        );

        let context = ToolContext::new(workspace);
        let tool = LoadSkillTool;
        let result = tool
            .execute(json!({}), &context)
            .await
            .expect("empty load_skill should return guidance");
        let content: Value = serde_json::from_str(&result.content).expect("json result");
        assert_eq!(content["kind"], "skill_help");
        assert!(
            content["available_skills"]
                .as_array()
                .unwrap()
                .contains(&json!("real-one"))
        );
    }

    #[tokio::test]
    async fn execute_returns_helpful_guidance_for_unknown_skill() {
        let tmp = tempdir().unwrap();
        let workspace = tmp.path().to_path_buf();
        // One real skill so the available list is non-empty.
        write_skill(
            &workspace.join(".agents").join("skills"),
            "real-one",
            "x",
            "body",
        );

        let context = ToolContext::new(workspace);
        let tool = LoadSkillTool;
        let result = tool
            .execute(json!({"name": "imaginary"}), &context)
            .await
            .expect("unknown skill should return guidance");
        let msg = result.content;
        assert!(
            msg.contains("imaginary") && msg.contains("real-one"),
            "error must name the missing skill and list available ones: {msg}"
        );
    }
}
