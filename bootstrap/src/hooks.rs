//! `t27c hook-*` — Rust implementation of git hooks for T27.
//!
//! This module provides the Rust implementation of git hooks that replace the shell scripts
//! previously used in the repository. The hooks are versioned and reviewable as part of
//! the tracked codebase.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Date, Utc};
use clap::{Parser, Subcommand};
use regex::Regex;

/// Git hooks commands for T27
#[derive(Parser)]
#[command(name = "t27c")]
// `t27c version` existed; `--version` and `-V` did not, so the flag every
// other CLI answers returned "unexpected argument". One attribute.
#[command(version)]
#[command(about = "T27 Bootstrap Compiler for Trinity S³AI Framework", long_about = None)]
pub struct HooksCli {
    #[command(subcommand)]
    pub cmd: HooksCmd,
}

/// Subcommands for git hooks
#[derive(Subcommand, Debug)]
pub enum HooksCmd {
    /// Run pre-commit gates: NOW freshness, seal presence, no-new-.sh, cargo check
    PreCommit,
    /// Check L1 TRACEABILITY over the commit message being written
    CommitMsg {
        /// Path to commit message file (passed by git)
        path: PathBuf,
    },
    /// Install git hooks by setting core.hooksPath to .githooks/
    InstallHooks,
}

/// Execute the appropriate hook command
pub fn run(cmd: &HooksCmd) -> Result<()> {
    match cmd {
        HooksCmd::PreCommit => pre_commit(),
        HooksCmd::CommitMsg { path } => commit_msg(path),
        HooksCmd::InstallHooks => install_hooks(),
    }
}

/// Run pre-commit gates
fn pre_commit() -> Result<()> {
    // NOW freshness check
    now_freshness()?;
    
    // Seal presence check (non-blocking)
    seal_presence()?;
    
    // No-new-.sh files check
    no_new_sh_files()?;
    
    // Cargo check when Rust files changed
    if rust_files_changed()? {
        cargo_check()?;
    }
    
    println!("✅ Pre-commit gates passed");
    Ok(())
}

/// Check that a fresh docs/now/ entry exists
fn now_freshness() -> Result<()> {
    let root = repo_root()?;
    let now_dir = root.join("docs/now");
    
    if !now_dir.exists() {
        bail!("❌ docs/now/ directory does not exist");
    }
    
    let today = Utc::now().date_naive();
    let entry_pattern = format!("^{}-{{.*}}\\.md$", today.format("%Y-%m-%d"));
    let entry_re = Regex::new(&entry_pattern).context("Failed to compile entry regex")?;
    
    // Check staged files
    let staged_files = git_diff_cached_names()?;
    let has_staged_entry = staged_files.iter().any(|file| {
        let file_str = file.to_string_lossy();
        entry_re.is_match(&file_str)
    });
    
    // Check unstaged files if no staged entry
    if !has_staged_entry {
        let status_output = Command::new("git")
            .args(["status", "--porcelain", "--untracked-files=all", "docs/now"])
            .current_dir(&root)
            .output()
            .context("Failed to run git status")?;
            
        if status_output.status.success() {
            let status_text = String::from_utf8_lossy(&status_output.stdout);
            let unstaged_files: Vec<&str> = status_text
                .lines()
                .filter(|line| line.len() >= 3)
                .map(|line| &line[3..])
                .collect();
            
            let has_unstaged_entry = unstaged_files.iter().any(|file| {
                let file_str = file.to_string_lossy();
                entry_re.is_match(&file_str)
            });
            
            if has_unstaged_entry {
                println!("⚠️  WARNING: a docs/now/ entry exists but is NOT staged");
                println!("   Run: git add docs/now");
                println!("   Or: stage and commit it together with your changes");
                println!("");
            }
        }
    }
    
    // Check if any entry exists for today
    let entries: Vec<PathBuf> = std::fs::read_dir(&now_dir)
        .context("Failed to read docs/now/ directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.extension().and_then(|ext| ext.to_str()) == Some("md") &&
                entry_re.is_match(&path.file_name().unwrap().to_string_lossy())
        })
        .collect();
    
    if entries.is_empty() {
        bail!("❌ No fresh docs/now/ entry found for today ({})", today.format("%Y-%m-%d"));
    }
    
    println!("✅ NOW freshness check passed");
    Ok(())
}

/// Check that seals are present (non-blocking)
fn seal_presence() -> Result<()> {
    let root = repo_root()?;
    let seals_dir = root.join(".trinity/seals");
    
    if !seals_dir.exists() {
        println!("⚠️  Seals directory does not exist - skipping seal check");
        return Ok(());
    }
    
    let seal_files: Vec<PathBuf> = std::fs::read_dir(&seals_dir)
        .context("Failed to read seals directory")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();
    
    if seal_files.is_empty() {
        println!("⚠️  No seal files found - skipping seal check");
        return Ok(());
    }
    
    // Check if any seal files are staged
    let staged_files = git_diff_cached_names()?;
    let staged_seals = staged_files.iter().any(|file| {
        let file_str = file.to_string_lossy();
        file_str.starts_with(".trinity/seals/")
    });
    
    if staged_seals {
        println!("✅ Seal presence check passed (seals staged)");
    } else {
        println!("ℹ️  No seals staged - seal check passed");
    }
    
    Ok(())
}

/// Check for new .sh files
fn no_new_sh_files() -> Result<()> {
    let staged_files = git_diff_cached_names()?;
    
    let new_sh_files = staged_files.iter().filter(|file| {
        let file_str = file.to_string_lossy();
        file_str.ends_with(".sh")
    }).collect::<Vec<_>>();
    
    if !new_sh_files.is_empty() {
        bail!("❌ New .sh files are not allowed: {:?}", new_sh_files);
    }
    
    println!("✅ No new .sh files check passed");
    Ok(())
}

/// Check if Rust files have been changed
fn rust_files_changed() -> Result<bool> {
    let staged_files = git_diff_cached_names()?;
    
    let rust_extensions = [".rs", ".t27"];
    let rust_files = staged_files.iter().filter(|file| {
        let file_str = file.to_string_lossy();
        rust_extensions.iter().any(|ext| file_str.ends_with(ext))
    }).collect::<Vec<_>>();
    
    Ok(!rust_files.is_empty())
}

/// Run cargo check
fn cargo_check() -> Result<()> {
    println!("🔍 Running cargo check...");
    
    let output = Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(repo_root()?)
        .output()
        .context("Failed to run cargo check")?;
    
    if !output.status.success() {
        bail!("❌ cargo check failed:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("✅ cargo check passed");
    Ok(())
}

/// Run commit-msg hook (L1 TRACEABILITY)
fn commit_msg(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read commit message file: {}", path.display()))?;
    
    check_commit_message(&content)?;
    println!("✅ L1 TRACEABILITY check passed");
    Ok(())
}

/// Check commit message for L1 TRACEABILITY requirements
fn check_commit_message(msg: &str) -> Result<()> {
    // Strip comment lines exactly as git does
    let clean_msg = msg
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    
    let re = Regex::new(L1_PATTERN).expect("static regex always compiles");
    
    match re.captures(&clean_msg) {
        Some(caps) => {
            let issue = caps.get(2).map(|m| m.as_str()).unwrap_or("?");
            println!("L1 PASSED: Issue #{} referenced", issue);
            Ok(())
        }
        None => {
            eprintln!("L1 VIOLATION: Commit missing issue reference");
            eprintln!("Expected: Closes #N, Fixes #N, Resolves #N, Refs #N, or Updates #N (case-insensitive)");
            eprintln!("Examples: Closes #123, Fixes #456, Resolves #789");
            eprintln!("Note: Bare '#123' is rejected - relationship must be stated");
            bail!("L1 traceability requirement not met");
        }
    }
}

/// Install git hooks by setting core.hooksPath
fn install_hooks() -> Result<()> {
    let root = repo_root()?;
    let githooks_dir = root.join(".githooks");
    
    if !githooks_dir.exists() {
        bail!(".githooks/ directory does not exist at {}", githooks_dir.display());
    }
    
    // Check that required hook files exist
    let required_hooks = ["pre-commit", "commit-msg"];
    for hook in &required_hooks {
        let hook_path = githooks_dir.join(hook);
        if !hook_path.exists() {
            bail!("Required hook file {} does not exist", hook_path.display());
        }
    }
    
    // Set core.hooksPath to .githooks/
    let output = Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .current_dir(&root)
        .output()
        .context("Failed to set git config")?;
    
    if !output.status.success() {
        bail!("Failed to set core.hooksPath:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("✅ Git hooks installed successfully");
    println!("   core.hooksPath set to .githooks/");
    println!("   Run 'git config --get core.hooksPath' to verify");
    Ok(())
}

/// Get repository root
fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get repository root")?;
    
    if !output.status.success() {
        bail!("git rev-parse --show-toplevel failed");
    }
    
    let root = String::from_utf8_lossy(&output.stdout).trim();
    Ok(PathBuf::from(root))
}

/// Get staged file names from git
fn git_diff_cached_names() -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("Failed to get staged files")?;
    
    if !output.status.success() {
        bail!("git diff --cached --name-only failed");
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let files: Vec<PathBuf> = output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect();
    
    Ok(files)
}

/// L1 TRACEABILITY pattern - matches issue references in commit messages
const L1_PATTERN: &str = r"(?i)(closes?|fixes?|resolves?|refs?|updates?)\s*#(\d+)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l1_pattern_closes() {
        let test_cases = [
            ("Closes #123", true),
            ("closes #123", true),
            ("CLOSES #123", true),
            ("Closes#123", true),
            ("Closes # 123", true),
            ("Fixes #456", true),
            ("fixes #456", true),
            ("Resolves #789", true),
            ("resolves #789", true),
            ("Refs #999", true),
            ("refs #999", true),
            ("Updates #1000", true),
            ("updates #1000", true),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_rejects_bare_issue() {
        let test_cases = [
            ("#123", false),
            ("Fixes #123 and #456", true), // Should still match the first one
            ("No issue here", false),
            ("Just a number 123", false),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_with_colons() {
        let test_cases = [
            ("Closes: #123", true),
            ("Fixes: #456", true),
            ("Resolves: #789", true),
            ("Refs: #999", true),
            ("Updates: #1000", true),
            ("closes: #123", true),
            ("fixes: #456", true),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_continuation_scanning() {
        let test_cases = [
            ("Closes the loop on the design\nResolves #77", true),
            ("Fixes the issue\nUpdates #123", true),
            ("Some text\nRefs #456\nMore text", true),
            ("No references here", false),
            ("Closes #123 but also prose", true),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_case_variations() {
        let test_cases = [
            ("closes #123", true),
            ("Closes #123", true),
            ("CLOSES #123", true),
            ("ClOsEs #123", true),
            ("fixes #456", true),
            ("Fixes #456", true),
            ("FIXES #456", true),
            ("resolves #789", true),
            ("Resolves #789", true),
            ("RESOLVES #789", true),
            ("refs #999", true),
            ("Refs #999", true),
            ("REFS #999", true),
            ("updates #1000", true),
            ("Updates #1000", true),
            ("UPDATES #1000", true),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_plural_forms() {
        let test_cases = [
            ("close #123", true),
            ("Close #123", true),
            ("fix #456", true),
            ("Fix #456", true),
            ("resolve #789", true),
            ("Resolve #789", true),
            ("ref #999", true),
            ("Ref #999", true),
            ("update #1000", true),
            ("Update #1000", true),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_whitespace_handling() {
        let test_cases = [
            ("Closes#123", true),
            ("Closes #123", true),
            ("Closes  #123", true),
            ("Closes\t#123", true),
            ("Fixes#456", true),
            ("Fixes #456", true),
            ("Resolves#789", true),
            ("Resolves #789", true),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let actual = re.is_match(msg);
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_l1_pattern_extract_issue_numbers() {
        let test_cases = [
            ("Closes #123", Some("123")),
            ("Fixes #456", Some("456")),
            ("Resolves #789", Some("789")),
            ("Refs #999", Some("999")),
            ("Updates #1000", Some("1000")),
            ("No match here", None),
        ];
        
        for (msg, expected) in test_cases {
            let re = Regex::new(L1_PATTERN).unwrap();
            let caps = re.captures(msg);
            let actual = caps.and_then(|c| c.get(2).map(|m| m.as_str()));
            assert_eq!(actual, expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_check_commit_message_with_comments() {
        let test_cases = [
            ("# This is a comment\nCloses #123", true),
            ("# Another comment\nFixes #456\n# Yet another comment", true),
            ("# Comment only\nNo reference here", false),
            ("Closes #123\n# Comment after", true),
        ];
        
        for (msg, expected) in test_cases {
            let result = check_commit_message(msg);
            assert_eq!(result.is_ok(), expected, "Failed: {}", msg);
        }
    }

    #[test]
    fn test_check_commit_message_rejects_bare_issue() {
        let test_cases = [
            ("#123", false),
            ("Fixes issue #123", true), // Should still match
            ("Reference to #123 without verb", false),
            ("Closes #123 and also #456", true), // Should match
        ];
        
        for (msg, expected) in test_cases {
            let result = check_commit_message(msg);
            assert_eq!(result.is_ok(), expected, "Failed: {}", msg);
        }
    }
}