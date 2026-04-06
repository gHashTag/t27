use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

use crate::config::Config;
use crate::logger;

/// Step 2 — Clone or pull the sandbox repository.
///
/// Returns the working directory path:
/// - If no SANDBOX_REPO_URL: returns `/home/sandbox/workspace` (default)
/// - If URL set and dir already has `.git`: runs `git pull --ff-only`
/// - Otherwise: runs `git clone <url> <dir>`
pub async fn clone_or_pull(config: &Config) -> Result<PathBuf> {
    let repo_url = match &config.sandbox_repo_url {
        Some(u) => u.clone(),
        None => {
            logger::log_step(
                2, 10, "GIT CLONE / PULL",
                &[("SANDBOX_REPO_URL", "not set — using default workspace")],
            );
            logger::log_step_result(2, true, 0, "skipped (no repo URL)");

            // Default workspace dir
            let default_dir = PathBuf::from("/home/sandbox/workspace");
            tokio::fs::create_dir_all(&default_dir).await.ok();
            return Ok(default_dir);
        }
    };

    // Derive repo name from URL  (strip trailing .git)
    let repo_name = repo_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git")
        .to_string();

    let workspace = PathBuf::from("/home/sandbox/workspace");
    tokio::fs::create_dir_all(&workspace).await.ok();
    let target_dir = workspace.join(&repo_name);

    logger::log_step(
        2, 10, "GIT CLONE / PULL",
        &[
            ("SANDBOX_REPO_URL", &repo_url),
            ("Repo name", &repo_name),
            ("Target dir", target_dir.to_str().unwrap_or("?")),
        ],
    );

    let start = Instant::now();

    let git_dir = target_dir.join(".git");
    if git_dir.exists() {
        // ── pull ─────────────────────────────────────────────────────────────
        logger::log_info(&format!(
            "Directory {:?} already exists — running git pull --ff-only",
            target_dir
        ));

        let output = Command::new("git")
            .args(["-C", target_dir.to_str().unwrap_or("."), "pull", "--ff-only"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        logger::log_tool_section(
            "git pull",
            &format!("git -C {:?} pull --ff-only", target_dir),
            Some(exit_code),
            &stdout,
            &stderr,
            start.elapsed().as_millis() as u64,
        );

        if !output.status.success() {
            // Non-fatal: log warning, still return the dir
            logger::log_error("git pull", &format!("exit {}: {}", exit_code, stderr));
        }
    } else {
        // ── clone ─────────────────────────────────────────────────────────────
        logger::log_info(&format!(
            "Cloning {} -> {:?}",
            repo_url, target_dir
        ));

        // Build authenticated URL if GH_TOKEN available
        let clone_url = build_auth_url(&repo_url, config.gh_token.as_deref());

        let output = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                &clone_url,
                target_dir.to_str().unwrap_or("."),
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        // Log with token masked
        logger::log_tool_section(
            "git clone",
            &format!("git clone --depth=1 <url> {:?}", target_dir),
            Some(exit_code),
            &stdout,
            &stderr,
            start.elapsed().as_millis() as u64,
        );

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git clone failed (exit {}): {}",
                exit_code,
                stderr
            ));
        }
    }

    // Change process CWD to the repo
    std::env::set_current_dir(&target_dir)
        .map_err(|e| anyhow::anyhow!("Failed to chdir to {:?}: {}", target_dir, e))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    logger::log_step_result(
        2, true, duration_ms,
        &format!("working directory: {:?}", target_dir),
    );

    Ok(target_dir)
}

// ─── helpers ──────────────────────────────────────────────────────────────────

fn build_auth_url(url: &str, token: Option<&str>) -> String {
    if let Some(t) = token {
        if url.starts_with("https://") {
            let without_scheme = &url["https://".len()..];
            return format!("https://x-access-token:{}@{}", t, without_scheme);
        }
    }
    url.to_string()
}
