use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths;
use crate::{GameError, Result};

pub const DEFAULT_LOOKUP_BYTES: usize = 16 * 1024;
pub const HARD_LOOKUP_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LookupRequest {
    pub handle: Option<String>,
    pub query: Option<String>,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupResult {
    pub excerpts: Vec<LookupExcerpt>,
    pub bytes_returned: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupExcerpt {
    pub source_handle: String,
    pub text: String,
}

pub fn lookup(
    game_root: impl AsRef<Path>,
    content_roots: &[PathBuf],
    request: LookupRequest,
) -> Result<LookupResult> {
    if request.handle.is_none() && request.query.is_none() {
        return Err(GameError::Lookup(
            "at least one of handle or query is required".to_string(),
        ));
    }
    let game_root = paths::canonicalize_root(game_root, "game root")?;
    let budget = request
        .max_bytes
        .unwrap_or(DEFAULT_LOOKUP_BYTES)
        .min(HARD_LOOKUP_BYTES);
    let roots = content_roots
        .iter()
        .map(|root| ensure_content_root(&game_root, root))
        .collect::<Result<Vec<_>>>()?;

    if let Some(handle) = request.handle {
        return lookup_handle(&game_root, &roots, &handle, budget);
    }
    let query = request.query.unwrap_or_default();
    lookup_query(&game_root, &roots, &query, budget)
}

fn lookup_handle(
    game_root: &Path,
    roots: &[PathBuf],
    handle: &str,
    budget: usize,
) -> Result<LookupResult> {
    let relative = paths::normalize_relative(handle, "content handle")?;
    for root in roots {
        let candidate = root.join(&relative);
        for path in [candidate.clone(), candidate.with_extension("md")] {
            if path.is_file() {
                return excerpt_file(game_root, &path, budget);
            }
        }
    }
    Err(GameError::Lookup(format!(
        "content handle not found: {handle}"
    )))
}

fn lookup_query(
    game_root: &Path,
    roots: &[PathBuf],
    query: &str,
    budget: usize,
) -> Result<LookupResult> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return Err(GameError::Lookup("query cannot be empty".to_string()));
    }
    let mut result = LookupResult {
        excerpts: Vec::new(),
        bytes_returned: 0,
        truncated: false,
    };
    for root in roots {
        for path in collect_content_files(root)? {
            let raw = std::fs::read_to_string(&path).map_err(|source| GameError::Read {
                path: path.clone(),
                source,
            })?;
            if !raw.to_ascii_lowercase().contains(&needle) {
                continue;
            }
            push_excerpt(game_root, &path, &raw, budget, &mut result)?;
            if result.truncated || result.bytes_returned >= budget {
                return Ok(result);
            }
        }
    }
    Ok(result)
}

fn excerpt_file(game_root: &Path, path: &Path, budget: usize) -> Result<LookupResult> {
    let raw = std::fs::read_to_string(path).map_err(|source| GameError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut result = LookupResult {
        excerpts: Vec::new(),
        bytes_returned: 0,
        truncated: false,
    };
    push_excerpt(game_root, path, &raw, budget, &mut result)?;
    Ok(result)
}

fn push_excerpt(
    game_root: &Path,
    path: &Path,
    raw: &str,
    budget: usize,
    result: &mut LookupResult,
) -> Result<()> {
    let remaining = budget.saturating_sub(result.bytes_returned);
    if remaining == 0 {
        result.truncated = true;
        return Ok(());
    }
    let mut text = raw.to_string();
    if text.len() > remaining {
        text.truncate(floor_char_boundary(&text, remaining));
        result.truncated = true;
    }
    result.bytes_returned += text.len();
    let handle = paths::relative_to(game_root, path, "lookup result")?
        .to_string_lossy()
        .replace('\\', "/");
    result.excerpts.push(LookupExcerpt {
        source_handle: handle,
        text,
    });
    Ok(())
}

fn floor_char_boundary(value: &str, mut index: usize) -> usize {
    while index > 0 && !value.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn ensure_content_root(game_root: &Path, root: &Path) -> Result<PathBuf> {
    let root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        game_root.join(root)
    };
    let canonical = std::fs::canonicalize(&root).map_err(|source| GameError::Read {
        path: root.clone(),
        source,
    })?;
    if !canonical.starts_with(game_root) {
        return Err(GameError::PathEscape {
            label: "content root".to_string(),
            root: game_root.to_path_buf(),
            path: canonical,
        });
    }
    Ok(canonical)
}

fn collect_content_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_content_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_content_files_inner(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(root).map_err(|source| GameError::Read {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| GameError::Read {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| GameError::Read {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_content_files_inner(&path, files)?;
        } else if file_type.is_file()
            && matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("md" | "txt")
            )
        {
            files.push(path);
        }
    }
    Ok(())
}
