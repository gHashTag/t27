use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub id: u64,
    pub name: String,
    pub working_dir: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub env: HashMap<String, String>,
}

impl SessionConfig {
    pub fn new(name: impl Into<String>, working_dir: impl Into<String>) -> Self {
        Self {
            id: SESSION_COUNTER.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            working_dir: working_dir.into(),
            model: "claude-sonnet-4-20250514".to_string(),
            system_prompt: None,
            env: HashMap::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub config: SessionConfig,
    pub messages: Vec<Message>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Idle,
    Running,
    WaitingInput,
    Error,
    Terminated,
}
