//! Agent router — tracks connected agents and routes messages.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::{AgentId, AgentState, AgentStatus};

/// Agent router — thread-safe agent registry with pub/sub routing.
#[derive(Debug, Clone)]
pub struct AgentRouter {
    agents: Arc<RwLock<HashMap<AgentId, AgentState>>>,
}

impl AgentRouter {
    /// Create a new empty router.
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new agent.
    pub async fn register(&self, agent_id: AgentId, name: String) {
        let state = AgentState::new(agent_id.clone(), name);
        self.agents.write().await.insert(agent_id, state);
    }

    /// Update an agent's status.
    pub async fn update_status(&self, agent_id: &str, status: AgentStatus, message: String) {
        let mut agents = self.agents.write().await;
        if let Some(state) = agents.get_mut(agent_id) {
            state.status = status;
            state.message = message;
            state.last_update = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Claim an issue for an agent.
    pub async fn claim_issue(&self, agent_id: &str, issue_number: u64, branch: String) {
        let mut agents = self.agents.write().await;
        if let Some(state) = agents.get_mut(agent_id) {
            state.issue = Some(issue_number);
            state.branch = Some(branch);
            state.status = AgentStatus::Claiming;
            state.last_update = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Remove an agent from the router.
    pub async fn unregister(&self, agent_id: &str) {
        self.agents.write().await.remove(agent_id);
    }

    /// Get all registered agents.
    pub async fn list(&self) -> Vec<AgentState> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Check if an agent is registered.
    pub async fn is_registered(&self, agent_id: &str) -> bool {
        self.agents.read().await.contains_key(agent_id)
    }

    /// Get a specific agent's state.
    pub async fn get(&self, agent_id: &str) -> Option<AgentState> {
        self.agents.read().await.get(agent_id).cloned()
    }
}

impl Default for AgentRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_list() {
        let router = AgentRouter::new();
        router.register("agent-1".into(), "Agent One".into()).await;
        router.register("agent-2".into(), "Agent Two".into()).await;

        let agents = router.list().await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn update_status() {
        let router = AgentRouter::new();
        router.register("agent-1".into(), "Agent One".into()).await;
        router.update_status("agent-1", AgentStatus::Working, "training".into()).await;

        let state = router.get("agent-1").await.unwrap();
        assert_eq!(state.status, AgentStatus::Working);
        assert_eq!(state.message, "training");
    }

    #[tokio::test]
    async fn claim_issue() {
        let router = AgentRouter::new();
        router.register("agent-1".into(), "Agent One".into()).await;
        router.claim_issue("agent-1", 42, "feature/issue-42".into()).await;

        let state = router.get("agent-1").await.unwrap();
        assert_eq!(state.issue, Some(42));
        assert_eq!(state.branch.as_deref(), Some("feature/issue-42"));
        assert_eq!(state.status, AgentStatus::Claiming);
    }

    #[tokio::test]
    async fn unregister() {
        let router = AgentRouter::new();
        router.register("agent-1".into(), "Agent One".into()).await;
        router.unregister("agent-1").await;

        let agents = router.list().await;
        assert!(agents.is_empty());
    }
}
