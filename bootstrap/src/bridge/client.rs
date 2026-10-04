// HTTP half of `t27c bridge`: every subcommand that talks to the OpenCode
// REST API on localhost. Compiled only with the `net` cargo feature, so the
// default compiler build links no HTTP client; `bridge handoff` reads a local
// file and stays in `bridge.rs`.

use chrono::{Local, Utc};
use colored::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

const BASE_URL: &str = "http://127.0.0.1:4096";
const AGENT_ID: &str = "agent-t-antigravity";
const AGENT_SIGN: &str = "[Ϯ AGENT T / Queen Antigravity]";

// REST Client types
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

pub(super) fn cmd_status(root: &Path) {
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

pub(super) fn cmd_sessions(root: &Path) {
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

pub(super) fn cmd_create(root: &Path, title: &str, priority: &str) {
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

pub(super) fn cmd_send(root: &Path, session_id: &str, message: &str) {
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

pub(super) fn cmd_watch(root: &Path, session_id: &str) {
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
