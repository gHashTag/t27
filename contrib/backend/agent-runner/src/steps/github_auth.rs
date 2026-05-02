use anyhow::Result;
use std::time::Instant;
use tokio::process::Command;

use crate::config::Config;
use crate::logger;

/// Step 1 — GitHub Authentication
///
/// If GH_TOKEN is set:
///   1. Pipes the token into `gh auth login --with-token`
///   2. Configures git URL rewriting so every `https://github.com/` clone
///      uses the token transparently.
///
/// If GH_TOKEN is absent the step is a no-op (returns Ok).
pub async fn setup(config: &Config) -> Result<()> {
    let token = match &config.gh_token {
        Some(t) => t.clone(),
        None => {
            logger::log_step(
                1, 10, "GITHUB AUTHENTICATION",
                &[("GH_TOKEN", "not set — skipping (public repos only)")],
            );
            logger::log_step_result(1, true, 0, "skipped (no GH_TOKEN)");
            return Ok(());
        }
    };

    let token_hint = format!("set ({} chars)", token.len());
    logger::log_step(
        1, 10, "GITHUB AUTHENTICATION",
        &[
            ("GH_TOKEN", &token_hint),
            ("Command", "gh auth login --with-token"),
        ],
    );

    let start = Instant::now();

    // ── gh auth login ──────────────────────────────────────────────────────────
    let gh_result = run_with_stdin("gh", &["auth", "login", "--with-token"], &token).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match &gh_result {
        Ok(out) => {
            logger::log_tool_section(
                "gh auth login",
                "gh auth login --with-token",
                Some(0),
                out,
                "",
                duration_ms,
            );
        }
        Err(e) => {
            // Non-fatal — log and continue; git URL rewrite may still work
            logger::log_error("gh auth login", &e.to_string());
        }
    }

    // ── git config URL rewrite ─────────────────────────────────────────────────
    let insteadof_url = format!(
        "https://x-access-token:{}@github.com/",
        token
    );
    let git_result = Command::new("git")
        .args([
            "config", "--global",
            &format!("url.{}.insteadOf", insteadof_url),
            "https://github.com/",
        ])
        .output()
        .await;

    match git_result {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            logger::log_tool_section(
                "git config url",
                "git config --global url.<token>@github.com.insteadOf https://github.com/",
                Some(out.status.code().unwrap_or(-1)),
                &stdout,
                &stderr,
                0,
            );
        }
        Err(e) => {
            logger::log_error("git config url rewrite", &e.to_string());
        }
    }

    let total_ms = start.elapsed().as_millis() as u64;
    logger::log_step_result(1, true, total_ms, "GitHub authentication complete");

    Ok(())
}

// ─── helper — run a command with stdin piped ──────────────────────────────────

async fn run_with_stdin(program: &str, args: &[&str], stdin_data: &str) -> Result<String> {
    use tokio::io::AsyncWriteExt;

    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_data.as_bytes()).await?;
    }

    let output = child.wait_with_output().await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(anyhow::anyhow!(
            "gh auth login exited {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        ))
    }
}
