use anyhow::{Context, Result};
use std::time::Instant;

use crate::api::{AnthropicClient, ContentBlock, Message, MessageContent};
use crate::config::Config;
use crate::logger;
use crate::tools;

// ─── Agent Report ─────────────────────────────────────────────────────────────

pub struct AgentReport {
    pub turns: u32,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub tools_called: Vec<String>,
    pub files_modified: Vec<String>,
    pub task_completed: bool,
    pub summary: String,
    pub duration_seconds: f64,
}

// ─── System Prompt Builder ────────────────────────────────────────────────────

async fn build_system_prompt(config: &Config) -> String {
    let mut parts = Vec::new();

    // Try to load SOUL.md from cwd or /workspace
    let soul_paths = [
        "SOUL.md",
        "/workspace/SOUL.md",
        "./SOUL.md",
    ];

    for soul_path in &soul_paths {
        if let Ok(soul_content) = tokio::fs::read_to_string(soul_path).await {
            logger::log_info(&format!("Loaded SOUL.md from {} ({} chars)", soul_path, soul_content.len()));
            parts.push(format!(
                "# Project Constitution (SOUL.md)\n\n{}",
                soul_content
            ));
            break;
        }
    }

    // Core system prompt
    parts.push(format!(
        r#"# T27 Agent Runner

You are an autonomous software engineering agent. You have been given a task to complete.

## Your Capabilities

You have access to the following tools:
- `bash` — Execute shell commands (git, npm, cargo, python, etc.)
- `read_file` — Read file contents
- `write_file` — Write or overwrite files
- `task_complete` — Signal task completion with a summary

## Working Guidelines

1. **Be systematic**: Start by understanding the codebase structure before making changes
2. **Verify your work**: After making changes, run tests or checks to verify correctness
3. **Log your reasoning**: Explain what you're doing and why before each action
4. **Handle errors gracefully**: If a command fails, read the error and adapt
5. **Complete the full task**: Don't stop until the task is truly done

## Current Task

{}

## Important Notes

- You are running in an autonomous environment — there is no human to ask for help
- Make your best judgment and proceed confidently
- Use `task_complete` when you have finished ALL aspects of the task
- Commit your changes using git if appropriate
"#,
        config.task_prompt
    ));

    parts.join("\n\n---\n\n")
}

// ─── Main Agent Loop ──────────────────────────────────────────────────────────

pub async fn run_agent(config: &Config) -> Result<AgentReport> {
    let agent_start = Instant::now();

    logger::log_banner("T27 AUTONOMOUS AGENT STARTING");

    // Build system prompt
    let system_prompt = build_system_prompt(config).await;
    logger::log_info(&format!("System prompt: {} chars", system_prompt.len()));

    // Initialize API client
    let client = AnthropicClient::new(config)
        .context("Failed to create API client")?;

    // Tool definitions
    let tool_defs = tools::tool_definitions();

    // Message history
    let mut messages: Vec<Message> = Vec::new();

    // Add the initial user message
    messages.push(Message {
        role: "user".to_string(),
        content: MessageContent::Text(format!(
            "Please complete the following task:\n\n{}",
            config.task_prompt
        )),
    });

    // Report tracking
    let mut total_input_tokens: u64 = 0;
    let mut total_output_tokens: u64 = 0;
    let mut tools_called: Vec<String> = Vec::new();
    let mut files_modified: Vec<String> = Vec::new();
    let mut task_completed = false;
    let mut completion_summary = String::new();
    let mut turns_completed = 0u32;

    // ── Turn loop ──────────────────────────────────────────────────────────────

    for turn in 1..=config.max_turns {
        turns_completed = turn;

        logger::log_turn_start(
            turn,
            config.max_turns,
            messages.len(),
            total_input_tokens + total_output_tokens,
        );

        logger::log_api_request(
            &config.model,
            system_prompt.len(),
            messages.len(),
        );

        // Call the API
        let (response, duration_ms) = client
            .send_message(config, &system_prompt, &messages, tool_defs.clone())
            .await
            .with_context(|| format!("API call failed on turn {}", turn))?;

        // Update token counts
        total_input_tokens += response.usage.input_tokens as u64;
        total_output_tokens += response.usage.output_tokens as u64;

        logger::log_api_response(&response, duration_ms);

        let stop_reason = response.stop_reason.clone().unwrap_or_default();

        // Process content blocks — collect assistant message content
        let mut assistant_blocks: Vec<ContentBlock> = Vec::new();
        let mut tool_use_blocks: Vec<(String, String, serde_json::Value)> = Vec::new(); // (id, name, input)

        for block in &response.content {
            match block {
                ContentBlock::Text { text } => {
                    logger::log_text_block(text);
                    assistant_blocks.push(block.clone());
                }
                ContentBlock::Thinking { thinking } => {
                    logger::log_thinking_block(thinking);
                    assistant_blocks.push(block.clone());
                }
                ContentBlock::ToolUse { id, name, input } => {
                    // Summarize input for logging
                    let input_summary = summarize_tool_input(name, input);
                    logger::log_tool_call(name, &input_summary);

                    assistant_blocks.push(block.clone());
                    tool_use_blocks.push((id.clone(), name.clone(), input.clone()));
                }
                ContentBlock::ToolResult { .. } => {
                    // Unexpected in assistant response, but handle gracefully
                    assistant_blocks.push(block.clone());
                }
            }
        }

        // Add assistant message to history
        messages.push(Message {
            role: "assistant".to_string(),
            content: MessageContent::Blocks(assistant_blocks),
        });

        // ── Execute tools if stop_reason == "tool_use" ─────────────────────────

        if stop_reason == "tool_use" && !tool_use_blocks.is_empty() {
            let mut tool_result_blocks: Vec<ContentBlock> = Vec::new();

            for (tool_id, tool_name, tool_input) in &tool_use_blocks {
                tools_called.push(tool_name.clone());

                // Track file modifications
                if tool_name == "write_file" {
                    if let Some(path) = tool_input["path"].as_str() {
                        files_modified.push(path.to_string());
                    }
                }

                // Execute the tool
                let result = tools::execute_tool(tool_name, tool_input).await;

                logger::log_tool_result(
                    tool_name,
                    &result.output,
                    result.duration_ms,
                    result.success,
                );

                tool_result_blocks.push(ContentBlock::ToolResult {
                    tool_use_id: tool_id.clone(),
                    content: result.output.clone(),
                });

                // Check for task completion
                if result.is_complete {
                    task_completed = true;
                    completion_summary = result.output.clone();
                    // Extract summary from task_complete output
                    if let Some(stripped) = result.output.strip_prefix("TASK COMPLETE\n\n") {
                        completion_summary = stripped.to_string();
                    }
                }
            }

            // Add tool results to history as user message
            messages.push(Message {
                role: "user".to_string(),
                content: MessageContent::Blocks(tool_result_blocks),
            });

            // If task was completed via task_complete tool, break
            if task_completed {
                logger::log_info("Task completed via task_complete tool");
                break;
            }

            // Continue loop for next turn
            continue;
        }

        // ── end_turn or no tools — agent is done ──────────────────────────────

        if stop_reason == "end_turn" || stop_reason.is_empty() {
            // Extract final text as summary
            for block in &response.content {
                if let ContentBlock::Text { text } = block {
                    completion_summary = text.clone();
                    break;
                }
            }
            logger::log_info(&format!("Agent ended turn naturally (stop_reason={})", stop_reason));
            break;
        }

        // If we hit max_tokens stop reason, add a continuation message
        if stop_reason == "max_tokens" {
            logger::log_info("Max tokens reached, asking agent to continue");
            messages.push(Message {
                role: "user".to_string(),
                content: MessageContent::Text(
                    "Please continue where you left off. You ran out of tokens.".to_string(),
                ),
            });
            continue;
        }

        // Any other stop reason: log and break
        logger::log_info(&format!("Unexpected stop_reason: '{}', ending loop", stop_reason));
        break;
    }

    let duration_seconds = agent_start.elapsed().as_secs_f64();

    let report = AgentReport {
        turns: turns_completed,
        total_input_tokens,
        total_output_tokens,
        tools_called,
        files_modified,
        task_completed,
        summary: completion_summary,
        duration_seconds,
    };

    logger::log_agent_complete(&report);

    Ok(report)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn summarize_tool_input(name: &str, input: &serde_json::Value) -> String {
    match name {
        "bash" => {
            let cmd = input["command"].as_str().unwrap_or("");
            format!("command: {:?}", truncate_str(cmd, 200))
        }
        "read_file" => {
            let path = input["path"].as_str().unwrap_or("");
            format!("path: {:?}", path)
        }
        "write_file" => {
            let path = input["path"].as_str().unwrap_or("");
            let content = input["content"].as_str().unwrap_or("");
            format!("path: {:?}\ncontent: {} chars", path, content.len())
        }
        "task_complete" => {
            let summary = input["summary"].as_str().unwrap_or("");
            format!("summary: {:?}", truncate_str(summary, 200))
        }
        _ => {
            serde_json::to_string_pretty(input)
                .unwrap_or_else(|_| input.to_string())
        }
    }
}

fn truncate_str(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let end = s.char_indices()
            .nth(max)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        &s[..end]
    }
}
