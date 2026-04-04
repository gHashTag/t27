use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub anthropic_api_key: String,
    pub anthropic_base_url: String,
    pub model: String,
    pub task_prompt: String,
    pub task_title: Option<String>,
    pub max_turns: u32,
    pub max_tokens: u32,
    pub sandbox_repo_url: Option<String>,
    pub gh_token: Option<String>,
    pub port: u16,
    pub log_file: String,
    pub verbose: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Support both ANTHROPIC_API_KEY and ANTHROPIC_AUTH_TOKEN
        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| env::var("ANTHROPIC_AUTH_TOKEN"))
            .context("ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN must be set")?;

        let anthropic_base_url = env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.z.ai/api/anthropic".to_string());

        let model = env::var("MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-5-20250514".to_string());

        let task_prompt = env::var("TASK_PROMPT")
            .context("TASK_PROMPT must be set")?;

        let task_title = env::var("TASK_TITLE").ok();

        let max_turns = env::var("MAX_TURNS")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<u32>()
            .context("MAX_TURNS must be a valid integer")?;

        let max_tokens = env::var("MAX_TOKENS")
            .unwrap_or_else(|_| "8192".to_string())
            .parse::<u32>()
            .context("MAX_TOKENS must be a valid integer")?;

        let sandbox_repo_url = env::var("SANDBOX_REPO_URL").ok();
        let gh_token = env::var("GH_TOKEN").ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("PORT must be a valid port number")?;

        let log_file = env::var("LOG_FILE")
            .unwrap_or_else(|_| "/tmp/agent-log.jsonl".to_string());

        let verbose = env::var("VERBOSE")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            != "false";

        Ok(Config {
            anthropic_api_key,
            anthropic_base_url,
            model,
            task_prompt,
            task_title,
            max_turns,
            max_tokens,
            sandbox_repo_url,
            gh_token,
            port,
            log_file,
            verbose,
        })
    }

    pub fn log_startup(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              T27 AGENT RUNNER — CONFIGURATION                ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  ANTHROPIC_API_KEY:   {}  ║", mask_secret(&self.anthropic_api_key));
        println!("║  ANTHROPIC_BASE_URL:  {:<38} ║", truncate(&self.anthropic_base_url, 38));
        println!("║  MODEL:               {:<38} ║", truncate(&self.model, 38));
        println!("║  MAX_TURNS:           {:<38} ║", self.max_turns);
        println!("║  MAX_TOKENS:          {:<38} ║", self.max_tokens);
        println!("║  PORT:                {:<38} ║", self.port);
        println!("║  LOG_FILE:            {:<38} ║", truncate(&self.log_file, 38));
        println!("║  VERBOSE:             {:<38} ║", self.verbose);

        if let Some(ref title) = self.task_title {
            println!("║  TASK_TITLE:          {:<38} ║", truncate(title, 38));
        }

        if let Some(ref repo) = self.sandbox_repo_url {
            println!("║  SANDBOX_REPO_URL:    {:<38} ║", truncate(repo, 38));
        }

        if self.gh_token.is_some() {
            println!("║  GH_TOKEN:            {:<38} ║", "[SET]");
        } else {
            println!("║  GH_TOKEN:            {:<38} ║", "[NOT SET]");
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  TASK_PROMPT ({} chars):                              ║", self.task_prompt.len());

        // Print task prompt with wrapping
        for line in wrap_text(&self.task_prompt, 58) {
            println!("║    {:<58} ║", line);
        }

        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }
}

fn mask_secret(s: &str) -> String {
    if s.len() <= 8 {
        return "****".to_string();
    }
    let prefix = &s[..4];
    let suffix = &s[s.len() - 4..];
    format!("{:<38}", format!("{}...{}", prefix, suffix))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
        if lines.len() >= 10 {
            // Truncate very long prompts in display
            current.push_str(" ...");
            break;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
