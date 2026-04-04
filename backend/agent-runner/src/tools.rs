use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;

use crate::logger;

// ─── Tool Result ─────────────────────────────────────────────────────────────

pub struct ToolResult {
    pub output: String,
    pub is_complete: bool,
    pub duration_ms: u64,
    pub success: bool,
}

// ─── Tool Definitions ────────────────────────────────────────────────────────

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "bash",
            "description": "Execute a shell command and capture stdout and stderr. The command runs in the current working directory. Use this for running tests, installing dependencies, running build commands, git operations, etc.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Optional timeout in seconds (default: 120)"
                    }
                },
                "required": ["command"]
            }
        }),
        json!({
            "name": "read_file",
            "description": "Read the contents of a file from the filesystem.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to read"
                    }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "write_file",
            "description": "Write content to a file, creating parent directories as needed. Overwrites existing files.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }
        }),
        json!({
            "name": "task_complete",
            "description": "Mark the task as complete and provide a summary of what was accomplished. Call this when you have finished the task successfully.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "A detailed summary of what was accomplished"
                    },
                    "files_modified": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of files that were created or modified"
                    }
                },
                "required": ["summary"]
            }
        }),
    ]
}

// ─── Tool Executor ───────────────────────────────────────────────────────────

pub async fn execute_tool(name: &str, input: &Value) -> ToolResult {
    match name {
        "bash" => execute_bash(input).await,
        "read_file" => execute_read_file(input).await,
        "write_file" => execute_write_file(input).await,
        "task_complete" => execute_task_complete(input).await,
        _ => ToolResult {
            output: format!("Unknown tool: {}", name),
            is_complete: false,
            duration_ms: 0,
            success: false,
        },
    }
}

// ─── bash ────────────────────────────────────────────────────────────────────

async fn execute_bash(input: &Value) -> ToolResult {
    let command = match input["command"].as_str() {
        Some(c) => c.to_string(),
        None => {
            return ToolResult {
                output: "Error: 'command' field required".to_string(),
                is_complete: false,
                duration_ms: 0,
                success: false,
            }
        }
    };

    let timeout_secs = input["timeout_seconds"]
        .as_u64()
        .unwrap_or(120);

    let start = Instant::now();

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output(),
    )
    .await;

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Err(_timeout) => {
            let output = format!("Error: Command timed out after {}s\nCommand: {}", timeout_secs, command);
            logger::log_tool_section("bash", &command, None, "", &format!("TIMEOUT after {}s", timeout_secs), duration_ms);
            ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success: false,
            }
        }
        Ok(Err(e)) => {
            let output = format!("Error: Failed to execute command: {}\nCommand: {}", e, command);
            logger::log_tool_section("bash", &command, None, "", &e.to_string(), duration_ms);
            ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success: false,
            }
        }
        Ok(Ok(proc_output)) => {
            let exit_code = proc_output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&proc_output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&proc_output.stderr).to_string();

            logger::log_tool_section("bash", &command, Some(exit_code), &stdout, &stderr, duration_ms);

            let mut output_parts = Vec::new();
            if !stdout.is_empty() {
                output_parts.push(format!("STDOUT:\n{}", stdout));
            }
            if !stderr.is_empty() {
                output_parts.push(format!("STDERR:\n{}", stderr));
            }
            output_parts.push(format!("Exit code: {}", exit_code));

            let output = output_parts.join("\n\n");
            let success = exit_code == 0;

            ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success,
            }
        }
    }
}

// ─── read_file ───────────────────────────────────────────────────────────────

async fn execute_read_file(input: &Value) -> ToolResult {
    let path = match input["path"].as_str() {
        Some(p) => p.to_string(),
        None => {
            return ToolResult {
                output: "Error: 'path' field required".to_string(),
                is_complete: false,
                duration_ms: 0,
                success: false,
            }
        }
    };

    let start = Instant::now();

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let size = content.len();

            logger::log_tool_section(
                "read_file",
                &path,
                Some(0),
                &format!("{} bytes read", size),
                "",
                duration_ms,
            );

            ToolResult {
                output: content,
                is_complete: false,
                duration_ms,
                success: true,
            }
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let output = format!("Error reading file '{}': {}", path, e);

            logger::log_tool_section("read_file", &path, Some(1), "", &e.to_string(), duration_ms);

            ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success: false,
            }
        }
    }
}

// ─── write_file ──────────────────────────────────────────────────────────────

async fn execute_write_file(input: &Value) -> ToolResult {
    let path = match input["path"].as_str() {
        Some(p) => p.to_string(),
        None => {
            return ToolResult {
                output: "Error: 'path' field required".to_string(),
                is_complete: false,
                duration_ms: 0,
                success: false,
            }
        }
    };

    let content = match input["content"].as_str() {
        Some(c) => c.to_string(),
        None => {
            return ToolResult {
                output: "Error: 'content' field required".to_string(),
                is_complete: false,
                duration_ms: 0,
                success: false,
            }
        }
    };

    let start = Instant::now();

    // Create parent directories if needed
    if let Some(parent) = Path::new(&path).parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            let duration_ms = start.elapsed().as_millis() as u64;
            let output = format!("Error creating directories for '{}': {}", path, e);
            logger::log_tool_section("write_file", &path, Some(1), "", &e.to_string(), duration_ms);
            return ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success: false,
            };
        }
    }

    match tokio::fs::write(&path, &content).await {
        Ok(()) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let size = content.len();
            let output = format!("Successfully wrote {} bytes to '{}'", size, path);

            logger::log_tool_section(
                "write_file",
                &path,
                Some(0),
                &format!("{} bytes written", size),
                "",
                duration_ms,
            );

            ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success: true,
            }
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let output = format!("Error writing file '{}': {}", path, e);
            logger::log_tool_section("write_file", &path, Some(1), "", &e.to_string(), duration_ms);

            ToolResult {
                output,
                is_complete: false,
                duration_ms,
                success: false,
            }
        }
    }
}

// ─── task_complete ───────────────────────────────────────────────────────────

async fn execute_task_complete(input: &Value) -> ToolResult {
    let summary = input["summary"]
        .as_str()
        .unwrap_or("Task completed.")
        .to_string();

    let files: Vec<String> = input["files_modified"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let output = if files.is_empty() {
        format!("TASK COMPLETE\n\n{}", summary)
    } else {
        format!(
            "TASK COMPLETE\n\n{}\n\nFiles modified:\n{}",
            summary,
            files.iter().map(|f| format!("  - {}", f)).collect::<Vec<_>>().join("\n")
        )
    };

    logger::log_tool_section("task_complete", "TASK COMPLETE", Some(0), &output, "", 0);

    ToolResult {
        output,
        is_complete: true,
        duration_ms: 0,
        success: true,
    }
}
