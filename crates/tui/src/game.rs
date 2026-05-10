use std::path::{Path, PathBuf};

use anyhow::Result;
use deepseek_game::agents::{AgentPack, build_agent_packs};
use deepseek_game::driver::{DriverResolver, LoadedDriver};
use deepseek_game::interaction::{ActionSkill, build_playbook, format_playbook};
use deepseek_game::manifest::LoadedGame;
use deepseek_game::render::{GameViewSnapshot, RenderPanel, render_panels, render_view_snapshot};
use deepseek_game::save::{LoadedSave, driver_lock, load_save};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct GameLaunchOptions {
    pub game_or_path: Option<PathBuf>,
    pub save: Option<String>,
    pub developer_mode: bool,
    pub language: GameLanguage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameLanguage {
    #[default]
    English,
    Chinese,
}

impl GameLanguage {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" | "eng" | "english" => Some(Self::English),
            "zh" | "cn" | "chinese" | "中文" | "汉语" | "漢語" => Some(Self::Chinese),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Chinese => "Chinese",
        }
    }

    pub fn is_chinese(self) -> bool {
        matches!(self, Self::Chinese)
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum GameSession {
    Loaded(LoadedGameSession),
    Notice(GameSessionNotice),
}

#[derive(Debug, Clone)]
pub struct LoadedGameSession {
    pub game_root: PathBuf,
    pub saves_root: PathBuf,
    pub driver_root: Option<PathBuf>,
    pub game_id: String,
    pub title: String,
    pub save_id: String,
    pub revision: u64,
    pub driver_id: String,
    pub driver_requirement: String,
    pub locked_driver_version: Option<String>,
    pub panels: Vec<RenderPanel>,
    pub view: GameViewSnapshot,
    pub action_skills: Vec<ActionSkill>,
    pub skills: Vec<GameSkillCatalogEntry>,
    pub warnings: Vec<String>,
    pub developer_mode: bool,
    pub language: GameLanguage,
}

#[derive(Debug, Clone)]
pub struct GameSkillCatalogEntry {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GameAgentPackSummary {
    pub role: String,
    pub output_contract: String,
    pub allowed_files: Vec<String>,
    pub assigned_skills: Vec<String>,
    pub callable_driver_functions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameActionSkillShortcut {
    pub trigger: String,
    pub skill_name: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct GameSessionNotice {
    pub message: String,
    pub developer_mode: bool,
    pub language: GameLanguage,
}

impl GameSession {
    pub fn developer_mode(&self) -> bool {
        match self {
            Self::Loaded(session) => session.developer_mode,
            Self::Notice(notice) => notice.developer_mode,
        }
    }

    pub fn set_developer_mode(&mut self, enabled: bool) {
        match self {
            Self::Loaded(session) => session.developer_mode = enabled,
            Self::Notice(notice) => notice.developer_mode = enabled,
        }
    }

    pub fn language(&self) -> GameLanguage {
        match self {
            Self::Loaded(session) => session.language,
            Self::Notice(notice) => notice.language,
        }
    }

    pub fn status_label(&self) -> String {
        match self {
            Self::Loaded(session) => {
                format!("Game Console: {} / {}", session.title, session.save_id)
            }
            Self::Notice(notice) => format!("Game Console: {}", notice.message),
        }
    }

    pub fn transcript_intro(&self) -> String {
        match self {
            Self::Loaded(session) => session.transcript_intro(),
            Self::Notice(notice) => {
                let mut lines = vec![
                    "Game Console".to_string(),
                    String::new(),
                    notice.message.clone(),
                    format!("Selected language: {}", notice.language.label()),
                ];
                if notice.developer_mode {
                    lines.push("Developer mode: on".to_string());
                }
                lines.join("\n")
            }
        }
    }

    pub fn status_report(&self) -> String {
        match self {
            Self::Loaded(session) => session.status_report(),
            Self::Notice(notice) => {
                let mut lines = vec![
                    "Game Console status".to_string(),
                    String::new(),
                    notice.message.clone(),
                ];
                lines.push(format!(
                    "Developer mode: {}",
                    if notice.developer_mode { "on" } else { "off" }
                ));
                lines.push(format!("Selected language: {}", notice.language.label()));
                lines.join("\n")
            }
        }
    }

    pub fn choices_report(&self) -> Result<String> {
        match self {
            Self::Loaded(session) => session.choices_report(),
            Self::Notice(notice) => Ok(format!(
                "No loaded Game Console session: {}",
                notice.message
            )),
        }
    }

    pub fn rules_report(&self) -> Result<String> {
        match self {
            Self::Loaded(session) => session.rules_report(),
            Self::Notice(notice) => Ok(format!(
                "No loaded Game Console session: {}\n\nUse /play <game-or-path> to start a game.",
                notice.message
            )),
        }
    }

    pub fn skill_directories(&self) -> Vec<PathBuf> {
        match self {
            Self::Loaded(session) => session.skill_directories(),
            Self::Notice(_) => Vec::new(),
        }
    }

    pub fn action_skill_shortcuts(&self) -> Vec<GameActionSkillShortcut> {
        match self {
            Self::Loaded(session) => session.action_skill_shortcuts(),
            Self::Notice(_) => Vec::new(),
        }
    }

    pub fn resolve_action_skill_name(&self, name: &str) -> Option<String> {
        match self {
            Self::Loaded(session) => session.resolve_action_skill_name(name),
            Self::Notice(_) => None,
        }
    }

    pub fn refresh_from_game_tool_value(&mut self, value: &Value) -> bool {
        match self {
            Self::Loaded(session) => session.refresh_from_game_tool_value(value),
            Self::Notice(_) => false,
        }
    }
}

impl LoadedGameSession {
    pub(crate) fn agent_packs(&self) -> Result<Vec<AgentPack>> {
        let driver_root = self
            .driver_root
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("active game driver is not resolved"))?;
        let driver = deepseek_game::driver::load_driver(driver_root)?;
        let save = load_save(&self.saves_root, &self.save_id)?;
        Ok(build_agent_packs(&driver.manifest, &save.state))
    }

    pub(crate) fn agent_pack_summaries(&self) -> Result<Vec<GameAgentPackSummary>> {
        Ok(self
            .agent_packs()?
            .iter()
            .map(GameAgentPackSummary::from_agent_pack)
            .collect())
    }

    fn status_report(&self) -> String {
        let mut lines = vec![
            "Game Console status".to_string(),
            String::new(),
            format!("Game: {} ({})", self.title, self.game_id),
            format!("Save: {} @ revision {}", self.save_id, self.revision),
            format!("Selected language: {}", self.language.label()),
            format!(
                "Driver: {} {}",
                self.driver_id,
                self.locked_driver_version
                    .as_deref()
                    .unwrap_or(&self.driver_requirement)
            ),
            format!(
                "Developer mode: {}",
                if self.developer_mode { "on" } else { "off" }
            ),
        ];

        if self.developer_mode {
            lines.push(format!("Game root: {}", self.game_root.display()));
            lines.push(format!("Saves root: {}", self.saves_root.display()));
            if let Some(driver_root) = &self.driver_root {
                lines.push(format!("Driver root: {}", driver_root.display()));
            }
        }
        self.append_agent_pack_guidance(&mut lines);
        if !self.warnings.is_empty() {
            lines.push(String::new());
            lines.push("Warnings:".to_string());
            lines.extend(self.warnings.iter().map(|warning| format!("- {warning}")));
        }
        if !self.skills.is_empty() {
            lines.push(String::new());
            lines.push(format!("Loadable game skills: {}", self.skills.len()));
        }
        lines.join("\n")
    }

    fn transcript_intro(&self) -> String {
        let mut lines = vec![
            "Game Console".to_string(),
            String::new(),
            format!("{} ({})", self.title, self.game_id),
            format!("Save: {} @ revision {}", self.save_id, self.revision),
            format!("Selected language: {}", self.language.label()),
            format!(
                "Driver: {} {}",
                self.driver_id,
                self.locked_driver_version
                    .as_deref()
                    .unwrap_or(&self.driver_requirement)
            ),
        ];
        append_player_rules(&mut lines, self.language);

        if self.developer_mode {
            lines.push(format!("Game root: {}", self.game_root.display()));
            lines.push(format!("Saves root: {}", self.saves_root.display()));
            if let Some(driver_root) = &self.driver_root {
                lines.push(format!("Driver root: {}", driver_root.display()));
            }
        }
        if !self.warnings.is_empty() {
            lines.push(String::new());
            lines.push("Warnings:".to_string());
            lines.extend(self.warnings.iter().map(|warning| format!("- {warning}")));
        }
        if !self.skills.is_empty() {
            lines.push(String::new());
            lines.push("Loadable game skills:".to_string());
            for skill in &self.skills {
                let description = if skill.description.trim().is_empty() {
                    String::new()
                } else {
                    format!(" - {}", skill.description.trim())
                };
                let path = if self.developer_mode {
                    format!(" @ {}", skill.path.display())
                } else {
                    String::new()
                };
                lines.push(format!(
                    "- {} ({}){}{}",
                    skill.name, skill.source, description, path
                ));
            }
            lines.push("Use load_skill when a turn needs that rule pack.".to_string());
        }
        self.append_agent_pack_guidance(&mut lines);
        if !self.panels.is_empty() {
            lines.push(String::new());
            lines.push("Panels:".to_string());
            for panel in &self.panels {
                lines.push(format!("## {}", panel.title));
                if !panel.body.is_empty() {
                    lines.push(panel.body.clone());
                }
            }
        }
        lines.join("\n")
    }

    fn append_agent_pack_guidance(&self, lines: &mut Vec<String>) {
        match self.agent_pack_summaries() {
            Ok(packs) if !packs.is_empty() => {
                lines.push(String::new());
                lines.push("Scoped game sub-agents:".to_string());
                for pack in packs {
                    lines.push(format!("- {}: {}", pack.role, pack.output_contract));
                    if !pack.assigned_skills.is_empty() {
                        lines.push(format!("  skills: {}", pack.assigned_skills.join(", ")));
                    }
                    if !pack.allowed_files.is_empty() {
                        lines.push(format!("  files: {}", pack.allowed_files.join(", ")));
                    }
                }
                lines.push("Use game_agent_list to reuse live processors. For active NPC dialogue, reactions, memories, or stateful character behavior, call game_agent_spawn with pack/role set to the matching scoped pack, wait with game_agent_wait or game_agent_result, and treat the result as a proposal only. Final narration and save writes stay in the main session through game_commit_turn.".to_string());
            }
            Err(err) if self.developer_mode => {
                lines.push(String::new());
                lines.push(format!("Scoped game sub-agents: unavailable ({err})"));
            }
            _ => {}
        }
    }

    fn choices_report(&self) -> Result<String> {
        let save = load_save(&self.saves_root, &self.save_id)?;
        Ok(format_playbook(&build_playbook(&save.state)))
    }

    fn rules_report(&self) -> Result<String> {
        let save = load_save(&self.saves_root, &self.save_id)?;
        let playbook = build_playbook(&save.state);
        let mut lines = vec![
            "Rule Repeat".to_string(),
            String::new(),
            format!("{} ({})", self.title, self.game_id),
            format!("Save: {} @ revision {}", self.save_id, self.revision),
            format!("Selected language: {}", self.language.label()),
        ];
        append_player_rules(&mut lines, self.language);
        lines.push(String::new());
        lines.push("Current playbook".to_string());
        lines.push(format_playbook(&playbook));
        Ok(lines.join("\n"))
    }

    fn skill_directories(&self) -> Vec<PathBuf> {
        let mut dirs = vec![
            self.game_root.join("skills"),
            self.saves_root.join(&self.save_id).join("skills"),
        ];
        if let Some(driver_root) = &self.driver_root {
            dirs.push(driver_root.join("skills"));
        }
        dirs.into_iter().filter(|dir| dir.is_dir()).collect()
    }

    fn action_skill_shortcuts(&self) -> Vec<GameActionSkillShortcut> {
        let mut shortcuts = Vec::new();
        for action in &self.action_skills {
            let primary = action.id.trim();
            if primary.is_empty() || action.skill.trim().is_empty() {
                continue;
            }
            let label = action.label.trim();
            let label = if label.is_empty() { primary } else { label };
            if shortcuts
                .iter()
                .any(|shortcut: &GameActionSkillShortcut| shortcut.trigger == primary)
            {
                continue;
            }
            shortcuts.push(GameActionSkillShortcut {
                trigger: primary.to_string(),
                skill_name: action.skill.trim().to_string(),
                label: label.to_string(),
                description: action.description.trim().to_string(),
            });
        }
        shortcuts
    }

    fn resolve_action_skill_name(&self, name: &str) -> Option<String> {
        let requested = normalized_action_skill_name(name);
        if requested.is_empty() {
            return None;
        }
        self.action_skills
            .iter()
            .find(|action| {
                normalized_action_skill_name(&action.skill) == requested
                    || normalized_action_skill_name(&action.id) == requested
                    || normalized_action_skill_name(&action.label) == requested
                    || action
                        .aliases
                        .iter()
                        .any(|alias| normalized_action_skill_name(alias) == requested)
            })
            .map(|action| action.skill.clone())
    }

    fn refresh_from_game_tool_value(&mut self, value: &Value) -> bool {
        let mut refreshed = false;

        if let Some(state) = value.get("state") {
            self.revision = state
                .get("revision")
                .and_then(Value::as_u64)
                .unwrap_or(self.revision);
            self.panels = render_panels(state);
            self.view = render_view_snapshot(state);
            self.action_skills = build_playbook(state).action_skills;
            return true;
        }

        if let Some(revision) = value.get("revision").and_then(Value::as_u64) {
            self.revision = revision;
            refreshed = true;
        }

        if let Some(panels) = value.get("panels")
            && let Ok(panels) = serde_json::from_value::<Vec<RenderPanel>>(panels.clone())
        {
            self.panels = panels;
            refreshed = true;
        }

        if let Some(view) = value.get("view")
            && let Ok(view) = serde_json::from_value::<GameViewSnapshot>(view.clone())
        {
            self.view = view;
            refreshed = true;
        }

        refreshed
    }
}

impl GameAgentPackSummary {
    fn from_agent_pack(pack: &AgentPack) -> Self {
        Self {
            role: pack.role.clone(),
            output_contract: pack.output_contract.clone(),
            allowed_files: path_list(&pack.allowed_files),
            assigned_skills: path_list(&pack.assigned_skills),
            callable_driver_functions: pack.callable_driver_functions.clone(),
        }
    }
}

fn path_list(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

fn normalized_action_skill_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn append_player_rules(lines: &mut Vec<String>, language: GameLanguage) {
    lines.push(String::new());
    lines.push("Language".to_string());
    lines.push(format!(
        "The selected play language is {}. Do not ask the player to choose a language inside the session; keep all player-facing scene, dialogue, choices, and panel text in that language. Only English and Chinese are supported.",
        language.label()
    ));
    lines.push(String::new());
    lines.push("How to play".to_string());
    lines.push("- Type a natural action, a line of dialogue, a numbered choice, or a bracket command shown by the current cartridge.".to_string());
    lines.push("- When the playbook lists action skills, every action must fit exactly one listed skill. Free wording is allowed inside that skill; actions outside the skill set are not valid.".to_string());
    lines.push("- The opening must always restate the background story in the selected language before the first live dialogue beat. After that, keep Dialogue focused on chat and immediate narration; status, tasks, items, choices, and story details belong in their own panels.".to_string());
    lines.push(
        "- Use /game choices for current options, /game render for the scene, /game status for save status, and /skill rule-repeat to see this guide again."
            .to_string(),
    );
}

pub fn load_game_session(workspace: &Path, launch: GameLaunchOptions) -> Result<GameSession> {
    let Some(game_root) = resolve_game_root(workspace, launch.game_or_path.as_deref()) else {
        return Ok(GameSession::Notice(GameSessionNotice {
            message: "no game package selected".to_string(),
            developer_mode: launch.developer_mode,
            language: launch.language,
        }));
    };

    let loaded_game = match deepseek_game::manifest::load_game(&game_root) {
        Ok(game) => game,
        Err(err) => {
            return Ok(GameSession::Notice(GameSessionNotice {
                message: format!("failed to load {}: {err}", game_root.display()),
                developer_mode: launch.developer_mode,
                language: launch.language,
            }));
        }
    };
    let save_id = launch
        .save
        .or_else(|| loaded_game.manifest.game.default_save.clone())
        .unwrap_or_else(|| "default".to_string());
    let loaded_save = match load_save(&loaded_game.saves_root, &save_id) {
        Ok(save) => save,
        Err(err) => {
            return Ok(GameSession::Notice(GameSessionNotice {
                message: format!("failed to load save {save_id}: {err}"),
                developer_mode: launch.developer_mode,
                language: launch.language,
            }));
        }
    };

    match build_loaded_session(
        loaded_game,
        loaded_save,
        launch.developer_mode,
        launch.language,
    ) {
        Ok(session) => Ok(GameSession::Loaded(session)),
        Err(err) => Ok(GameSession::Notice(GameSessionNotice {
            message: format!("failed to load game session: {err}"),
            developer_mode: launch.developer_mode,
            language: launch.language,
        })),
    }
}

fn resolve_game_root(workspace: &Path, explicit: Option<&Path>) -> Option<PathBuf> {
    explicit.map(Path::to_path_buf).or_else(|| {
        workspace
            .join("game.toml")
            .exists()
            .then(|| workspace.to_path_buf())
    })
}

fn build_loaded_session(
    loaded_game: LoadedGame,
    loaded_save: LoadedSave,
    developer_mode: bool,
    language: GameLanguage,
) -> Result<LoadedGameSession> {
    let revision = loaded_save
        .state
        .get("revision")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let locked_driver = driver_lock(&loaded_save.state)?;
    if locked_driver.id != loaded_game.manifest.driver.id {
        anyhow::bail!(
            "save locks driver {}, but game manifest requires {}",
            locked_driver.id,
            loaded_game.manifest.driver.id
        );
    }
    let resolved_driver = resolve_driver(&loaded_game, Some(&locked_driver.version))?;
    let locked_driver_version = resolved_driver.manifest.driver.version.clone();
    let skills = discover_game_skill_catalog(
        &loaded_game.root,
        &loaded_save.root,
        Some(&resolved_driver.root),
    );
    let action_skills = build_playbook(&loaded_save.state).action_skills;
    let mut warnings = loaded_game.warnings;
    warnings.extend(resolved_driver.warnings);
    let panels = render_panels(&loaded_save.state);
    let view = render_view_snapshot(&loaded_save.state);
    Ok(LoadedGameSession {
        game_root: loaded_game.root,
        saves_root: loaded_game.saves_root,
        driver_root: Some(resolved_driver.root),
        game_id: loaded_game.manifest.game.id,
        title: loaded_game.manifest.game.title,
        save_id: loaded_save.id,
        revision,
        driver_id: loaded_game.manifest.driver.id,
        driver_requirement: loaded_game.manifest.driver.version,
        locked_driver_version: Some(locked_driver_version),
        panels,
        view,
        action_skills,
        skills,
        warnings,
        developer_mode,
        language,
    })
}

fn discover_game_skill_catalog(
    game_root: &Path,
    save_root: &Path,
    driver_root: Option<&Path>,
) -> Vec<GameSkillCatalogEntry> {
    let mut roots = vec![
        ("game".to_string(), game_root.join("skills")),
        ("save".to_string(), save_root.join("skills")),
    ];
    if let Some(driver_root) = driver_root {
        roots.push(("driver".to_string(), driver_root.join("skills")));
    }

    let mut entries = Vec::new();
    for (source, root) in roots {
        if !root.is_dir() {
            continue;
        }
        let registry = crate::skills::SkillRegistry::discover(&root);
        for skill in registry.list() {
            if entries
                .iter()
                .any(|entry: &GameSkillCatalogEntry| entry.name == skill.name)
            {
                continue;
            }
            entries.push(GameSkillCatalogEntry {
                name: skill.name.clone(),
                description: skill.description.clone(),
                path: skill.path.clone(),
                source: source.clone(),
            });
        }
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries
}

fn resolve_driver(loaded_game: &LoadedGame, locked_version: Option<&str>) -> Result<LoadedDriver> {
    let roots = driver_roots(&loaded_game.root);
    let resolver = DriverResolver::new(roots);
    let resolved = if let Some(version) = locked_version {
        resolver.resolve_exact(&loaded_game.manifest.driver.id, version)
    } else {
        resolver.resolve(
            &loaded_game.manifest.driver.id,
            &loaded_game.manifest.driver.version,
        )
    }?;
    Ok(resolved.loaded)
}

fn driver_roots(game_root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![game_root.join("drivers")];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".deepseek").join("game-drivers"));
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use deepseek_game::script::DriverCall;
    use serde_json::json;

    #[test]
    fn bundled_reconciliation_demo_loads_with_local_driver() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/games/reconciliation-demo");
        let session = load_game_session(
            Path::new("."),
            GameLaunchOptions {
                game_or_path: Some(root),
                save: Some("default".to_string()),
                developer_mode: false,
                language: GameLanguage::English,
            },
        )
        .expect("demo load should not error");

        let GameSession::Loaded(session) = session else {
            panic!("expected loaded demo game");
        };
        assert_eq!(session.game_id, "reconciliation-demo");
        assert_eq!(session.driver_id, "galgame");
        assert_eq!(session.locked_driver_version.as_deref(), Some("0.1.0"));
        assert!(session.driver_root.is_some());
        assert!(!session.panels.is_empty());
        assert!(session.warnings.is_empty(), "{:?}", session.warnings);
        let intro = session.transcript_intro();
        assert!(intro.contains("Selected language: English"), "{intro}");
        assert!(intro.contains("/skill rule-repeat"), "{intro}");
        assert!(
            intro.contains("every action must fit exactly one listed skill"),
            "{intro}"
        );
        assert!(
            intro.contains("The opening must always restate the background story"),
            "{intro}"
        );
        assert!(intro.contains("Scoped game sub-agents"), "{intro}");
        assert!(intro.contains("dialogue_girlfriend"), "{intro}");
        assert!(intro.contains("game_agent_spawn"), "{intro}");
        assert!(intro.contains("skills/npc/girlfriend/SKILL.md"), "{intro}");

        let agent_packs = session.agent_pack_summaries().expect("agent packs");
        assert!(
            agent_packs
                .iter()
                .any(|pack| pack.role == "dialogue_girlfriend"),
            "{agent_packs:?}"
        );
        let status = session.status_report();
        assert!(status.contains("game_agent_result"), "{status}");

        let rules = session.rules_report().expect("rules report");
        assert!(rules.contains("Rule Repeat"), "{rules}");
        assert!(rules.contains("Current playbook"), "{rules}");
    }

    #[test]
    fn bundled_thirteen_angry_man_loads_with_deliberation_driver() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/games/thirteen-angry-man");
        let session = load_game_session(
            Path::new("."),
            GameLaunchOptions {
                game_or_path: Some(root),
                save: Some("default".to_string()),
                developer_mode: false,
                language: GameLanguage::English,
            },
        )
        .expect("demo load should not error");

        let GameSession::Loaded(session) = session else {
            panic!("expected loaded deliberation game");
        };
        assert_eq!(session.game_id, "thirteen-angry-man");
        assert_eq!(session.driver_id, "deliberation-drama");
        assert_eq!(session.locked_driver_version.as_deref(), Some("0.1.0"));
        assert!(session.driver_root.is_some());
        assert_eq!(session.view.scene_title, "jury room");
        assert!(
            session
                .view
                .status
                .iter()
                .any(|line| line.contains("Votes"))
        );
        assert!(!session.view.tasks.is_empty());
        assert!(
            session.panels.len() >= 6,
            "expected base panels plus action/story panels"
        );
        assert!(session.panels.iter().any(|panel| panel.id == "actions"));
        assert!(session.panels.iter().any(|panel| panel.id == "story"));
        assert!(session.warnings.is_empty(), "{:?}", session.warnings);

        let driver_root = session.driver_root.as_ref().expect("driver root");
        let driver = deepseek_game::driver::load_driver(driver_root).expect("driver should load");
        let result = deepseek_game::script::run_driver_function(
            driver_root,
            &driver.manifest,
            DriverCall {
                function: "advance_room".to_string(),
                args: [
                    ("action_type".to_string(), json!("reconstruction")),
                    ("clock_minutes".to_string(), json!(12)),
                    ("room_heat".to_string(), json!(3)),
                    ("fatigue".to_string(), json!(1)),
                    ("impatience".to_string(), json!(2)),
                    ("conflict_level".to_string(), json!(1)),
                    ("procedure_integrity".to_string(), json!(100)),
                ]
                .into_iter()
                .collect(),
            },
        )
        .expect("declared driver function should run");
        assert_eq!(result.result["clock_minutes"], 22);
        assert_eq!(result.result["time_delta"], 10);
    }

    #[test]
    fn loaded_game_session_refreshes_from_game_tool_json() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/games/reconciliation-demo");
        let mut session = load_game_session(
            Path::new("."),
            GameLaunchOptions {
                game_or_path: Some(root),
                save: Some("default".to_string()),
                developer_mode: false,
                language: GameLanguage::English,
            },
        )
        .expect("demo load should not error");

        let changed = session.refresh_from_game_tool_value(&json!({
            "revision": 3,
            "panels": [
                {
                    "id": "scene",
                    "title": "Changed",
                    "body": "Changed body",
                    "kind": "scene"
                }
            ],
            "view": {
                "revision": 3,
                "scene_title": "Changed",
                "scene": "Changed body",
                "figure_title": "Speaker",
                "figure": "Speaker body",
                "status": ["valid"],
                "items": [],
                "tasks": ["new task"],
                "dialogue": [],
                "choices": [],
                "validation": "valid",
                "scene_art": null,
                "figure_art": null,
                "music": null
            }
        }));

        assert!(changed);
        let GameSession::Loaded(session) = session else {
            panic!("expected loaded game");
        };
        assert_eq!(session.revision, 3);
        assert_eq!(session.panels[0].title, "Changed");
        assert_eq!(session.view.tasks, vec!["new task".to_string()]);
    }

    #[test]
    fn save_locked_driver_must_resolve_exactly() {
        let temp = tempfile::tempdir().expect("tempdir");
        let game = temp.path().join("game");
        fs::create_dir_all(game.join("content")).expect("content dir");
        fs::create_dir_all(game.join("saves/default")).expect("save dir");
        fs::write(
            game.join("game.toml"),
            r#"
[game]
id = "strict-driver-test"
title = "Strict Driver Test"
version = "0.1.0"
default_save = "default"

[driver]
id = "deliberation-drama"
version = "^0.1"

[content]
roots = ["content"]

[saves]
root = "saves"
"#,
        )
        .expect("write manifest");
        fs::write(
            game.join("saves/default/STATE.json"),
            serde_json::to_vec_pretty(&json!({
                "schema_version": 1,
                "revision": 0,
                "driver": {
                    "id": "deliberation-drama",
                    "version": "9.9.9"
                }
            }))
            .expect("serialize state"),
        )
        .expect("write state");
        fs::write(game.join("saves/default/TURN_LOG.jsonl"), "").expect("write turn log");

        let session = load_game_session(
            Path::new("."),
            GameLaunchOptions {
                game_or_path: Some(game),
                save: Some("default".to_string()),
                developer_mode: false,
                language: GameLanguage::English,
            },
        )
        .expect("loading failures are represented as notices");

        let GameSession::Notice(notice) = session else {
            panic!("expected unresolved locked driver to produce a notice");
        };
        assert!(
            notice.message.contains("failed to load game session"),
            "{}",
            notice.message
        );
        assert!(
            notice.message.contains("9.9.9"),
            "notice should name the missing locked version: {}",
            notice.message
        );
    }
}
