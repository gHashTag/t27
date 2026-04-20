use anyhow::{Context, Result};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::session::{SessionConfig, SessionStatus};

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

pub struct ClaudeProcess {
    child: Option<Child>,
    config: SessionConfig,
}

impl ClaudeProcess {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            child: None,
            config,
        }
    }

    pub async fn spawn(&mut self, tx: mpsc::UnboundedSender<ProcessEvent>) -> Result<()> {
        let working_dir = Path::new(&self.config.working_dir);
        if !working_dir.exists() {
            std::fs::create_dir_all(working_dir)
                .with_context(|| format!("create working dir: {}", self.config.working_dir))?;
        }

        let mut cmd = Command::new("claude");
        cmd.arg("--print")
            .arg("--model")
            .arg(&self.config.model)
            .arg("--output-format")
            .arg("json")
            .current_dir(&self.config.working_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().with_context(|| "spawning claude process")?;

        let stdout = child.stdout.take().context("no stdout")?;
        let stderr = child.stderr.take().context("no stderr")?;

        let tx_out = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_out.send(ProcessEvent::Stdout(line)).is_err() {
                    break;
                }
            }
        });

        let tx_err = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_err.send(ProcessEvent::Stderr(line)).is_err() {
                    break;
                }
            }
        });

        self.child = Some(child);
        Ok(())
    }

    pub async fn send_prompt(&mut self, prompt: &str) -> Result<String> {
        let output = Command::new("claude")
            .arg("--print")
            .arg("--model")
            .arg(&self.config.model)
            .arg("--output-format")
            .arg("json")
            .arg(prompt)
            .current_dir(&self.config.working_dir)
            .output()
            .await
            .context("running claude prompt")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("claude process failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    pub async fn kill(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.child {
            child.kill().await.context("killing claude process")?;
            self.child = None;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    pub fn status(&self) -> SessionStatus {
        if self.is_running() {
            SessionStatus::Running
        } else {
            SessionStatus::Idle
        }
    }
}

impl Drop for ClaudeProcess {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
    }
}
