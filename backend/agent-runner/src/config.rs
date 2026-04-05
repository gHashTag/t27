use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // ── API Keys ─────────────────────────────────────────────────
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub railway_api_token: Option<String>,

    // ── Anthropic settings ───────────────────────────────────────
    pub anthropic_base_url: String,
    pub model: String,

    // ── Agent loop ───────────────────────────────────────────────
    pub task_prompt: Option<String>,
    pub task_title: Option<String>,
    pub max_turns: u32,
    pub max_tokens: u32,

    // ── Sandbox / repo ───────────────────────────────────────────
    pub sandbox_repo_url: Option<String>,
    pub gh_token: Option<String>,

    // ── Runtime ──────────────────────────────────────────────────
    pub port: u16,
    pub log_file: String,
    pub verbose: bool,
    pub opencode_binary: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // API keys — none required at load time (validated later if TASK_PROMPT set)
        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| env::var("ANTHROPIC_AUTH_TOKEN"))
            .ok();

        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let railway_api_token = env::var("RAILWAY_API_TOKEN").ok();

        let anthropic_base_url = env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.z.ai/api/anthropic".to_string());

        let model = env::var("MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-5-20250514".to_string());

        let task_prompt = env::var("TASK_PROMPT").ok();
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

        let opencode_binary = env::var("OPENCODE_BINARY")
            .unwrap_or_else(|_| "opencode".to_string());

        // Validation: if TASK_PROMPT is set, require at least one API key
        if task_prompt.is_some() && anthropic_api_key.is_none() && openai_api_key.is_none() {
            return Err(anyhow::anyhow!(
                "TASK_PROMPT is set but no API key found. \
                 Set ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN, or OPENAI_API_KEY."
            ));
        }

        // Pick a real anthropic_api_key for agent.rs compatibility
        // (agent.rs expects a non-optional key when running)
        Ok(Config {
            anthropic_api_key,
            openai_api_key,
            railway_api_token,
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
            opencode_binary,
        })
    }

    /// Return the active API key for Anthropic-compatible calls.
    /// Prefers ANTHROPIC_API_KEY, falls back to OPENAI_API_KEY usage pattern.
    pub fn effective_api_key(&self) -> &str {
        self.anthropic_api_key
            .as_deref()
            .or(self.openai_api_key.as_deref())
            .unwrap_or("")
    }

    pub fn log_startup(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              T27 AGENT RUNNER — CONFIGURATION                ║");
        println!("╠══════════════════════════════════════════════════════════════╣");

        match &self.anthropic_api_key {
            Some(k) => println!("║  ANTHROPIC_API_KEY:   {}  ║", mask_secret(k)),
            None    => println!("║  ANTHROPIC_API_KEY:   {:<38} ║", "[NOT SET]"),
        }
        match &self.openai_api_key {
            Some(k) => println!("║  OPENAI_API_KEY:      {}  ║", mask_secret(k)),
            None    => println!("║  OPENAI_API_KEY:      {:<38} ║", "[NOT SET]"),
        }
        match &self.railway_api_token {
            Some(_) => println!("║  RAILWAY_API_TOKEN:   {:<38} ║", "[SET]"),
            None    => println!("║  RAILWAY_API_TOKEN:   {:<38} ║", "[NOT SET]"),
        }

        println!("║  ANTHROPIC_BASE_URL:  {:<38} ║", truncate(&self.anthropic_base_url, 38));
        println!("║  MODEL:               {:<38} ║", truncate(&self.model, 38));
        println!("║  OPENCODE_BINARY:     {:<38} ║", truncate(&self.opencode_binary, 38));
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
        match &self.gh_token {
            Some(_) => println!("║  GH_TOKEN:            {:<38} ║", "[SET]"),
            None    => println!("║  GH_TOKEN:            {:<38} ║", "[NOT SET]"),
        }

        if let Some(ref prompt) = self.task_prompt {
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║  TASK_PROMPT ({} chars):                              ║", prompt.len());
            for line in wrap_text(prompt, 58) {
                println!("║    {:<58} ║", line);
            }
        } else {
            println!("║  TASK_PROMPT:         {:<38} ║", "[NOT SET — web UI only mode]");
        }

        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }
}

// ─── helpers (also used by agent.rs compat layer) ────────────────────────────

pub fn mask_secret(s: &str) -> String {
    if s.len() <= 8 {
        return format!("{:<38}", "****");
    }
    let prefix = &s[..4];
    let suffix = &s[s.len() - 4..];
    format!("{:<38}", format!("{}...{}", prefix, suffix))
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
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
            current.push_str(" ...");
            break;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
