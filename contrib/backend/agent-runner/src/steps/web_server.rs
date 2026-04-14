use anyhow::Result;
use std::path::Path;
use std::time::Instant;
use tokio::process::Child;
use tokio::process::Command;

use crate::config::Config;
use crate::logger;
use crate::steps::healthcheck;

/// Step 4 — Spawn the OpenCode web UI (or code-server fallback).
///
/// Tries `opencode web --hostname 0.0.0.0 --port PORT` first.
/// If the opencode binary is not found, falls back to
/// `code-server --bind-addr 0.0.0.0:PORT --auth none`.
///
/// Returns the Child handle so the caller can wait on it to keep
/// the container alive.
pub async fn spawn(config: &Config, work_dir: &Path) -> Result<Child> {
    logger::log_step(
        4, 10, "SPAWN WEB UI",
        &[
            ("binary", &config.opencode_binary),
            ("port", &config.port.to_string()),
            ("work_dir", work_dir.to_str().unwrap_or("?")),
        ],
    );

    let start = Instant::now();

    // Check whether the preferred binary exists
    let opencode_available = binary_exists(&config.opencode_binary).await;

    let child = if opencode_available {
        spawn_opencode(config, work_dir).await?
    } else {
        logger::log_info(&format!(
            "'{}' not found — trying code-server fallback",
            config.opencode_binary
        ));
        spawn_code_server(config, work_dir).await?
    };

    let pid = child.id().unwrap_or(0);
    let duration_ms = start.elapsed().as_millis() as u64;

    logger::log_step_result(
        4, true, duration_ms,
        &format!("web UI spawned (PID {})", pid),
    );

    Ok(child)
}

/// Step 5 — Poll the health endpoint until the server is ready.
///
/// Polls every 2 seconds for up to `max_wait_secs` (default 60).
/// Logs every poll attempt. Returns Ok(()) once healthy or after timeout
/// (non-fatal — the server may still be starting).
pub async fn wait_for_ready(config: &Config, max_wait_secs: u64) -> bool {
    logger::log_step(
        5, 10, "WAIT FOR WEB UI READY",
        &[
            ("url", &format!("http://localhost:{}/global/health", config.port)),
            ("max_wait", &format!("{}s", max_wait_secs)),
            ("poll_interval", "2s"),
        ],
    );

    let start = Instant::now();
    let max_duration = std::time::Duration::from_secs(max_wait_secs);
    let poll_interval = std::time::Duration::from_secs(2);
    let mut attempt = 0u32;

    loop {
        attempt += 1;
        let elapsed = start.elapsed();

        if elapsed >= max_duration {
            let duration_ms = elapsed.as_millis() as u64;
            logger::log_info(&format!(
                "Health check timed out after {}ms ({} attempts) — continuing anyway",
                duration_ms, attempt
            ));
            logger::log_step_result(5, false, duration_ms, "timeout — server may still be starting");
            return false;
        }

        let healthy = healthcheck::check(config.port).await;
        logger::log_info(&format!(
            "Health poll #{}: {} (elapsed {:.1}s)",
            attempt,
            if healthy { "✓ healthy" } else { "waiting..." },
            elapsed.as_secs_f64()
        ));

        if healthy {
            let duration_ms = start.elapsed().as_millis() as u64;
            logger::log_step_result(
                5, true, duration_ms,
                &format!("ready after {} polls ({:.1}s)", attempt, duration_ms as f64 / 1000.0),
            );
            return true;
        }

        tokio::time::sleep(poll_interval).await;
    }
}

// ─── internal spawn helpers ───────────────────────────────────────────────────

async fn spawn_opencode(config: &Config, work_dir: &Path) -> Result<Child> {
    let port_str = config.port.to_string();
    logger::log_info(&format!(
        "Spawning: {} web --hostname 0.0.0.0 --port {}",
        config.opencode_binary, port_str
    ));

    let child = Command::new(&config.opencode_binary)
        .args(["web", "--hostname", "0.0.0.0", "--port", &port_str])
        .current_dir(work_dir)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn {}: {}", config.opencode_binary, e))?;

    logger::log_info(&format!(
        "OpenCode started (PID {})",
        child.id().unwrap_or(0)
    ));
    Ok(child)
}

async fn spawn_code_server(config: &Config, work_dir: &Path) -> Result<Child> {
    let bind = format!("0.0.0.0:{}", config.port);
    logger::log_info(&format!(
        "Spawning: code-server --bind-addr {} --auth none .",
        bind
    ));

    // Try to find code-server in common locations
    let binary = find_code_server().await.unwrap_or_else(|| "code-server".to_string());

    let child = Command::new(&binary)
        .args(["--bind-addr", &bind, "--auth", "none", "."])
        .current_dir(work_dir)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn code-server: {}", e))?;

    logger::log_info(&format!(
        "code-server started (PID {})",
        child.id().unwrap_or(0)
    ));
    Ok(child)
}

// ─── utility ──────────────────────────────────────────────────────────────────

async fn binary_exists(name: &str) -> bool {
    // 'which' returns exit 0 if found
    Command::new("which")
        .arg(name)
        .output()
        .await
        .map(|out| out.status.success())
        .unwrap_or(false)
}

async fn find_code_server() -> Option<String> {
    let candidates = [
        "/root/.local/bin/code-server",
        "/usr/local/bin/code-server",
        "/usr/bin/code-server",
    ];
    for c in candidates {
        if tokio::fs::metadata(c).await.is_ok() {
            return Some(c.to_string());
        }
    }
    None
}
