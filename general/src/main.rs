use chrono::{Local, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════
// tri bridge — OpenCode A2A Bridge for Queen T (Ϯ)
//
// This is a subcommand of `tri` that connects AGENT T (Antigravity)
// to OpenCode agents via the REST API on port 4096.
//
// Follows SOUL.md (Constitution) and coordination-law.md:
// - Events written to .trinity/events/akashic-log.jsonl
// - Claims respected before mutations
// - 6-Phase cycle: plan → assign → run → test → verdict → evolve
// ═══════════════════════════════════════════════════════════════

const BASE_URL: &str = "http://127.0.0.1:4096";
const TRINITY_ROOT: &str = "/Users/playom/t27";
const AGENT_ID: &str = "agent-t-antigravity";
const AGENT_SIGN: &str = "[Ϯ AGENT T / Queen Antigravity]";

// ─── CLI ────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "tri")]
#[command(about = "Ϯ tri — Trinity CLI with A2A Bridge")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    /// Show SOUL.md constitution
    Soul,
    /// Read last loop.handoff and show FUTURE OPTIONS
    Handoff,
}

// ─── API Types ──────────────────────────────────────────────────

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
struct Part {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "toolInvocation")]
    tool_invocation: Option<ToolInvocation>,
}

#[derive(Deserialize)]
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

// ─── Akashic Chronicle ─────────────────────────────────────────

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

fn akashic_log_path() -> String {
    format!("{}/.trinity/events/akashic-log.jsonl", TRINITY_ROOT)
}

fn append_akashic(event: &AkashicEvent) {
    let path = akashic_log_path();
    let dir = Path::new(&path).parent().unwrap();
    fs::create_dir_all(dir).ok();

    if let Ok(json) = serde_json::to_string(event) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{}", json);
        }
    }
}

fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn now_local() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

// ─── HTTP Client ────────────────────────────────────────────────

fn client() -> Client {
    Client::new()
}

fn url(path: &str) -> String {
    format!("{}{}?directory={}", BASE_URL, path, TRINITY_ROOT)
}

fn check_health() -> Result<HealthResponse, String> {
    client()
        .get(format!("{}/global/health", BASE_URL))
        .send()
        .map_err(|e| format!("Server unreachable: {}", e))?
        .json::<HealthResponse>()
        .map_err(|e| format!("Invalid response: {}", e))
}

fn list_sessions() -> Result<Vec<Session>, String> {
    client()
        .get(url("/session"))
        .send()
        .map_err(|e| format!("{}", e))?
        .json::<Vec<Session>>()
        .map_err(|e| format!("{}", e))
}

// ─── Queue State ────────────────────────────────────────────────

fn read_queue_file(name: &str) -> String {
    let path = format!("{}/.trinity/queue/{}", TRINITY_ROOT, name);
    fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string())
}

fn read_last_handoff() -> Option<String> {
    let path = akashic_log_path();
    let content = fs::read_to_string(&path).ok()?;
    content
        .lines()
        .rev()
        .find(|line| line.contains("loop.handoff"))
        .map(|s| s.to_string())
}

// ─── Commands ───────────────────────────────────────────────────

fn cmd_status() {
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

    // Health
    match check_health() {
        Ok(h) => println!(
            "  {} OpenCode v{} healthy={}",
            "✅".green(),
            h.version.cyan(),
            h.healthy
        ),
        Err(e) => {
            println!("  {} {}", "❌".red(), e);
            println!("  {}", "Run: opencode web".bright_black());
            return;
        }
    }
    println!();

    // Sessions
    match list_sessions() {
        Ok(sessions) => {
            println!("  {} Sessions:", "📋".bold());
            for s in &sessions {
                let title = s.title.as_deref().unwrap_or("(untitled)");
                let short: String = title.chars().take(55).collect();
                println!("    {} {}", "🟢".green(), s.id.bright_black());
                println!("       {}", short);
            }
            println!("\n  Total: {}", sessions.len().to_string().cyan().bold());
        }
        Err(e) => println!("  {} {}", "❌".red(), e),
    }
    println!();

    // .trinity state
    println!("  {} .trinity State:", "📁".bold());
    let pending = read_queue_file("pending.json");
    let active = read_queue_file("active.json");
    let blocked = read_queue_file("blocked.json");
    println!("    Queue pending: {}", pending.len().to_string().bright_black());
    println!("    Queue active:  {}", active.len().to_string().bright_black());
    println!("    Queue blocked: {}", blocked.len().to_string().bright_black());

    // Last handoff
    if let Some(handoff) = read_last_handoff() {
        println!();
        println!("  {} Last handoff:", "🔗".bold());
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&handoff) {
            if let Some(opts) = v.get("future_options") {
                if let Some(arr) = opts.as_array() {
                    for (i, opt) in arr.iter().enumerate() {
                        let label = opt.get("label").and_then(|l| l.as_str()).unwrap_or("?");
                        println!("    {}) {}", i + 1, label);
                    }
                }
            }
        }
    }

    println!();
    println!("  Web UI: {}", BASE_URL.underline());
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_yellow()
    );
}

fn cmd_sessions() {
    match list_sessions() {
        Ok(sessions) => {
            for s in &sessions {
                let title = s.title.as_deref().unwrap_or("(untitled)");
                println!("{} {}", "🟢".green(), s.id);
                println!("   {}", title);
                println!();
            }
            println!("Total: {}", sessions.len());
        }
        Err(e) => eprintln!("{} {}", "❌".red(), e),
    }
}

fn cmd_create(title: &str, priority: &str) {
    // 1. Write task.intent to akashic (Coordination Law)
    append_akashic(&AkashicEvent {
        ts: now_utc(),
        event: "task.intent".into(),
        agent_id: AGENT_ID.into(),
        task_id: Some(format!("BRIDGE-{}", Local::now().format("%H%M%S"))),
        session_id: None,
        message: Some(title.to_string()),
        priority: Some(priority.to_string()),
    });

    println!("[{}] 🆕 Creating session: {}", now_local(), title.cyan());

    let resp = client()
        .post(url("/session"))
        .json(&CreateSessionRequest {
            title: title.to_string(),
        })
        .send();

    match resp {
        Ok(r) => match r.json::<Session>() {
            Ok(session) => {
                // 2. Record session creation in akashic
                append_akashic(&AkashicEvent {
                    ts: now_utc(),
                    event: "session.created".into(),
                    agent_id: AGENT_ID.into(),
                    task_id: None,
                    session_id: Some(session.id.clone()),
                    message: Some(title.to_string()),
                    priority: Some(priority.to_string()),
                });

                println!(
                    "[{}] {} Session: {}",
                    now_local(),
                    "✅".green(),
                    session.id.bold()
                );
                println!(
                    "\nNext: tri send {} \"your task\"",
                    session.id.bright_black()
                );
            }
            Err(e) => eprintln!("{} {}", "❌".red(), e),
        },
        Err(e) => eprintln!("{} {}", "❌".red(), e),
    }
}

fn cmd_send(session_id: &str, message: &str) {
    // 1. Record intent in akashic
    append_akashic(&AkashicEvent {
        ts: now_utc(),
        event: "bridge.send".into(),
        agent_id: AGENT_ID.into(),
        task_id: None,
        session_id: Some(session_id.to_string()),
        message: Some(message.to_string()),
        priority: None,
    });

    println!(
        "[{}] {} {} [{}]:",
        now_local(),
        "📤".bold(),
        "Ϯ QUEEN T →".bright_yellow().bold(),
        session_id.bright_black()
    );
    println!("   {}", message);

    // 2. Send with Queen T signature
    let full_message = format!("{}\n{}", AGENT_SIGN, message);
    let body = PromptRequest {
        parts: vec![TextPart {
            part_type: "text".into(),
            text: full_message,
        }],
    };

    let resp = client()
        .post(url(&format!("/session/{}/prompt_async", session_id)))
        .json(&body)
        .send();

    match resp {
        Ok(r) if r.status().is_success() => {
            println!(
                "[{}] {} Task dispatched. Agent working.",
                now_local(),
                "✅".green()
            );
            println!("   Watch: {} or tri watch {}", BASE_URL, session_id);
        }
        Ok(r) => eprintln!("[{}] {} Server: {}", now_local(), "❌".red(), r.status()),
        Err(e) => eprintln!("[{}] {} {}", now_local(), "❌".red(), e),
    }
}

fn cmd_watch(session_id: &str) {
    println!(
        "[{}] {} Watching {} {}",
        now_local(),
        "👁".bold(),
        session_id.cyan(),
        "(Ctrl+C to stop)".bright_black()
    );
    println!("   Also in Web UI: {}", BASE_URL.underline());
    println!();

    let mut last_count: usize = 0;
    loop {
        let resp = client()
            .get(url(&format!(
                "/session/{}/message&limit=10",
                session_id
            )))
            .send();

        if let Ok(r) = resp {
            if let Ok(messages) = r.json::<Vec<MessageEnvelope>>() {
                if messages.len() != last_count {
                    println!("─── {} ───", now_local().bright_black());
                    for msg in &messages {
                        let icon = if msg.info.role == "user" { "👤" } else { "🤖" };
                        for part in &msg.parts {
                            match part.part_type.as_str() {
                                "text" => {
                                    if let Some(text) = &part.text {
                                        let preview: String = text.chars().take(150).collect();
                                        println!(
                                            "  {} [{}] {}",
                                            icon,
                                            msg.info.role.bright_black(),
                                            preview
                                        );
                                    }
                                }
                                "tool-invocation" => {
                                    if let Some(ti) = &part.tool_invocation {
                                        println!(
                                            "  🔧 tool: {}",
                                            ti.tool_name.yellow()
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    println!();
                    last_count = messages.len();
                }
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}

fn cmd_soul() {
    let path = format!("{}/SOUL.md", TRINITY_ROOT);
    match fs::read_to_string(&path) {
        Ok(content) => println!("{}", content),
        Err(_) => eprintln!("{} SOUL.md not found", "❌".red()),
    }
}

fn cmd_handoff() {
    match read_last_handoff() {
        Some(line) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                println!("{}", "═══ LOOP HANDOFF ═══".bright_yellow().bold());
                if let Some(past) = v.get("past").and_then(|p| p.get("summary")).and_then(|s| s.as_str()) {
                    println!("[PAST]    {}", past);
                }
                if let Some(present) = v.get("present").and_then(|p| p.get("summary")).and_then(|s| s.as_str()) {
                    println!("[PRESENT] {}", present);
                }
                println!("[FUTURE OPTIONS]");
                if let Some(opts) = v.get("future_options").and_then(|o| o.as_array()) {
                    for (i, opt) in opts.iter().enumerate() {
                        let label = opt.get("label").and_then(|l| l.as_str()).unwrap_or("?");
                        let prio = opt.get("priority").and_then(|p| p.as_str()).unwrap_or("?");
                        println!("  {}) [{}] {}", i + 1, prio, label);
                    }
                }
            } else {
                println!("{}", line);
            }
        }
        None => println!("No loop.handoff found in akashic-log.jsonl"),
    }
}

// ─── Main ───────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => cmd_status(),
        Commands::Sessions => cmd_sessions(),
        Commands::Create { title, priority } => cmd_create(&title, &priority),
        Commands::Send { session_id, message } => cmd_send(&session_id, &message),
        Commands::Watch { session_id } => cmd_watch(&session_id),
        Commands::Soul => cmd_soul(),
        Commands::Handoff => cmd_handoff(),
    }
}
