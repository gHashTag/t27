//! Filesystem tools module.
//!
//! Provides file and directory operations with security validation.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::PathBuf;

/// Returns the list of allowed repository root directories.
/// Configured via `TRIOS_ALLOWED_ROOTS` env var (colon-separated paths).
/// If empty, all absolute paths are allowed.
fn allowed_roots() -> Vec<PathBuf> {
    std::env::var("TRIOS_ALLOWED_ROOTS")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Dispatch filesystem tools.
pub async fn dispatch(name: &str, input: &Value) -> Option<Result<Value>> {
    match name {
        "fs_read_file" => Some(fs_read_file(input).await),
        "fs_write_file" => Some(fs_write_file(input).await),
        "fs_list_dir" => Some(fs_list_dir(input).await),
        _ => None,
    }
}

/// Read file contents (must be within allowed roots).
async fn fs_read_file(input: &Value) -> Result<Value> {
    let path = input
        .get("path")
        .and_then(|v| v.as_str())
        .context("path is required")?;

    let path = PathBuf::from(path);
    if !path.is_absolute() {
        bail!("path must be absolute");
    }

    // Validate against allowed roots
    let roots = allowed_roots();
    if !roots.is_empty() {
        let canonical = path.canonicalize().map_err(|e| anyhow::anyhow!("invalid path: {}", e))?;
        let permitted = roots.iter().any(|root| {
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            canonical.starts_with(&canonical_root)
        });
        if !permitted {
            bail!("path is outside allowed directories");
        }
    }

    let content = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
        "content": content,
        "size_bytes": content.len(),
    }))
}

/// Write content to a file (must be within allowed roots).
async fn fs_write_file(input: &Value) -> Result<Value> {
    let path = input
        .get("path")
        .and_then(|v| v.as_str())
        .context("path is required")?;
    let content = input
        .get("content")
        .and_then(|v| v.as_str())
        .context("content is required")?;

    if content.len() > 1024 * 1024 {
        bail!("content too large (max 1 MB)");
    }

    let path = PathBuf::from(path);
    if !path.is_absolute() {
        bail!("path must be absolute");
    }

    // Validate against allowed roots
    let roots = allowed_roots();
    if !roots.is_empty() {
        let parent = path.parent().context("path has no parent directory")?;
        let canonical = parent.canonicalize().map_err(|e| anyhow::anyhow!("invalid parent dir: {}", e))?;
        let permitted = roots.iter().any(|root| {
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            canonical.starts_with(&canonical_root)
        });
        if !permitted {
            bail!("path is outside allowed directories");
        }
    }

    tokio::fs::write(&path, content).await?;
    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
        "written": true,
        "size_bytes": content.len(),
    }))
}

/// List directory contents (must be within allowed roots).
async fn fs_list_dir(input: &Value) -> Result<Value> {
    let path = input
        .get("path")
        .and_then(|v| v.as_str())
        .context("path is required")?;

    let path = PathBuf::from(path);
    if !path.is_absolute() {
        bail!("path must be absolute");
    }
    if !path.is_dir() {
        bail!("path is not a directory");
    }

    // Validate against allowed roots
    let roots = allowed_roots();
    if !roots.is_empty() {
        let canonical = path.canonicalize().map_err(|e| anyhow::anyhow!("invalid path: {}", e))?;
        let permitted = roots.iter().any(|root| {
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            canonical.starts_with(&canonical_root)
        });
        if !permitted {
            bail!("path is outside allowed directories");
        }
    }

    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(&path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().await?.is_dir();
        entries.push(serde_json::json!({
            "name": name,
            "is_dir": is_dir,
        }));
    }

    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
        "entries": entries,
        "count": entries.len(),
    }))
}
