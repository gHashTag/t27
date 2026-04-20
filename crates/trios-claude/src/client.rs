use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::process::ClaudeProcess;
use crate::session::{Message, Role, SessionConfig, SessionState, SessionStatus};

pub struct ClaudeClient {
    sessions: Arc<Mutex<HashMap<u64, ClaudeProcess>>>,
    states: Arc<Mutex<HashMap<u64, SessionState>>>,
}

impl Default for ClaudeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeClient {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, config: SessionConfig) -> Result<u64> {
        let id = config.id;
        let state = SessionState {
            config: config.clone(),
            messages: Vec::new(),
            status: SessionStatus::Idle,
        };

        let process = ClaudeProcess::new(config);

        self.sessions.lock().await.insert(id, process);
        self.states.lock().await.insert(id, state);

        Ok(id)
    }

    pub async fn send_prompt(&self, session_id: u64, prompt: &str) -> Result<String> {
        let mut sessions = self.sessions.lock().await;
        let process = sessions
            .get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("session {} not found", session_id))?;

        self.add_message(session_id, Role::User, prompt).await;

        let response = process.send_prompt(prompt).await?;

        self.add_message(session_id, Role::Assistant, &response).await;

        Ok(response)
    }

    pub async fn kill_session(&self, session_id: u64) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(process) = sessions.get_mut(&session_id) {
            process.kill().await?;
        }
        sessions.remove(&session_id);

        if let Some(state) = self.states.lock().await.get_mut(&session_id) {
            state.status = SessionStatus::Terminated;
        }

        Ok(())
    }

    pub async fn get_state(&self, session_id: u64) -> Option<SessionState> {
        self.states.lock().await.get(&session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<SessionState> {
        self.states.lock().await.values().cloned().collect()
    }

    async fn add_message(&self, session_id: u64, role: Role, content: &str) {
        let msg = Message {
            role,
            content: content.to_string(),
            timestamp: chrono_less_timestamp(),
        };

        if let Some(state) = self.states.lock().await.get_mut(&session_id) {
            state.messages.push(msg);
        }
    }
}

fn chrono_less_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", now.as_secs(), now.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let client = ClaudeClient::new();
        let config = SessionConfig::new("test-agent", "/tmp");
        let id = client.create_session(config).await.unwrap();
        assert!(id >= 1);

        let sessions = client.list_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].config.name, "test-agent");
    }

    #[tokio::test]
    async fn test_kill_nonexistent_session() {
        let client = ClaudeClient::new();
        let result = client.kill_session(999).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_state() {
        let client = ClaudeClient::new();
        let config = SessionConfig::new("state-test", "/tmp");
        let id = client.create_session(config).await.unwrap();

        let state = client.get_state(id).await.unwrap();
        assert_eq!(state.config.name, "state-test");
        assert_eq!(state.messages.len(), 0);
        assert_eq!(state.status, SessionStatus::Idle);
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let client = ClaudeClient::new();
        let c1 = SessionConfig::new("alpha", "/tmp");
        let c2 = SessionConfig::new("beta", "/tmp");

        let id1 = client.create_session(c1).await.unwrap();
        let id2 = client.create_session(c2).await.unwrap();

        assert_ne!(id1, id2);

        let sessions = client.list_sessions().await;
        assert_eq!(sessions.len(), 2);

        client.kill_session(id1).await.unwrap();
        client.kill_session(id2).await.unwrap();
    }

    #[test]
    fn test_session_config_builder() {
        let config = SessionConfig::new("builder-test", "/workspace")
            .with_model("claude-opus-4")
            .with_system_prompt("You are a helpful assistant")
            .with_env("ANTHROPIC_API_KEY", "test-key");

        assert_eq!(config.name, "builder-test");
        assert_eq!(config.model, "claude-opus-4");
        assert_eq!(
            config.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
        assert_eq!(
            config.env.get("ANTHROPIC_API_KEY"),
            Some(&"test-key".to_string())
        );
    }
}
