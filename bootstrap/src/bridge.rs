use chrono::{Local, Utc};
use clap::Subcommand;
use colored::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════
// tri bridge — OpenCode A2A Bridge for Queen T (Ϯ)
//
// This is a native part of the T27 DNA that connects AGENT T
// to OpenCode agents via the REST API.
// ═══════════════════════════════════════════════════════════════

const BASE_URL: &str = "http://127.0.0.1:4096";
const AGENT_ID: &str = "agent-t-antigravity";
const AGENT_SIGN: &str = "[Ϯ AGENT T / Queen Antigravity]";

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
    /// GitHub SSOT commands (Ring-072)
    #[command(subcommand)]
    GitHub(GitHubCommands),
}

#[derive(Subcommand, Debug)]
pub enum GitHubCommands {
    /// Check GitHub authentication status
    AuthStatus,
    /// List GitHub issues
    IssueList {
        #[arg(short, long, default_value = "open")]
        state: String,
        #[arg(short, long, default_value = "5")]
        limit: String,
    },
    /// Create GitHub issue
    IssueCreate {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        body: String,
    },
    /// List GitHub PRs
    PrList {
        #[arg(short, long, default_value = "open")]
        state: String,
        #[arg(short, long, default_value = "5")]
        limit: String,
    },
    /// Create GitHub PR
    PrCreate {
        #[arg(short, long)]
        branch: String,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        body: String,
    },
    /// Run full GitHub ↔ NotebookLM sync
    Sync,
}

pub fn run_bridge(command: BridgeCommands) -> anyhow::Result<()> {
    let root = find_repo_root().ok_or_else(|| anyhow::anyhow!("Could not find repo root (no specs/ directory)"))?;

    match command {
        BridgeCommands::Status => cmd_status(&root),
        BridgeCommands::Sessions => cmd_sessions(&root),
        BridgeCommands::Create { title, priority } => cmd_create(&root, &title, &priority),
        BridgeCommands::Send { session_id, message } => cmd_send(&root, &session_id, &message),
        BridgeCommands::Watch { session_id } => cmd_watch(&root, &session_id),
        BridgeCommands::Handoff => cmd_handoff(&root),
        BridgeCommands::GitHub(github_cmd) => run_github_command(github_cmd),
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

// REST Client types
#[derive(Deserialize)]
struct HealthResponse { healthy: bool, version: String }
#[derive(Deserialize)]
struct Session { id: String, title: Option<String> }
#[derive(Deserialize)]
struct MessageEnvelope { info: MessageInfo, parts: Vec<Part> }
#[derive(Deserialize)]
struct MessageInfo { role: String }
#[derive(Deserialize)]
struct Part {
    #[serde(rename = "type")] part_type: String,
    #[serde(default)] text: Option<String>,
    #[serde(rename = "toolInvocation")] tool_invocation: Option<ToolInvocation>
}
#[derive(Deserialize)]
struct ToolInvocation { #[serde(rename = "toolName")] tool_name: String }
#[derive(Serialize)]
struct CreateSessionRequest { title: String }
#[derive(Serialize)]
struct PromptRequest { parts: Vec<TextPart> }
#[derive(Serialize)]
struct TextPart { #[serde(rename = "type")] part_type: String, text: String }

// GitHub API types
#[derive(Deserialize, Debug)]
struct GitHubIssue {
    number: u32,
    title: String,
    state: String,
    html_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubPR {
    number: u32,
    title: String,
    state: String,
    html_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubAuthStatus {
    #[serde(rename = "login")]
    username: Option<String>,
    #[serde(rename = "oauthScopes")]
    scopes: Option<String>,
}

#[derive(Serialize)]
struct AkashicEvent {
    ts: String,
    event: String,
    agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")] task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] priority: Option<String>,
}

fn append_akashic(root: &Path, event: &AkashicEvent) {
    let path = root.join(".trinity").join("events").join("akashic-log.jsonl");
    fs::create_dir_all(path.parent().unwrap()).ok();
    if let Ok(json) = serde_json::to_string(event) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{}", json);
        }
    }
}

fn url(root: &Path, path: &str) -> String {
    format!("{}{}?directory={}", BASE_URL, path, root.to_string_lossy())
}

fn cmd_status(root: &Path) {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "tri — Queen T Command Center".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let client = Client::new();
    match client.get(format!("{}/global/health", BASE_URL)).send().and_then(|r| r.json::<HealthResponse>()) {
        Ok(h) => println!("  {} OpenCode v{} healthy={}", "✅".green(), h.version.cyan(), h.healthy),
        Err(_) => { println!("  {} OpenCode server unreachable on port 4096", "❌".red()); return; }
    }

    match client.get(url(root, "/session")).send().and_then(|r| r.json::<Vec<Session>>()) {
        Ok(sessions) => {
            println!("\n  {} Sessions:", "📋".bold());
            for s in &sessions {
                let title = s.title.as_deref().unwrap_or("(untitled)");
                println!("    {} {} — {}", "🟢".green(), s.id.bright_black(), title);
            }
        },
        Err(_) => println!("\n  {} Could not list sessions", "❌".red()),
    }
    println!("\n  Web UI: {}", BASE_URL.underline());
}

fn cmd_sessions(root: &Path) {
    let client = Client::new();
    if let Ok(sessions) = client.get(url(root, "/session")).send().and_then(|r| r.json::<Vec<Session>>()) {
        for s in &sessions {
            println!("{} {} — {}", "🟢".green(), s.id, s.title.as_deref().unwrap_or(""));
        }
    }
}

fn cmd_create(root: &Path, title: &str, priority: &str) {
    append_akashic(root, &AkashicEvent {
        ts: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        event: "task.intent".into(),
        agent_id: AGENT_ID.into(),
        task_id: Some(format!("BRIDGE-{}", Local::now().format("%H%M%S"))),
        session_id: None,
        message: Some(title.to_string()),
        priority: Some(priority.to_string()),
    });

    let client = Client::new();
    if let Ok(resp) = client.post(url(root, "/session")).json(&CreateSessionRequest { title: title.to_string() }).send() {
        if let Ok(session) = resp.json::<Session>() {
            println!("{} Created session: {}", "✅".green(), session.id.bold());
        }
    }
}

fn cmd_send(root: &Path, session_id: &str, message: &str) {
    let full_message = format!("{}\n{}", AGENT_SIGN, message);
    let client = Client::new();
    let body = PromptRequest { parts: vec![TextPart { part_type: "text".into(), text: full_message }] };
    
    if let Ok(r) = client.post(url(root, &format!("/session/{}/prompt_async", session_id))).json(&body).send() {
        if r.status().is_success() {
            println!("{} Task dispatched to {}", "✅".green(), session_id);
        } else {
            println!("{} Error: {}", "❌".red(), r.status());
        }
    }
}

fn cmd_watch(root: &Path, session_id: &str) {
    println!("{} Watching {} (Ctrl+C to stop)", "👁".bold(), session_id.cyan());
    let client = Client::new();
    let mut last_count = 0;
    loop {
        if let Ok(r) = client.get(url(root, &format!("/session/{}/message&limit=5", session_id))).send() {
            if let Ok(messages) = r.json::<Vec<MessageEnvelope>>() {
                if messages.len() != last_count {
                    for msg in &messages {
                        for part in &msg.parts {
                            if let Some(text) = &part.text {
                                println!("\n{} [{}]: {}", if msg.info.role == "user" { "👤" } else { "🤖" }, msg.info.role, text);
                            }
                        }
                    }
                    last_count = messages.len();
                }
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn cmd_handoff(root: &Path) {
    let path = root.join(".trinity").join("events").join("akashic-log.jsonl");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Some(line) = content.lines().rev().find(|l| l.contains("loop.handoff")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                println!("{}", "═══ LOOP HANDOFF ═══".bright_yellow().bold());
                if let Some(opts) = v.get("future_options").and_then(|o| o.as_array()) {
                    for (i, opt) in opts.iter().enumerate() {
                        println!("  {}) {}", i + 1, opt.get("label").and_then(|l| l.as_str()).unwrap_or("?"));
                    }
                }
            }
        }
    }
}

// ─── GitHub SSOT Commands (Ring-072) ─────────────────────────────────

fn run_github_command(command: GitHubCommands) {
    match command {
        GitHubCommands::AuthStatus => cmd_github_auth_status(),
        GitHubCommands::IssueList { state, limit } => cmd_github_issue_list(&state, &limit),
        GitHubCommands::IssueCreate { title, body } => cmd_github_issue_create(&title, &body),
        GitHubCommands::PrList { state, limit } => cmd_github_pr_list(&state, &limit),
        GitHubCommands::PrCreate { branch, title, body } => cmd_github_pr_create(&branch, &title, &body),
        GitHubCommands::Sync => cmd_github_sync(),
    }
}

fn cmd_github_auth_status() {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "🔐".bold(), "GitHub Authentication Status".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    match std::process::Command::new("gh")
        .args(&["auth", "status"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stdout.contains("Logged in to") {
                println!("  {} {}", "✅".green(), "Authenticated".bold());
                // Try to get username
                if let Ok(user_output) = std::process::Command::new("gh")
                    .args(&["api", "user", "--jq", ".login"])
                    .output()
                {
                    let user = String::from_utf8_lossy(&user_output.stdout);
                    println!("  {} User: {}", "👤".bold(), user.trim());
                }
            } else {
                println!("  {} {}", "❌".red(), "Not authenticated".bold());
                println!("\n  Run: {}", "gh auth login".cyan());
            }
            if !stderr.is_empty() {
                println!("\n  {} {}", "⚠️".yellow(), stderr.trim());
            }
        }
        Err(_e) => {
            println!("  {} {}", "❌".red(), "gh CLI not found".bold());
            println!("\n  Install: {}", "brew install gh".cyan());
            println!("  Then run: {}", "gh auth login".cyan());
        }
    }
}

fn cmd_github_issue_list(state: &str, limit: &str) {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {} Issues (state={}, limit={})", "📋".bold(), "GitHub".bright_yellow().bold(), state, limit);
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    match std::process::Command::new("gh")
        .args(&["issue", "list", "--state", state, "--limit", limit, "--json", "number,title,state,url"])
        .output()
    {
        Ok(output) => {
            if let Ok(issues) = serde_json::from_str::<Vec<GitHubIssue>>(&String::from_utf8_lossy(&output.stdout)) {
                if issues.is_empty() {
                    println!("  {} No {} issues found", "📭".bold(), state);
                } else {
                    for issue in &issues {
                        println!("  {} #{} — {}", "🟢".green(), issue.number, issue.title);
                        println!("      {} {}", "🔗".bright_black(), issue.html_url);
                        println!("      {} {}", "📊".bright_black(), issue.state.cyan());
                        println!();
                    }
                }
            } else {
                println!("  {} Failed to parse issue list", "❌".red());
            }
        }
        Err(e) => {
            println!("  {} {}", "❌".red(), format!("Error: {}", e));
        }
    }
}

fn cmd_github_issue_create(title: &str, body: &str) {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "📝".bold(), "Creating GitHub Issue".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let mut cmd = std::process::Command::new("gh");
    cmd.args(&["issue", "create", "--title", title]);

    if !body.is_empty() {
        cmd.args(&["--body", body]);
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let url = String::from_utf8_lossy(&output.stdout);
                println!("  {} Issue created: {}", "✅".green(), url.trim().cyan());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("  {} {}", "❌".red(), stderr.trim());
            }
        }
        Err(e) => {
            println!("  {} {}", "❌".red(), format!("Error: {}", e));
        }
    }
}

fn cmd_github_pr_list(state: &str, limit: &str) {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {} PRs (state={}, limit={})", "🔀".bold(), "GitHub".bright_yellow().bold(), state, limit);
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    match std::process::Command::new("gh")
        .args(&["pr", "list", "--state", state, "--limit", limit, "--json", "number,title,state,url"])
        .output()
    {
        Ok(output) => {
            if let Ok(prs) = serde_json::from_str::<Vec<GitHubPR>>(&String::from_utf8_lossy(&output.stdout)) {
                if prs.is_empty() {
                    println!("  {} No {} PRs found", "📭".bold(), state);
                } else {
                    for pr in &prs {
                        println!("  {} #{} — {}", "🟢".green(), pr.number, pr.title);
                        println!("      {} {}", "🔗".bright_black(), pr.html_url);
                        println!("      {} {}", "📊".bright_black(), pr.state.cyan());
                        println!();
                    }
                }
            } else {
                println!("  {} Failed to parse PR list", "❌".red());
            }
        }
        Err(e) => {
            println!("  {} {}", "❌".red(), format!("Error: {}", e));
        }
    }
}

fn cmd_github_pr_create(branch: &str, title: &str, body: &str) {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "🔀".bold(), "Creating GitHub PR".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let mut cmd = std::process::Command::new("gh");
    cmd.args(&["pr", "create", "--base", "master", "--head", branch, "--title", title]);

    if !body.is_empty() {
        cmd.args(&["--body", body]);
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let url = String::from_utf8_lossy(&output.stdout);
                println!("  {} PR created: {}", "✅".green(), url.trim().cyan());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("  {} {}", "❌".red(), stderr.trim());
            }
        }
        Err(e) => {
            println!("  {} {}", "❌".red(), format!("Error: {}", e));
        }
    }
}

fn cmd_github_sync() {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "🔄".bold(), "GitHub ↔ NotebookLM Sync".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let root = find_repo_root().unwrap();
    let start_ms = Local::now().timestamp_millis();

    // Record sync event to akashic
    append_akashic(&root, &AkashicEvent {
        ts: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        event: "github.sync.start".into(),
        agent_id: AGENT_ID.into(),
        task_id: Some(format!("SYNC-{}", Local::now().format("%H%M%S"))),
        session_id: None,
        message: Some("Full GitHub ↔ NotebookLM sync initiated".into()),
        priority: None,
    });

    // For now, this is a placeholder - actual sync would:
    // 1. Call gh issue list / pr list
    // 2. Format as NotebookLM sources
    // 3. Upload via contrib/backend/notebooklm (to be replaced)

    let duration = Local::now().timestamp_millis() - start_ms;

    println!("  {} Sync placeholder (full implementation pending specs/tri/sync.t27)", "⚠️".yellow());
    println!("\n  Duration: {}ms", duration);
    println!("  {} {}", "✅".green(), "Sync event recorded to akashic-log.jsonl");
}

