mod agent;
mod api;
mod config;
mod logger;
mod steps;
mod tools;

use anyhow::{Context, Result};
use std::process;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // ── Step 0: Init structured logging ───────────────────────────────────────
    // JSON format for Railway log aggregation; pretty for local dev
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_ansi(false)
        .init();

    // ── Load config from env vars ─────────────────────────────────────────────
    let config = config::Config::from_env().context("Failed to load configuration")?;

    // Init JSONL log file
    logger::init_log_file(&config.log_file);

    // Print startup banner
    logger::log_banner("T27 AGENT RUNNER STARTING");
    config.log_startup();

    run(config).await
}

async fn run(config: config::Config) -> Result<()> {
    // ── Step 1: GitHub auth ────────────────────────────────────────────────────
    steps::github_auth::setup(&config)
        .await
        .context("GitHub auth step failed")?;

    // ── Step 2: Clone / pull repo ──────────────────────────────────────────────
    let work_dir = steps::git_clone::clone_or_pull(&config)
        .await
        .context("Git clone/pull step failed")?;

    // ── Step 3: Write opencode.json ────────────────────────────────────────────
    steps::opencode_config::write_config(&config, &work_dir)
        .context("opencode config write failed")?;

    // ── Step 4: Spawn web UI ───────────────────────────────────────────────────
    logger::log_info(&format!("Spawning web UI in {:?}", work_dir));
    let web_child = match steps::web_server::spawn(&config, &work_dir).await {
        Ok(child) => {
            logger::log_info(&format!("Web UI process started (PID {:?})", child.id()));
            Some(child)
        }
        Err(e) => {
            logger::log_error("spawn web UI", &e.to_string());
            logger::log_info("Continuing without web UI");
            None
        }
    };

    // ── Step 5: Poll health ────────────────────────────────────────────────────
    steps::web_server::wait_for_ready(&config, 60).await;

    // ── Step 6-9: Agent loop (only if TASK_PROMPT is set) ────────────────────
    if let Some(ref _prompt) = config.task_prompt {
        logger::log_step(
            6, 10, "AUTONOMOUS AGENT MODE",
            &[
                ("TASK_PROMPT", "set"),
                ("MODEL", &config.model),
                ("MAX_TURNS", &config.max_turns.to_string()),
                ("MAX_TOKENS", &config.max_tokens.to_string()),
            ],
        );

        let report = agent::run_agent(&config)
            .await
            .context("Agent loop failed")?;

        logger::log_step_result(
            6, report.task_completed,
            (report.duration_seconds * 1000.0) as u64,
            if report.task_completed { "task completed" } else { "max turns reached" },
        );

        if report.task_completed {
            logger::log_info("Agent runner exiting successfully (task completed)");
        } else {
            logger::log_info(&format!(
                "Agent runner exiting (max turns {} reached without task_complete)",
                config.max_turns
            ));
        }

        // If we had a web process, let it die naturally or kill it
        if let Some(mut child) = web_child {
            logger::log_info("Agent done — keeping web UI alive until process exits");
            // Wait on the web server process (keeps container running)
            match child.wait().await {
                Ok(status) => logger::log_info(&format!("Web UI exited: {}", status)),
                Err(e) => logger::log_error("web UI wait", &e.to_string()),
            }
        }

        process::exit(0);
    } else {
        // No TASK_PROMPT — web-UI only mode, keep alive forever
        logger::log_step(
            6, 10, "WEB UI ONLY MODE",
            &[
                ("TASK_PROMPT", "not set"),
                ("Mode", "web UI observation only"),
                ("URL", &format!("http://0.0.0.0:{}", config.port)),
            ],
        );

        match web_child {
            Some(mut child) => {
                logger::log_info("Waiting on web UI process (keep-alive)...");
                match child.wait().await {
                    Ok(status) => {
                        logger::log_info(&format!("Web UI exited: {}", status));
                    }
                    Err(e) => {
                        logger::log_error("web UI wait", &e.to_string());
                    }
                }
            }
            None => {
                // No web process — sleep forever so the container stays up
                logger::log_info("No web UI process — entering keep-alive sleep loop");
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    logger::log_info("keep-alive ping");
                }
            }
        }

        process::exit(0);
    }
}
