use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;

use crate::config::Config;
use crate::logger;

/// Step 3 — Write opencode.json to the work directory and ~/.config/opencode/
///
/// Generates a valid opencode.json with:
///   - $schema, model, small_model, mcp (optional), server
///   - NO: providers, agents, keybindings, tools (these crash OpenCode)
///   - API keys are picked up from env vars by OpenCode automatically
pub fn write_config(config: &Config, work_dir: &Path) -> Result<()> {
    let start = Instant::now();

    // Determine default model from available API keys
    let (model, small_model) = choose_models(config);

    logger::log_step(
        3, 10, "WRITE OPENCODE CONFIG",
        &[
            ("work_dir", work_dir.to_str().unwrap_or("?")),
            ("model", &model),
            ("small_model", &small_model),
            ("MCP (railway)", if config.railway_api_token.is_some() { "yes" } else { "no" }),
        ],
    );

    // Build the config object
    let mut cfg: serde_json::Map<String, Value> = serde_json::Map::new();
    cfg.insert("$schema".to_string(), json!("https://opencode.ai/config.json"));
    cfg.insert("model".to_string(), json!(model));
    cfg.insert("small_model".to_string(), json!(small_model));

    // MCP block — only if RAILWAY_API_TOKEN is set
    if let Some(ref railway_token) = config.railway_api_token {
        cfg.insert("mcp".to_string(), json!({
            "railway": {
                "type": "local",
                "command": ["npx", "-y", "@railway/mcp-server"],
                "environment": {
                    "RAILWAY_API_TOKEN": railway_token
                }
            }
        }));
    }

    // Server block
    cfg.insert("server".to_string(), json!({
        "port": config.port,
        "hostname": "0.0.0.0"
    }));

    let config_json = serde_json::to_string_pretty(&Value::Object(cfg))?;

    // Masked version for logging (hide railway token if present)
    let log_json = if config.railway_api_token.is_some() {
        config_json.replace(
            config.railway_api_token.as_deref().unwrap_or(""),
            "<RAILWAY_API_TOKEN>",
        )
    } else {
        config_json.clone()
    };

    // Write to work_dir/opencode.json
    let work_path = work_dir.join("opencode.json");
    std::fs::write(&work_path, &config_json)
        .map_err(|e| anyhow::anyhow!("Failed to write {:?}: {}", work_path, e))?;

    logger::log_tool_section(
        "write opencode.json",
        work_path.to_str().unwrap_or("?"),
        Some(0),
        &format!("Wrote {} bytes\n{}", config_json.len(), log_json),
        "",
        0,
    );

    // Write to ~/.config/opencode/opencode.json
    let home_config_dir = dirs_home_config();
    if let Some(ref dir) = home_config_dir {
        let home_path = Path::new(dir).join("opencode").join("opencode.json");
        if let Some(parent) = home_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Err(e) = std::fs::write(&home_path, &config_json) {
            logger::log_error(
                "write ~/.config/opencode/opencode.json",
                &e.to_string(),
            );
        } else {
            logger::log_info(&format!(
                "Also wrote config to {:?}",
                home_path
            ));
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    logger::log_step_result(3, true, duration_ms, "opencode.json written");

    Ok(())
}

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Pick the right model string based on which API keys are available.
fn choose_models(config: &Config) -> (String, String) {
    // Prefer Anthropic; fall back to OpenAI
    if config.anthropic_api_key.is_some() {
        (
            "anthropic/claude-sonnet-4-5".to_string(),
            "anthropic/claude-haiku-3-5".to_string(),
        )
    } else if config.openai_api_key.is_some() {
        (
            "openai/gpt-4o".to_string(),
            "openai/gpt-4o-mini".to_string(),
        )
    } else {
        // Fallback — opencode will error on its own if no key found
        (
            "anthropic/claude-sonnet-4-5".to_string(),
            "anthropic/claude-haiku-3-5".to_string(),
        )
    }
}

/// Return ~/.config path portably.
fn dirs_home_config() -> Option<String> {
    // Try $HOME/.config
    if let Ok(home) = std::env::var("HOME") {
        return Some(format!("{}/.config", home));
    }
    None
}
