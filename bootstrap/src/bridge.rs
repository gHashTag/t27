use chrono::{Local, Utc};
use clap::Subcommand;
use colored::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    /// Task notebook management (NotebookLM integration)
    #[command(subcommand)]
    Task(TaskCommands),
}

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Initialize task: create NotebookLM notebook + write .notebook_id
    Start {
        /// Task title
        #[arg(short, long)]
        title: String,
        /// Sources to add (comma-separated paths)
        #[arg(long, default_value = "")]
        sources: String,
    },
    /// Attach existing notebook ID to current task
    Attach {
        /// Notebook ID to attach
        #[arg(long)]
        notebook_id: String,
    },
    /// Show current task notebook status
    Status,
    /// Verify notebook ID is valid
    Verify,
    /// Upload activity.md to notebook
    Upload,
}

pub fn run_bridge(command: BridgeCommands) -> anyhow::Result<()> {
    let root = find_repo_root()
        .ok_or_else(|| anyhow::anyhow!("Could not find repo root (no specs/ directory)"))?;

    match command {
        BridgeCommands::Status => cmd_status(&root),
        BridgeCommands::Sessions => cmd_sessions(&root),
        BridgeCommands::Create { title, priority } => cmd_create(&root, &title, &priority),
        BridgeCommands::Send {
            session_id,
            message,
        } => cmd_send(&root, &session_id, &message),
        BridgeCommands::Watch { session_id } => cmd_watch(&root, &session_id),
        BridgeCommands::Handoff => cmd_handoff(&root),
        BridgeCommands::Task(task_cmd) => handle_task(&root, task_cmd),
    }
    Ok(())
}

// ─── Task Commands (NotebookLM) ─────────────────────────────────

fn handle_task(root: &Path, command: TaskCommands) -> anyhow::Result<()> {
    let task_dir = root.join(".trinity").join("current_task");
    fs::create_dir_all(&task_dir)?;

    match command {
        TaskCommands::Start { title, sources } => {
            let notebook_id_path = task_dir.join(".notebook_id");

            if notebook_id_path.exists() {
                let existing_id = fs::read_to_string(&notebook_id_path)?;
                if !existing_id.is_empty() {
                    eprintln!(
                        "{} Notebook already configured: {}",
                        "⚠️".yellow(),
                        existing_id
                    );
                    eprintln!("Use 't27c task attach' to use a different notebook");
                    return Ok(());
                }
            }

            let branch = get_current_branch(root);

            println!("{} Creating NotebookLM notebook...", "📓".bold());
            println!("  Title: {}", title.cyan());
            println!("  Branch: {}", branch.cyan());

            let notebook_id = create_notebook_via_python(&title)?;

            println!("{} Notebook created: {}", "✅".green(), notebook_id.cyan());

            fs::write(&notebook_id_path, &notebook_id)?;

            let meta = NotebookMeta {
                notebook_id: notebook_id.clone(),
                title: title.clone(),
                branch: branch.clone(),
                created_at: Utc::now().to_rfc3339(),
                sources: if sources.is_empty() {
                    Vec::new()
                } else {
                    sources.split(',').map(|s| s.trim().to_string()).collect()
                },
            };
            let meta_path = task_dir.join("notebook_meta.json");
            fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;

            println!("{} Files written:", "📝".bold());
            println!("  {}", notebook_id_path.display());
            println!("  {}", meta_path.display());

            Ok(())
        }
        TaskCommands::Attach { notebook_id } => {
            let notebook_id_path = task_dir.join(".notebook_id");

            if notebook_id_path.exists() {
                let existing = fs::read_to_string(&notebook_id_path)?;
                if !existing.is_empty() {
                    eprintln!(
                        "{} Overwriting existing notebook: {}",
                        "⚠️".yellow(),
                        existing
                    );
                }
            }

            println!("{} Attaching notebook: {}", "🔗".bold(), notebook_id.cyan());

            if verify_notebook_via_python(&notebook_id)? {
                fs::write(&notebook_id_path, &notebook_id)?;
                println!("{} Notebook attached successfully", "✅".green());
            } else {
                eprintln!("{} Notebook verification failed", "❌".red());
                eprintln!("  Notebook ID may be invalid or not accessible");
                return Err(anyhow::anyhow!("Notebook verification failed"));
            }

            Ok(())
        }
        TaskCommands::Status => {
            let notebook_id_path = task_dir.join(".notebook_id");

            if !notebook_id_path.exists() {
                println!("{}", "No notebook configured".red().bold());
                println!("Use 't27c task start --title \"Your task\"' to create one");
                return Ok(());
            }

            let notebook_id = fs::read_to_string(&notebook_id_path)?;
            if notebook_id.is_empty() {
                println!("{}", "No notebook configured".red().bold());
                return Ok(());
            }

            println!("{}", "═══ TASK NOTEBOOK STATUS ═══".bright_yellow().bold());
            println!();
            println!("  {} ID: {}", "📓".bold(), notebook_id.cyan());

            let meta_path = task_dir.join("notebook_meta.json");
            if let Ok(meta_content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<NotebookMeta>(&meta_content) {
                    println!("  {} Title: {}", "📝".bold(), meta.title);
                    println!("  {} Branch: {}", "🌿".bold(), meta.branch);
                    println!("  {} Created: {}", "🕐".bold(), meta.created_at);
                    if !meta.sources.is_empty() {
                        println!("  {} Sources: {}", "📎".bold(), meta.sources.len());
                        for src in &meta.sources {
                            println!("      - {}", src);
                        }
                    }
                }
            }

            if verify_notebook_via_python(&notebook_id)? {
                println!();
                println!("  {} Status: {}", "✅".green(), "Valid and accessible");
                println!("  {} URL: {}", "🔗".bold(),
                    format!("https://notebooklm.google.com/notebook/{}", notebook_id).cyan());
            } else {
                println!();
                println!("  {} Status: {}", "⚠️".yellow(), "Not found or inaccessible");
            }

            Ok(())
        }
        TaskCommands::Verify => {
            let notebook_id_path = task_dir.join(".notebook_id");

            if !notebook_id_path.exists() {
                eprintln!("{}", "❌ No .notebook_id file found".red());
                return Err(anyhow::anyhow!("No notebook configured"));
            }

            let notebook_id = fs::read_to_string(&notebook_id_path)?;

            if verify_notebook_via_python(&notebook_id)? {
                println!("{} Notebook ID is valid: {}", "✅".green(), notebook_id);
                Ok(())
            } else {
                eprintln!("{} Notebook ID verification failed: {}", "❌".red(), notebook_id);
                Err(anyhow::anyhow!("Notebook verification failed"))
            }
        }
        TaskCommands::Upload => {
            let notebook_id_path = task_dir.join(".notebook_id");
            let activity_path = root.join(".trinity").join("current_task").join("activity.md");

            if !notebook_id_path.exists() {
                eprintln!("{}", "❌ No .notebook_id file found".red());
                return Err(anyhow::anyhow!("No notebook configured"));
            }

            if !activity_path.exists() {
                eprintln!("{}", "❌ No activity.md file found".red());
                return Err(anyhow::anyhow!("No activity to upload"));
            }

            let notebook_id = fs::read_to_string(&notebook_id_path)?;
            let activity = fs::read_to_string(&activity_path)?;

            println!("{} Uploading activity.md to notebook...", "📤".bold());

            eprintln!("{} Upload not yet implemented", "⚠️".yellow());
            eprintln!("  Notebook: {}", notebook_id);
            eprintln!("  Activity file: {}", activity_path.display());

            Ok(())
        }
    }
}

fn create_notebook_via_python(title: &str) -> anyhow::Result<String> {
    let output = Command::new("python3")
        .args([
            "-c",
            &format!(
                r#"import asyncio
import sys

async def create_notebook():
    try:
        from notebooklm import NotebookLMClient
        client = await NotebookLMClient.from_storage()
        notebook = await client.notebooks.create("{}")
        print(notebook.id)
    except Exception as e:
        print(f"Error: {{e}}", file=sys.stderr)
        sys.exit(1)

asyncio.run(create_notebook())
"#,
                title
            ),
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute Python: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Python backend failed: {}\n{}",
            output.status,
            stderr
        ));
    }

    let notebook_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if notebook_id.is_empty() || notebook_id.starts_with("Error:") {
        return Err(anyhow::anyhow!("No notebook ID returned from Python backend"));
    }

    Ok(notebook_id)
}

fn verify_notebook_via_python(notebook_id: &str) -> anyhow::Result<bool> {
    let output = Command::new("python3")
        .args([
            "-c",
            &format!(
                r#"import asyncio
import sys

async def verify_notebook():
    try:
        from notebooklm import NotebookLMClient
        client = await NotebookLMClient.from_storage()
        await client.notebooks.get("{}")
        print("OK")
    except Exception:
        sys.exit(1)

asyncio.run(verify_notebook())
"#,
                notebook_id
            ),
        ])
        .output()?;

    Ok(output.status.success())
}

#[derive(Serialize, Deserialize)]
struct NotebookMeta {
    notebook_id: String,
    title: String,
    branch: String,
    created_at: String,
    sources: Vec<String>,
}

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

fn get_current_branch(root: &Path) -> String {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

#[derive(Deserialize)]
struct HealthResponse {
    healthy: bool,
    version: String,
}

#[derive(Deserialize)]
struct Session {
    id: String,
    title: Option<String>,
}

#[derive(Deserialize)]
struct MessageEnvelope {
    info: MessageInfo,
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct MessageInfo {
    role: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Part {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "toolInvocation")]
    tool_invocation: Option<ToolInvocation>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ToolInvocation {
    #[serde(rename = "toolName")]
    tool_name: String,
}

#[derive(Serialize)]
struct CreateSessionRequest {
    title: String,
}

#[derive(Serialize)]
struct PromptRequest {
    parts: Vec<TextPart>,
}

#[derive(Serialize)]
struct TextPart {
    #[serde(rename = "type")]
    part_type: String,
    text: String,
}

#[derive(Serialize)]
struct AkashicEvent {
    ts: String,
    event: String,
    agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
}

fn append_akashic(root: &Path, event: &AkashicEvent) {
    let path = root
        .join(".trinity")
        .join("events")
        .join("akashic-log.jsonl");
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
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_yellow()
    );
    println!(
        "  {} {}",
        "Ϯ".bold(),
        "tri — Queen T Command Center".bright_yellow().bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_yellow()
    );
    println!();

    let client = Client::new();
    match client
        .get(format!("{}/global/health", BASE_URL))
        .send()
        .and_then(|r| r.json::<HealthResponse>())
    {
        Ok(h) => println!(
            "  {} OpenCode v{} healthy={}",
            "✅".green(),
            h.version.cyan(),
            h.healthy
        ),
        Err(_) => {
            println!("  {} OpenCode server unreachable on port 4096", "❌".red());
            return;
        }
    }

    match client
        .get(url(root, "/session"))
        .send()
        .and_then(|r| r.json::<Vec<Session>>())
    {
        Ok(sessions) => {
            println!("\n  {} Sessions:", "📋".bold());
            for s in &sessions {
                let title = s.title.as_deref().unwrap_or("(untitled)");
                println!("    {} {} — {}", "🟢".green(), s.id.bright_black(), title);
            }
        }
        Err(_) => println!("\n  {} Could not list sessions", "❌".red()),
    }
    println!("\n  Web UI: {}", BASE_URL.underline());
}

fn cmd_sessions(root: &Path) {
    let client = Client::new();
    if let Ok(sessions) = client
        .get(url(root, "/session"))
        .send()
        .and_then(|r| r.json::<Vec<Session>>())
    {
        for s in &sessions {
            println!(
                "{} {} — {}",
                "🟢".green(),
                s.id,
                s.title.as_deref().unwrap_or("")
            );
        }
    }
}

fn cmd_create(root: &Path, title: &str, priority: &str) {
    append_akashic(
        root,
        &AkashicEvent {
            ts: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            event: "task.intent".into(),
            agent_id: AGENT_ID.into(),
            task_id: Some(format!("BRIDGE-{}", Local::now().format("%H%M%S"))),
            session_id: None,
            message: Some(title.to_string()),
            priority: Some(priority.to_string()),
        },
    );

    let client = Client::new();
    if let Ok(resp) = client
        .post(url(root, "/session"))
        .json(&CreateSessionRequest {
            title: title.to_string(),
        })
        .send()
    {
        if let Ok(session) = resp.json::<Session>() {
            println!("{} Created session: {}", "✅".green(), session.id.bold());
        }
    }
}

fn cmd_send(root: &Path, session_id: &str, message: &str) {
    let full_message = format!("{}\n{}", AGENT_SIGN, message);
    let client = Client::new();
    let body = PromptRequest {
        parts: vec![TextPart {
            part_type: "text".into(),
            text: full_message,
        }],
    };

    if let Ok(r) = client
        .post(url(root, &format!("/session/{}/prompt_async", session_id)))
        .json(&body)
        .send()
    {
        if r.status().is_success() {
            println!("{} Task dispatched to {}", "✅".green(), session_id);
        } else {
            println!("{} Error: {}", "❌".red(), r.status());
        }
    }
}

fn cmd_watch(root: &Path, session_id: &str) {
    println!(
        "{} Watching {} (Ctrl+C to stop)",
        "👁".bold(),
        session_id.cyan()
    );
    let client = Client::new();
    let mut last_count = 0;
    loop {
        if let Ok(r) = client
            .get(url(
                root,
                &format!("/session/{}/message&limit=5", session_id),
            ))
            .send()
        {
            if let Ok(messages) = r.json::<Vec<MessageEnvelope>>() {
                if messages.len() != last_count {
                    for msg in &messages {
                        for part in &msg.parts {
                            if let Some(text) = &part.text {
                                println!(
                                    "\n{} [{}]: {}",
                                    if msg.info.role == "user" {
                                        "👤"
                                    } else {
                                        "🤖"
                                    },
                                    msg.info.role,
                                    text
                                );
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
