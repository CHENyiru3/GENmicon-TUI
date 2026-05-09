use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, GameError>;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML {path}: {source}")]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to parse JSON {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("invalid driver manifest: {0}")]
    InvalidDriverManifest(String),
    #[error("invalid path for {label}: {path}")]
    InvalidPath { label: String, path: PathBuf },
    #[error("path for {label} escapes root {root}: {path}")]
    PathEscape {
        label: String,
        root: PathBuf,
        path: PathBuf,
    },
    #[error("driver {id} matching {requirement} was not found")]
    DriverNotFound { id: String, requirement: String },
    #[error("invalid version requirement {0}")]
    InvalidVersionRequirement(String),
    #[error("invalid version {0}")]
    InvalidVersion(String),
    #[error("save validation failed: {0}")]
    SaveValidation(String),
    #[error("save revision conflict: expected {expected}, found {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("lookup failed: {0}")]
    Lookup(String),
    #[error("script execution failed: {0}")]
    Script(String),
}
