// Railway Vivado self-hosted GH Actions runner entrypoint.
// Required env: GH_RUNNER_TOKEN, GH_REPO_URL
// Optional env: RUNNER_NAME, RUNNER_LABELS, RUNNER_WORK
//
// This binary replaces the typical bash entrypoint.sh per project policy:
// no .sh / .py allowed; orchestration in Rust.

use anyhow::{Context, Result, bail};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_required(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("env var {key} is required"))
}

fn source_vivado() -> Result<()> {
    let settings = "/opt/Xilinx/Vivado/2025.2/settings64.sh";
    if Path::new(settings).exists() {
        // Capture env after sourcing
        let out = Command::new("bash")
            .arg("-c")
            .arg(format!("source {settings} && env"))
            .output()
            .context("failed to source Vivado settings")?;
        if !out.status.success() {
            bail!("vivado settings source failed: {}", String::from_utf8_lossy(&out.stderr));
        }
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some((k, v)) = line.split_once('=') {
                env::set_var(k, v);
            }
        }
        eprintln!("[entrypoint] Vivado env sourced from {settings}");
    } else {
        eprintln!("[entrypoint] WARN: {settings} not found; Vivado may not be installed yet");
    }
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("spawn {cmd}"))?;
    if !status.success() {
        bail!("{cmd} exited with {status}");
    }
    Ok(())
}

fn main() -> Result<()> {
    let runner_dir = "/actions-runner";
    env::set_current_dir(runner_dir).context("cd /actions-runner")?;

    let token = env_required("GH_RUNNER_TOKEN")?;
    let repo_url = env_required("GH_REPO_URL")?;
    let runner_name = env_or_default("RUNNER_NAME", "railway-vivado");
    let labels = env_or_default("RUNNER_LABELS", "vivado,x86_64,linux");
    let work = env_or_default("RUNNER_WORK", "/actions-runner/_work");

    // First-time registration is idempotent: skip if .runner exists.
    if !Path::new(".runner").exists() {
        eprintln!("[entrypoint] Registering runner '{runner_name}' against {repo_url}");
        run(
            "./config.sh",
            &[
                "--unattended",
                "--url", &repo_url,
                "--token", &token,
                "--name", &runner_name,
                "--labels", &labels,
                "--work", &work,
                "--replace",
            ],
        )?;
    } else {
        eprintln!("[entrypoint] Runner already configured; starting");
    }

    source_vivado()?;

    eprintln!("[entrypoint] Launching ./run.sh (GH Actions runner)");
    run("./run.sh", &[])
}
