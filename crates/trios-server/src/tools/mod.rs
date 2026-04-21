//! Tools module — chain-of-responsibility dispatch.
//!
//! Each tool module returns `Option<Result<Value>>`:
//! - `Some(Ok(...))` — tool handled
//! - `Some(Err(...))` — tool error
//! - `None` — pass to next module

pub mod fs;
pub mod git;
pub mod git_extended;
pub mod gitbutler;
pub mod golden_float;
pub mod trios_kg;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use trios_git::Git2Orchestrator;
use crate::security::validate_repo_path;

/// Returns the list of allowed repository root directories.
fn allowed_roots() -> Vec<PathBuf> {
    std::env::var("TRIOS_ALLOWED_ROOTS")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Returns the number of registered tools.
pub fn count() -> usize {
    fs::tool_count() + trios_kg::tool_count() + git::tool_count()
        + git_extended::tool_count() + gitbutler::tool_count() + golden_float::tool_count()
}

/// Main tool dispatcher — chain-of-responsibility pattern.
pub async fn dispatch(name: &str, input: Value) -> Result<Value> {
    // Filesystem tools first (no repo_path validation needed)
    if let Some(r) = fs::dispatch(name, &input).await { return r; }

    // Knowledge Graph tools (no repo_path needed)
    if let Some(r) = trios_kg::dispatch(name, &input).await { return r; }

    // System status tool (no repo_path needed)
    if name == "system_status" {
        return Ok(serde_json::json!({
            "status": "ok",
            "tools_registered": count(),
            "uptime_hint": "see /health",
            "version": env!("CARGO_PKG_VERSION"),
        }));
    }

    // Validate repo_path for git tools
    let raw_repo_path = input
        .get("repo_path")
        .and_then(|v| v.as_str())
        .context("repo_path is required")?;

    let repo_path = validate_repo_path(raw_repo_path, &allowed_roots())
        .map_err(|e| anyhow::anyhow!(e))?;
    let repo = repo_path.as_path();
    let git = Git2Orchestrator;

    // Git basic tools
    if let Some(r) = git::dispatch(name, &input, repo, &git).await { return r; }

    // Git extended tools
    if let Some(r) = git_extended::dispatch(name, &input, repo, &git).await { return r; }

    // GitButler tools
    if let Some(r) = gitbutler::dispatch(name, &input, repo, &git).await { return r; }

    // Golden Float tools
    if let Some(r) = golden_float::dispatch(name, &input).await { return r; }

    bail!("unknown tool: {name}")
}
