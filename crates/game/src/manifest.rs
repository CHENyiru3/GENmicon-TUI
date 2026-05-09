use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths;
use crate::{GameError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameManifest {
    pub game: GameSection,
    pub driver: DriverRef,
    #[serde(default)]
    pub content: ContentSection,
    #[serde(default)]
    pub saves: SavesSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameSection {
    pub id: String,
    pub title: String,
    pub version: String,
    pub entry_skill: Option<String>,
    pub default_save: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverRef {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentSection {
    pub index: Option<PathBuf>,
    #[serde(default)]
    pub roots: Vec<PathBuf>,
}

impl Default for ContentSection {
    fn default() -> Self {
        Self {
            index: Some(PathBuf::from("content/INDEX.md")),
            roots: vec![PathBuf::from("content")],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavesSection {
    pub root: PathBuf,
}

impl Default for SavesSection {
    fn default() -> Self {
        Self {
            root: PathBuf::from("saves"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedGame {
    pub root: PathBuf,
    pub manifest: GameManifest,
    pub content_roots: Vec<PathBuf>,
    pub content_index: Option<PathBuf>,
    pub saves_root: PathBuf,
    pub warnings: Vec<String>,
}

pub fn load_game(root: impl AsRef<Path>) -> Result<LoadedGame> {
    let root = paths::canonicalize_root(root, "game root")?;
    let manifest_path = root.join("game.toml");
    let raw = std::fs::read_to_string(&manifest_path).map_err(|source| GameError::Read {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest: GameManifest = toml::from_str(&raw).map_err(|source| GameError::Toml {
        path: manifest_path,
        source,
    })?;
    validate_game_manifest(&manifest)?;

    let mut warnings = Vec::new();
    let content_roots = manifest
        .content
        .roots
        .iter()
        .map(|path| {
            let resolved = paths::resolve_under(&root, path, "content root")?;
            if !resolved.exists() {
                warnings.push(format!("content root does not exist: {}", path.display()));
            }
            Ok(resolved)
        })
        .collect::<Result<Vec<_>>>()?;

    let content_index = manifest
        .content
        .index
        .as_ref()
        .map(|path| {
            let resolved = paths::resolve_under(&root, path, "content index")?;
            if !resolved.exists() {
                warnings.push(format!("content index does not exist: {}", path.display()));
            }
            Ok(resolved)
        })
        .transpose()?;

    if let Some(entry_skill) = &manifest.game.entry_skill {
        let skill_path = paths::resolve_under(
            &root,
            PathBuf::from("skills").join(entry_skill).join("SKILL.md"),
            "entry skill",
        )?;
        if !skill_path.exists() {
            warnings.push(format!("entry skill does not exist: {entry_skill}"));
        }
    }

    let saves_root = paths::resolve_under(&root, &manifest.saves.root, "saves root")?;
    if !saves_root.exists() {
        warnings.push(format!(
            "saves root does not exist: {}",
            manifest.saves.root.display()
        ));
    }

    Ok(LoadedGame {
        root,
        manifest,
        content_roots,
        content_index,
        saves_root,
        warnings,
    })
}

pub fn validate_game_manifest(manifest: &GameManifest) -> Result<()> {
    validate_id(&manifest.game.id, "game.id")?;
    validate_id(&manifest.driver.id, "driver.id")?;
    if manifest.game.title.trim().is_empty() {
        return Err(GameError::InvalidManifest(
            "game.title cannot be empty".to_string(),
        ));
    }
    if manifest.game.version.trim().is_empty() {
        return Err(GameError::InvalidManifest(
            "game.version cannot be empty".to_string(),
        ));
    }
    if manifest.driver.version.trim().is_empty() {
        return Err(GameError::InvalidManifest(
            "driver.version cannot be empty".to_string(),
        ));
    }
    paths::normalize_relative(&manifest.saves.root, "saves root")?;
    for root in &manifest.content.roots {
        paths::normalize_relative(root, "content root")?;
    }
    if let Some(index) = &manifest.content.index {
        paths::normalize_relative(index, "content index")?;
    }
    Ok(())
}

pub fn validate_id(value: &str, label: &str) -> Result<()> {
    let valid = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(GameError::InvalidManifest(format!(
            "{label} must be filesystem-safe"
        )))
    }
}
