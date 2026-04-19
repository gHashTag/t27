//! Git basic tools module.
//!
//! Provides core git operations: status, stage, unstage, commit, create branch.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use trios_core::git::GitOrchestrator;
use trios_git::Git2Orchestrator;

/// Returns the list of allowed repository root directories.
fn allowed_roots() -> Vec<PathBuf> {
    std::env::var("TRIOS_ALLOWED_ROOTS")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Dispatch git basic tools.
pub async fn dispatch(name: &str, input: &Value, repo: &Path, git: &Git2Orchestrator) -> Option<Result<Value>> {
    match name {
        "git_status" => Some(git_status(repo, git).await),
        "git_stage_files" => Some(git_stage_files(repo, git, input).await),
        "git_unstage_files" => Some(git_unstage_files(repo, git, input).await),
        "git_commit" => Some(git_commit(repo, git, input).await),
        "git_create_branch" => Some(git_create_branch(repo, git, input).await),
        _ => None,
    }
}

/// Get git status.
async fn git_status(repo: &Path, git: &Git2Orchestrator) -> Result<Value> {
    let changes = git.status(repo).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(serde_json::to_value(changes)?)
}

/// Stage files for commit.
async fn git_stage_files(repo: &Path, git: &Git2Orchestrator, input: &Value) -> Result<Value> {
    let paths: Vec<String> = input
        .get("paths")
        .and_then(|v| v.as_array())
        .context("paths array required")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let path_refs: Vec<&Path> = paths.iter().map(|p| Path::new(p.as_str())).collect();
    git.stage(repo, &path_refs).await?;
    Ok(serde_json::json!({"staged": paths.len()}))
}

/// Unstage files.
async fn git_unstage_files(repo: &Path, git: &Git2Orchestrator, input: &Value) -> Result<Value> {
    let paths: Vec<String> = input
        .get("paths")
        .and_then(|v| v.as_array())
        .context("paths array required")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let path_refs: Vec<&Path> = paths.iter().map(|p| Path::new(p.as_str())).collect();
    git.unstage(repo, &path_refs).await?;
    Ok(serde_json::json!({"unstaged": paths.len()}))
}

/// Create a commit.
async fn git_commit(repo: &Path, git: &Git2Orchestrator, input: &Value) -> Result<Value> {
    let message = input
        .get("message")
        .and_then(|v| v.as_str())
        .context("message required")?;
    if message.len() > 4096 {
        bail!("commit message too long (max 4096 characters)");
    }
    if message.is_empty() {
        bail!("commit message cannot be empty");
    }
    let result = git.commit(repo, message).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(serde_json::to_value(result)?)
}

/// Create a new branch.
async fn git_create_branch(repo: &Path, git: &Git2Orchestrator, input: &Value) -> Result<Value> {
    let branch_name = input
        .get("name")
        .and_then(|v| v.as_str())
        .context("name required")?;
    if branch_name.is_empty() {
        bail!("branch name cannot be empty");
    }
    if branch_name.contains(' ') || branch_name.contains("..") || branch_name.contains('~')
        || branch_name.contains('^') || branch_name.contains(':')
    {
        bail!("branch name contains invalid characters");
    }
    git.create_branch(repo, branch_name).await?;
    Ok(serde_json::json!({"created": branch_name}))
}
