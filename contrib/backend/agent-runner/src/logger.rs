use crate::agent::AgentReport;
use crate::api::ApiResponse;
use chrono::Utc;
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::OnceLock;

static LOG_FILE: OnceLock<String> = OnceLock::new();

pub fn init_log_file(path: &str) {
    LOG_FILE.set(path.to_string()).ok();
}

fn write_jsonl(event: &Value) {
    let path = LOG_FILE.get().map(|s| s.as_str()).unwrap_or("/tmp/agent-log.jsonl");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", event);
    }
}

fn jsonl_event(event_type: &str, data: Value) -> Value {
    json!({
        "ts": Utc::now().to_rfc3339(),
        "event": event_type,
        "data": data,
    })
}

// ─── Step Logging (new) ───────────────────────────────────────────────────────

/// Print a full step banner with key-value detail rows.
///
/// ```
/// ╔══════════════════════════════════════════════════════╗
/// ║  STEP 1/10: GITHUB AUTHENTICATION                    ║
/// ╠══════════════════════════════════════════════════════╣
/// ║  GH_TOKEN: set (32 chars)                            ║
/// ╚══════════════════════════════════════════════════════╝
/// ```
pub fn log_step(step: u32, total: u32, name: &str, details: &[(&str, &str)]) {
    let width = 54usize;
    let line = "═".repeat(width);
    let header = format!("  STEP {}/{}: {}  ", step, total, name);
    let header_pad = if width + 2 > header.len() { width + 2 - header.len() } else { 0 };

    println!();
    println!("╔{}╗", line);
    println!("║{}{}║", header, " ".repeat(header_pad));
    println!("╠{}╣", line);
    for (key, val) in details {
        let row = format!("  {}: {}", key, val);
        let pad = if width + 2 > row.len() { width + 2 - row.len() } else { 0 };
        println!("║{}{}║", row, " ".repeat(pad));
    }
    println!("╚{}╝", line);

    write_jsonl(&jsonl_event("step_start", json!({
        "step": step,
        "total": total,
        "name": name,
        "details": details.iter().map(|(k,v)| json!({"key": k, "value": v})).collect::<Vec<_>>(),
    })));
}

/// Print the result line for a step.
pub fn log_step_result(step: u32, success: bool, duration_ms: u64, message: &str) {
    let icon = if success { "✓" } else { "✗" };
    let width = 54usize;
    let row = format!("  STEP {} {} — {} ({}ms)", step, icon, message, duration_ms);
    let pad = if width + 2 > row.len() { width + 2 - row.len() } else { 0 };
    println!("╔{}╗", "═".repeat(width));
    println!("║{}{}║", row, " ".repeat(pad));
    println!("╚{}╝", "═".repeat(width));
    println!();

    write_jsonl(&jsonl_event("step_result", json!({
        "step": step,
        "success": success,
        "duration_ms": duration_ms,
        "message": message,
    })));
}

// ─── Banner / Turn / API ──────────────────────────────────────────────────────

pub fn log_banner(text: &str) {
    let width = 62;
    let inner = format!("  {}  ", text);
    let pad = if width > inner.len() { width - inner.len() } else { 0 };
    let left = pad / 2;
    let right = pad - left;
    let line = "═".repeat(width);
    println!();
    println!("╔{}╗", line);
    println!("║{}{}{}║", " ".repeat(left), inner, " ".repeat(right));
    println!("╚{}╝", line);
    println!();

    write_jsonl(&jsonl_event("banner", json!({ "text": text })));
}

pub fn log_turn_start(turn: u32, max_turns: u32, msg_count: usize, total_tokens: u64) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  TURN {}/{:<51} ║", turn, max_turns);
    println!("║  Messages: {}  |  Total tokens: {:<27} ║",
        msg_count,
        format_number(total_tokens)
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    write_jsonl(&jsonl_event("turn_start", json!({
        "turn": turn,
        "max_turns": max_turns,
        "message_count": msg_count,
        "total_tokens": total_tokens,
    })));
}

pub fn log_api_request(model: &str, system_len: usize, messages_count: usize) {
    println!("║  Sending request...                                          ║");
    println!("║  Model: {:<52} ║", truncate(model, 52));
    println!("║  System prompt: {:<44} ║", format!("{} chars", system_len));
    println!("║  User messages: {:<44} ║", messages_count);
    println!("╠══════════════════════════════════════════════════════════════╣");

    write_jsonl(&jsonl_event("api_request", json!({
        "model": model,
        "system_len": system_len,
        "messages_count": messages_count,
    })));
}

pub fn log_api_response(response: &ApiResponse, duration_ms: u64) {
    let duration_s = duration_ms as f64 / 1000.0;
    let text_blocks = response.content.iter().filter(|b| matches!(b, crate::api::ContentBlock::Text { .. })).count();
    let tool_blocks = response.content.iter().filter(|b| matches!(b, crate::api::ContentBlock::ToolUse { .. })).count();
    let thinking_blocks = response.content.iter().filter(|b| matches!(b, crate::api::ContentBlock::Thinking { .. })).count();

    let mut block_desc = format!("{} total", response.content.len());
    let mut parts = Vec::new();
    if text_blocks > 0 { parts.push(format!("{} text", text_blocks)); }
    if tool_blocks > 0 { parts.push(format!("{} tool_use", tool_blocks)); }
    if thinking_blocks > 0 { parts.push(format!("{} thinking", thinking_blocks)); }
    if !parts.is_empty() {
        block_desc = format!("{} ({})", response.content.len(), parts.join(", "));
    }

    println!("║  Response received in {:.1}s{:<37} ║", duration_s, "");
    println!("║  Model returned: {:<43} ║", truncate(&response.model, 43));
    println!("║  Tokens: input={} output={:<32} ║",
        format_number(response.usage.input_tokens as u64),
        format_number(response.usage.output_tokens as u64)
    );
    println!("║  Stop reason: {:<46} ║",
        response.stop_reason.as_deref().unwrap_or("none")
    );
    println!("║  Content blocks: {:<43} ║", truncate(&block_desc, 43));
    println!("╠══════════════════════════════════════════════════════════════╣");

    write_jsonl(&jsonl_event("api_response", json!({
        "id": response.id,
        "model": response.model,
        "stop_reason": response.stop_reason,
        "input_tokens": response.usage.input_tokens,
        "output_tokens": response.usage.output_tokens,
        "content_blocks": response.content.len(),
        "text_blocks": text_blocks,
        "tool_blocks": tool_blocks,
        "duration_ms": duration_ms,
    })));
}

pub fn log_text_block(text: &str) {
    println!("║  TEXT:                                                       ║");
    for line in text.lines().take(20) {
        let chunks = chunk_text(line, 56);
        for chunk in &chunks {
            println!("║    {:<58} ║", chunk);
        }
    }
    if text.lines().count() > 20 {
        println!("║    ... [{} more lines]                                       ║",
            text.lines().count() - 20);
    }
    println!("╠══════════════════════════════════════════════════════════════╣");

    write_jsonl(&jsonl_event("text_block", json!({ "text": text })));
}

pub fn log_thinking_block(text: &str) {
    println!("║  THINKING:                                                   ║");
    let preview: String = text.chars().take(200).collect();
    for line in preview.lines().take(5) {
        println!("║    {:<58} ║", truncate(line, 58));
    }
    if text.len() > 200 {
        println!("║    ... [{} chars total]                                      ║", text.len());
    }
    println!("╠══════════════════════════════════════════════════════════════╣");

    write_jsonl(&jsonl_event("thinking_block", json!({ "text_preview": &text[..text.len().min(500)] })));
}

pub fn log_tool_call(step: u32, name: &str, input_summary: &str) {
    println!("║  TOOL CALL #{}: {:<46} ║", step, name);
    for line in input_summary.lines().take(10) {
        println!("║    {:<58} ║", truncate(line, 58));
    }

    write_jsonl(&jsonl_event("tool_call", json!({
        "step": step,
        "tool": name,
        "input_summary": input_summary,
    })));
}

pub fn log_tool_result(name: &str, output_summary: &str, duration_ms: u64, success: bool) {
    let size_info = if output_summary.len() > 1024 {
        format!(" [{} bytes]", output_summary.len())
    } else {
        String::new()
    };
    println!("║  TOOL RESULT: {} ({}ms){}{:<25} ║",
        name,
        duration_ms,
        size_info,
        ""
    );
    for line in output_summary.lines().take(15) {
        println!("║    {:<58} ║", truncate(line, 58));
    }
    let total_lines = output_summary.lines().count();
    if total_lines > 15 {
        println!("║    ... [{} more lines]                                       ║",
            total_lines - 15);
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    write_jsonl(&jsonl_event("tool_result", json!({
        "tool": name,
        "output_summary": &output_summary[..output_summary.len().min(2000)],
        "duration_ms": duration_ms,
        "success": success,
    })));
}

pub fn log_tool_section(name: &str, command: &str, exit_code: Option<i32>, stdout: &str, stderr: &str, duration_ms: u64) {
    println!("[TOOL:{}] ─────────────────────────────────────────────────────", name);
    if !command.is_empty() {
        println!("  Command: {}", command);
    }
    if let Some(code) = exit_code {
        println!("  Exit code: {}", code);
    }
    let stdout_bytes = stdout.len();
    if stdout_bytes > 0 {
        println!("  Stdout ({} bytes):", stdout_bytes);
        for line in stdout.lines().take(50) {
            println!("    {}", line);
        }
        if stdout.lines().count() > 50 {
            println!("    ... [{} more lines]", stdout.lines().count() - 50);
        }
    }
    let stderr_bytes = stderr.len();
    if stderr_bytes > 0 {
        println!("  Stderr ({} bytes):", stderr_bytes);
        for line in stderr.lines().take(20) {
            println!("    {}", line);
        }
    }
    println!("  Duration: {}ms", duration_ms);
    println!("────────────────────────────────────────────────────────────────");
    println!();

    write_jsonl(&jsonl_event("tool_exec", json!({
        "tool": name,
        "command": command,
        "exit_code": exit_code,
        "stdout_bytes": stdout_bytes,
        "stderr_bytes": stderr_bytes,
        "stdout_preview": &stdout[..stdout.len().min(2000)],
        "stderr_preview": &stderr[..stderr.len().min(500)],
        "duration_ms": duration_ms,
    })));
}

pub fn log_agent_complete(report: &AgentReport) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    AGENT RUN COMPLETE                        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Status:        {:<44} ║",
        if report.task_completed { "TASK COMPLETED ✓" } else { "MAX TURNS REACHED" }
    );
    println!("║  Total turns:   {:<44} ║", report.turns);
    println!("║  Duration:      {:<44} ║", format!("{:.1}s", report.duration_seconds));
    println!("║  Input tokens:  {:<44} ║", format_number(report.total_input_tokens));
    println!("║  Output tokens: {:<44} ║", format_number(report.total_output_tokens));
    println!("║  Total tokens:  {:<44} ║", format_number(report.total_input_tokens + report.total_output_tokens));
    println!("║  Tools called:  {:<44} ║", report.tools_called.len());
    println!("║  Files modified:{:<44} ║", report.files_modified.len());
    println!("╠══════════════════════════════════════════════════════════════╣");
    if !report.summary.is_empty() {
        println!("║  Summary:                                                    ║");
        for line in crate::config::wrap_text(&report.summary, 58).iter().take(10) {
            println!("║    {:<58} ║", line);
        }
        println!("╠══════════════════════════════════════════════════════════════╣");
    }
    if !report.files_modified.is_empty() {
        println!("║  Files modified:                                             ║");
        for f in &report.files_modified {
            println!("║    {:<58} ║", truncate(f, 58));
        }
        println!("╠══════════════════════════════════════════════════════════════╣");
    }
    if !report.tools_called.is_empty() {
        let tool_summary = count_tools(&report.tools_called);
        println!("║  Tool usage:                                                 ║");
        for (tool, count) in &tool_summary {
            println!("║    {}: {:<54} ║", tool, count);
        }
        println!("╠══════════════════════════════════════════════════════════════╣");
    }
    println!("╚══════════════════════════════════════════════════════════════╝");

    write_jsonl(&jsonl_event("agent_complete", json!({
        "task_completed": report.task_completed,
        "turns": report.turns,
        "duration_seconds": report.duration_seconds,
        "total_input_tokens": report.total_input_tokens,
        "total_output_tokens": report.total_output_tokens,
        "tools_called": report.tools_called,
        "files_modified": report.files_modified,
        "summary": report.summary,
    })));
}

pub fn log_error(context: &str, err: &str) {
    println!("[ERROR] {}: {}", context, err);
    write_jsonl(&jsonl_event("error", json!({
        "context": context,
        "error": err,
    })));
}

pub fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
    write_jsonl(&jsonl_event("info", json!({ "message": msg })));
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max_len.saturating_sub(3))
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}...", &s[..end])
    }
}

fn chunk_text(s: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in s.chars() {
        current.push(ch);
        if current.chars().count() >= width {
            chunks.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        chunks.push(String::new());
    }
    chunks
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

fn count_tools(tools: &[String]) -> Vec<(String, usize)> {
    let mut map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for t in tools {
        *map.entry(t.clone()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(String, usize)> = map.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    pairs
}
