mod agent;
mod api;
mod config;
mod logger;
mod tools;

use anyhow::{Context, Result};
use std::process;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // ── Init structured logging ────────────────────────────────────────────────
    // JSON format for Railway log aggregation; human-readable goes to stdout
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_ansi(false)
        .init();

    // ── Parse config ──────────────────────────────────────────────────────────
    let config = config::Config::from_env().context("Failed to load configuration")?;

    // Init JSONL log file
    logger::init_log_file(&config.log_file);

    // Print startup config (masks secrets)
    config.log_startup();

    // ── Clone repo if requested ────────────────────────────────────────────────
    if let Some(ref repo_url) = config.sandbox_repo_url {
        clone_repo(repo_url, config.gh_token.as_deref()).await?;
    }

    // ── Start OpenCode web server (optional, best-effort) ─────────────────────
    let _opencode_handle = start_opencode(config.port).await;

    // ── Run agent loop ─────────────────────────────────────────────────────────
    let report = agent::run_agent(&config).await?;

    // ── Exit with appropriate status ──────────────────────────────────────────
    if report.task_completed {
        logger::log_info("Agent runner exiting successfully (task completed)");
        process::exit(0);
    } else {
        logger::log_info(&format!(
            "Agent runner exiting (max turns {} reached without task_complete)",
            config.max_turns
        ));
        // Exit 0 even if max turns — the logs tell the real story
        process::exit(0);
    }
}

// ─── Repo Cloning ─────────────────────────────────────────────────────────────

async fn clone_repo(repo_url: &str, gh_token: Option<&str>) -> Result<()> {
    logger::log_banner("CLONING REPOSITORY");
    logger::log_info(&format!("Repo URL: {}", repo_url));

    // Build authenticated URL if token provided
    let clone_url = if let Some(token) = gh_token {
        // Insert token into https URL: https://TOKEN@github.com/...
        if repo_url.starts_with("https://") {
            let without_scheme = &repo_url["https://".len()..];
            format!("https://{}@{}", token, without_scheme)
        } else {
            repo_url.to_string()
        }
    } else {
        repo_url.to_string()
    };

    // Clone to /workspace
    let output = tokio::process::Command::new("git")
        .args(["clone", "--depth=1", &clone_url, "/workspace"])
        .output()
        .await
        .context("Failed to spawn git clone")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    logger::log_tool_section("git clone", repo_url, Some(exit_code), &stdout, &stderr, 0);

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git clone failed with exit code {}: {}",
            exit_code,
            stderr
        ));
    }

    // Change working directory to /workspace
    std::env::set_current_dir("/workspace")
        .context("Failed to change to /workspace after clone")?;

    logger::log_info("Repository cloned successfully, working directory: /workspace");
    Ok(())
}

// ─── OpenCode Web Server ──────────────────────────────────────────────────────

async fn start_opencode(port: u16) -> Option<tokio::process::Child> {
    // Check if opencode binary exists
    let which = tokio::process::Command::new("which")
        .arg("opencode")
        .output()
        .await;

    let opencode_available = match which {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };

    if !opencode_available {
        logger::log_info("opencode binary not found — skipping web UI server");
        return None;
    }

    logger::log_info(&format!("Starting opencode web server on port {}", port));

    match tokio::process::Command::new("opencode")
        .args(["serve", "--port", &port.to_string()])
        .spawn()
    {
        Ok(child) => {
            logger::log_info(&format!("OpenCode web server started (PID {})", child.id().unwrap_or(0)));
            Some(child)
        }
        Err(e) => {
            logger::log_info(&format!("Failed to start opencode: {} — continuing without web UI", e));
            None
        }
    }
}
