use std::path::{Component, Path, PathBuf};

use crate::{GameError, Result};

pub fn canonicalize_root(path: impl AsRef<Path>, label: &str) -> Result<PathBuf> {
    let path = path.as_ref();
    let canonical = std::fs::canonicalize(path).map_err(|source| GameError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !canonical.is_dir() {
        return Err(GameError::InvalidPath {
            label: label.to_string(),
            path: canonical,
        });
    }
    Ok(canonical)
}

pub fn normalize_relative(path: impl AsRef<Path>, label: &str) -> Result<PathBuf> {
    let path = path.as_ref();
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(GameError::InvalidPath {
                    label: label.to_string(),
                    path: path.to_path_buf(),
                });
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(GameError::InvalidPath {
            label: label.to_string(),
            path: path.to_path_buf(),
        });
    }
    Ok(normalized)
}

pub fn resolve_under(
    root: impl AsRef<Path>,
    relative: impl AsRef<Path>,
    label: &str,
) -> Result<PathBuf> {
    let root = root.as_ref();
    let relative = normalize_relative(relative, label)?;
    let joined = root.join(relative);
    if !joined.starts_with(root) {
        return Err(GameError::PathEscape {
            label: label.to_string(),
            root: root.to_path_buf(),
            path: joined,
        });
    }
    Ok(joined)
}

pub fn resolve_existing_under(
    root: impl AsRef<Path>,
    relative: impl AsRef<Path>,
    label: &str,
) -> Result<PathBuf> {
    let root = root.as_ref();
    let candidate = resolve_under(root, relative, label)?;
    let canonical = std::fs::canonicalize(&candidate).map_err(|source| GameError::Read {
        path: candidate.clone(),
        source,
    })?;
    if !canonical.starts_with(root) {
        return Err(GameError::PathEscape {
            label: label.to_string(),
            root: root.to_path_buf(),
            path: canonical,
        });
    }
    Ok(canonical)
}

pub fn relative_to(root: impl AsRef<Path>, path: impl AsRef<Path>, label: &str) -> Result<PathBuf> {
    let root = root.as_ref();
    let path = path.as_ref();
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| GameError::PathEscape {
            label: label.to_string(),
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
}
