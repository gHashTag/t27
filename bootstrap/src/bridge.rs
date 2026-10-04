use chrono::Utc;
use clap::Subcommand;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "net")]
mod client;

// ═══════════════════════════════════════════════════════════════
// tri bridge — OpenCode A2A Bridge for Queen T (Ϯ)
//
// This is a native part of the T27 DNA that connects AGENT T
// to OpenCode agents via the REST API.
// ═══════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum BridgeCommands {
    /// Queen T Command Center — health, sessions, .trinity state
    Status,
    /// List OpenCode sessions
    Sessions,
    /// Create new session for a task (writes task.intent to akashic)
    Create {
        /// Session title / task description
        title: String,
        /// Priority: P0 (critical), P1 (high), P2 (normal)
        #[arg(short, long, default_value = "P1")]
        priority: String,
    },
    /// Send task to agent (writes to akashic, appears in OpenCode Web UI)
    Send {
        /// Session ID (ses_...)
        session_id: String,
        /// Task text
        message: String,
    },
    /// Monitor agent work in real-time
    Watch {
        /// Session ID (ses_...)
        session_id: String,
    },
    /// Read last loop.handoff and show FUTURE OPTIONS
    Handoff,
}

pub fn run_bridge(command: BridgeCommands) -> anyhow::Result<()> {
    let root = find_repo_root()
        .ok_or_else(|| anyhow::anyhow!("Could not find repo root (no specs/ directory)"))?;

    match command {
        #[cfg(feature = "net")]
        BridgeCommands::Status => client::cmd_status(&root),
        #[cfg(feature = "net")]
        BridgeCommands::Sessions => client::cmd_sessions(&root),
        #[cfg(feature = "net")]
        BridgeCommands::Create { title, priority } => client::cmd_create(&root, &title, &priority),
        #[cfg(feature = "net")]
        BridgeCommands::Send {
            session_id,
            message,
        } => client::cmd_send(&root, &session_id, &message),
        #[cfg(feature = "net")]
        BridgeCommands::Watch { session_id } => client::cmd_watch(&root, &session_id),
        BridgeCommands::Handoff => cmd_handoff(&root),
        #[cfg(not(feature = "net"))]
        _ => anyhow::bail!(
            "'bridge' requires 'net' feature (only 'bridge handoff' is offline); \
             rebuild with: cargo build --release -p t27c --features net"
        ),
    }
    Ok(())
}

// ─── Internal Implementation ────────────────────────────────────

fn find_repo_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    for _ in 0..4 {
        if dir.join("specs").is_dir() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
    None
}


fn cmd_handoff(root: &Path) {
    let path = root
        .join(".trinity")
        .join("events")
        .join("akashic-log.jsonl");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Some(line) = content.lines().rev().find(|l| l.contains("loop.handoff")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                println!("{}", "═══ LOOP HANDOFF ═══".bright_yellow().bold());
                if let Some(opts) = v.get("future_options").and_then(|o| o.as_array()) {
                    for (i, opt) in opts.iter().enumerate() {
                        println!(
                            "  {}) {}",
                            i + 1,
                            opt.get("label").and_then(|l| l.as_str()).unwrap_or("?")
                        );
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Task Commands (NotebookLM Gate Enforcement)
//
// Enforces L7 UNITY: every task must have a NotebookLM notebook
// before pushing code. Tracks .notebook_id in git for CI visibility.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Debug)]
struct NotebookMeta {
    notebook_id: String,
    title: String,
    branch: String,
    created_at: String,
    sources: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct NotebookCreateResponse {
    notebook_id: String,
    notebook_url: String,
    title: String,
    created_at: String,
}

const CURRENT_TASK_DIR: &str = ".trinity/current_task";
const NOTEBOOK_ID_FILE: &str = ".notebook_id";
const NOTEBOOK_META_FILE: &str = "notebook_meta.json";

fn handle_task_start(root: &Path, title: &str, sources: &str) {
    let task_dir = root.join(CURRENT_TASK_DIR);
    let id_file = task_dir.join(NOTEBOOK_ID_FILE);
    let meta_file = task_dir.join(NOTEBOOK_META_FILE);

    println!("{}", "═══ TASK INITIALIZATION ═══".bright_yellow().bold());
    println!();

    if id_file.exists() {
        let existing_id = fs::read_to_string(&id_file)
            .unwrap_or_else(|_| "(unreadable)".to_string())
            .trim()
            .to_string();

        if !existing_id.is_empty()
            && !existing_id.starts_with('#')
            && !existing_id.starts_with("//")
        {
            println!(
                "{}",
                format!(
                    "⚠️  Warning: Notebook ID already exists: {}",
                    existing_id.cyan()
                )
                .yellow()
            );
            println!("   Run: t27c task attach --notebook-id <new_id>");
            println!("   Or: rm {} and try again", id_file.display());
            return;
        }
    }

    if let Err(e) = fs::create_dir_all(&task_dir) {
        eprintln!("{} Failed to create {}: {}", "❌".red(), task_dir.display(), e);
        std::process::exit(1);
    }

    // For now, create a manual notebook entry
    let branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Generate a fake notebook ID for now (real implementation needs Python backend)
    let notebook_id = format!("nb-{}", title.to_lowercase().replace(" ", "-").chars().take(12).collect::<String>());

    let meta = NotebookMeta {
        notebook_id: notebook_id.clone(),
        title: title.to_string(),
        branch: branch.clone(),
        created_at: Utc::now().to_rfc3339(),
        sources: if sources.is_empty() {
            Vec::new()
        } else {
            sources.split(',').map(|s| s.trim().to_string()).collect()
        },
    };

    if let Err(e) = fs::write(&id_file, &notebook_id) {
        eprintln!("{} Failed to write {}: {}", "❌".red(), id_file.display(), e);
        std::process::exit(1);
    }

    if let Err(e) = fs::write(
        &meta_file,
        serde_json::to_string_pretty(&meta).unwrap_or_default(),
    ) {
        eprintln!(
            "{} Failed to write {}: {}",
            "❌".red(),
            meta_file.display(),
            e
        );
    }

    println!();
    println!("[OK] NotebookLM notebook created");
    println!();
    println!("   Notebook ID:  {}", notebook_id);
    println!("   Title:         {}", title);
    println!("   Branch:        {}", branch);
    println!();
}
