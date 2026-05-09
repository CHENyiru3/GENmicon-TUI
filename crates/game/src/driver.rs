use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::manifest::validate_id;
use crate::paths;
use crate::{GameError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverManifest {
    pub driver: DriverSection,
    #[serde(default)]
    pub runtime: RuntimeSection,
    #[serde(default)]
    pub skills: SkillsSection,
    #[serde(default)]
    pub scripts: ScriptsSection,
    #[serde(default)]
    pub subagents: SubagentsSection,
    #[serde(default)]
    pub functions: BTreeMap<String, DriverFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverSection {
    pub id: String,
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSection {
    pub script_engine: String,
    pub default_topology: String,
}

impl Default for RuntimeSection {
    fn default() -> Self {
        Self {
            script_engine: "starlark".to_string(),
            default_topology: "dynamic-main-plus-managers".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillsSection {
    pub entry: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptsSection {
    pub root: PathBuf,
}

impl Default for ScriptsSection {
    fn default() -> Self {
        Self {
            root: PathBuf::from("scripts"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentsSection {
    #[serde(default)]
    pub default_roles: Vec<String>,
    #[serde(default = "default_max_active")]
    pub max_active: usize,
    #[serde(default)]
    pub templates: BTreeMap<String, PathBuf>,
}

impl Default for SubagentsSection {
    fn default() -> Self {
        Self {
            default_roles: Vec::new(),
            max_active: default_max_active(),
            templates: BTreeMap::new(),
        }
    }
}

fn default_max_active() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverFunction {
    pub script: PathBuf,
    pub function: String,
    #[serde(default)]
    pub mutates: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedDriver {
    pub root: PathBuf,
    pub manifest: DriverManifest,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDriver {
    pub install_root: PathBuf,
    pub loaded: LoadedDriver,
}

#[derive(Debug, Clone)]
pub struct DriverResolver {
    roots: Vec<PathBuf>,
}

impl DriverResolver {
    pub fn new(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            roots: roots.into_iter().collect(),
        }
    }

    pub fn resolve(&self, id: &str, requirement: &str) -> Result<ResolvedDriver> {
        let requirement = VersionReq::parse(requirement)?;
        let mut candidates = self.driver_candidates(id)?;
        candidates.retain(|candidate| requirement.matches(&candidate.version));
        candidates.sort_by(|left, right| right.version.cmp(&left.version));
        let Some(candidate) = candidates.into_iter().next() else {
            return Err(GameError::DriverNotFound {
                id: id.to_string(),
                requirement: requirement.raw,
            });
        };
        let loaded = load_driver(&candidate.root)?;
        let expected_version = candidate.version.to_string();
        if loaded.manifest.driver.id != id || loaded.manifest.driver.version != expected_version {
            return Err(GameError::InvalidDriverManifest(format!(
                "driver installed at {} must declare id {id} version {expected_version}, found {} {}",
                candidate.root.display(),
                loaded.manifest.driver.id,
                loaded.manifest.driver.version
            )));
        }
        Ok(ResolvedDriver {
            install_root: candidate.install_root,
            loaded,
        })
    }

    pub fn resolve_exact(&self, id: &str, version: &str) -> Result<ResolvedDriver> {
        let exact = VersionReq::parse(version)?;
        if !matches!(exact.kind, VersionReqKind::Exact(_)) {
            return Err(GameError::InvalidVersionRequirement(version.to_string()));
        }
        self.resolve(id, version)
    }

    fn driver_candidates(&self, id: &str) -> Result<Vec<DriverCandidate>> {
        validate_id(id, "driver.id")?;
        let mut candidates = Vec::new();
        for root in &self.roots {
            let install_root = match paths::canonicalize_root(root, "driver install root") {
                Ok(root) => root,
                Err(_) => continue,
            };
            let id_root = install_root.join(id);
            let Ok(entries) = std::fs::read_dir(&id_root) else {
                continue;
            };
            for entry in entries {
                let entry = entry.map_err(|source| GameError::Read {
                    path: id_root.clone(),
                    source,
                })?;
                let file_type = entry.file_type().map_err(|source| GameError::Read {
                    path: entry.path(),
                    source,
                })?;
                if !file_type.is_dir() {
                    continue;
                }
                let raw_version = entry.file_name().to_string_lossy().to_string();
                let Ok(version) = Version::parse(&raw_version) else {
                    continue;
                };
                candidates.push(DriverCandidate {
                    install_root: install_root.clone(),
                    root: entry.path(),
                    version,
                });
            }
        }
        Ok(candidates)
    }
}

#[derive(Debug, Clone)]
struct DriverCandidate {
    install_root: PathBuf,
    root: PathBuf,
    version: Version,
}

pub fn load_driver(root: impl AsRef<Path>) -> Result<LoadedDriver> {
    let root = paths::canonicalize_root(root, "driver root")?;
    let manifest_path = root.join("driver.toml");
    let raw = std::fs::read_to_string(&manifest_path).map_err(|source| GameError::Read {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest: DriverManifest = toml::from_str(&raw).map_err(|source| GameError::Toml {
        path: manifest_path,
        source,
    })?;
    validate_driver_manifest(&manifest)?;

    let mut warnings = Vec::new();
    if let Some(entry) = &manifest.skills.entry {
        let resolved = paths::resolve_under(&root, entry, "driver entry skill")?;
        if !resolved.exists() {
            warnings.push(format!(
                "driver entry skill does not exist: {}",
                entry.display()
            ));
        }
    }
    paths::resolve_under(&root, &manifest.scripts.root, "driver scripts root")?;
    for (name, function) in &manifest.functions {
        validate_id(name, "function name")?;
        let script_path = paths::resolve_under(&root, &function.script, "driver function script")?;
        if !script_path.exists() {
            warnings.push(format!(
                "driver function script does not exist: {}",
                function.script.display()
            ));
        }
    }
    for (role, template) in &manifest.subagents.templates {
        validate_id(role, "subagent role")?;
        let resolved = paths::resolve_under(&root, template, "subagent template")?;
        if !resolved.exists() {
            warnings.push(format!(
                "subagent template does not exist: {}",
                template.display()
            ));
        }
    }
    Ok(LoadedDriver {
        root,
        manifest,
        warnings,
    })
}

pub fn validate_driver_manifest(manifest: &DriverManifest) -> Result<()> {
    validate_id(&manifest.driver.id, "driver.id")
        .map_err(|err| GameError::InvalidDriverManifest(err.to_string()))?;
    Version::parse(&manifest.driver.version)?;
    if manifest.driver.title.trim().is_empty() {
        return Err(GameError::InvalidDriverManifest(
            "driver.title cannot be empty".to_string(),
        ));
    }
    paths::normalize_relative(&manifest.scripts.root, "scripts root")?;
    for function in manifest.functions.values() {
        paths::normalize_relative(&function.script, "driver function script")?;
        if function.function.trim().is_empty() {
            return Err(GameError::InvalidDriverManifest(
                "driver function name cannot be empty".to_string(),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    fn parse(raw: &str) -> Result<Self> {
        let mut parts = raw.split('.');
        let major = parse_version_part(parts.next(), raw)?;
        let minor = parse_version_part(parts.next(), raw)?;
        let patch = parse_version_part(parts.next(), raw)?;
        if parts.next().is_some() {
            return Err(GameError::InvalidVersion(raw.to_string()));
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn parse_version_part(part: Option<&str>, raw: &str) -> Result<u64> {
    let Some(part) = part else {
        return Err(GameError::InvalidVersion(raw.to_string()));
    };
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(GameError::InvalidVersion(raw.to_string()));
    }
    part.parse::<u64>()
        .map_err(|_| GameError::InvalidVersion(raw.to_string()))
}

#[derive(Debug, Clone)]
struct VersionReq {
    raw: String,
    kind: VersionReqKind,
}

#[derive(Debug, Clone)]
enum VersionReqKind {
    Any,
    Exact(Version),
    Caret(Version),
}

impl VersionReq {
    fn parse(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw == "*" {
            return Ok(Self {
                raw: raw.to_string(),
                kind: VersionReqKind::Any,
            });
        }
        if let Some(version) = raw.strip_prefix('^') {
            let normalized = normalize_partial_version(version)?;
            return Ok(Self {
                raw: raw.to_string(),
                kind: VersionReqKind::Caret(Version::parse(&normalized)?),
            });
        }
        Ok(Self {
            raw: raw.to_string(),
            kind: VersionReqKind::Exact(Version::parse(raw)?),
        })
    }

    fn matches(&self, version: &Version) -> bool {
        match &self.kind {
            VersionReqKind::Any => true,
            VersionReqKind::Exact(required) => version == required,
            VersionReqKind::Caret(required) => {
                if version < required {
                    return false;
                }
                if required.major > 0 {
                    version.major == required.major
                } else if required.minor > 0 {
                    version.major == 0 && version.minor == required.minor
                } else {
                    version.major == 0 && version.minor == 0 && version.patch == required.patch
                }
            }
        }
    }
}

fn normalize_partial_version(raw: &str) -> Result<String> {
    let parts = raw.split('.').collect::<Vec<_>>();
    match parts.as_slice() {
        [major] => Ok(format!("{major}.0.0")),
        [major, minor] => Ok(format!("{major}.{minor}.0")),
        [_, _, _] => Ok(raw.to_string()),
        _ => Err(GameError::InvalidVersionRequirement(raw.to_string())),
    }
}
