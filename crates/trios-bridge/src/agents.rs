//! Agent registry — tracks connected agents and their status.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::protocol::{AgentInfo, AgentStatus};

/// Agent entry in the registry.
#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub info: AgentInfo,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Agent registry — thread-safe collection of connected agents.
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentEntry>>>,
}

impl AgentRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new agent.
    pub async fn register(&self, agent_id: String, agent_type: String, capabilities: Vec<String>) {
        let _ = capabilities; // Future: use for routing
        let now = chrono::Utc::now();
        let entry = AgentEntry {
            info: AgentInfo {
                agent_id: agent_id.clone(),
                agent_type,
                status: AgentStatus::Idle,
                task: None,
                connected_at: now.to_rfc3339(),
            },
            last_heartbeat: now,
        };
        self.agents.write().await.insert(agent_id, entry);
    }

    /// Update agent status.
    pub async fn update_status(&self, agent_id: &str, status: AgentStatus, task: Option<String>) {
        let mut agents = self.agents.write().await;
        if let Some(entry) = agents.get_mut(agent_id) {
            entry.info.status = status;
            entry.info.task = task;
            entry.last_heartbeat = chrono::Utc::now();
        }
    }

    /// Remove an agent from the registry.
    pub async fn unregister(&self, agent_id: &str) {
        self.agents.write().await.remove(agent_id);
    }

    /// Get all registered agents.
    pub async fn list(&self) -> Vec<AgentInfo> {
        self.agents
            .read()
            .await
            .values()
            .map(|e| e.info.clone())
            .collect()
    }

    /// Get agent IDs for broadcast.
    pub async fn agent_ids(&self) -> Vec<String> {
        self.agents.read().await.keys().cloned().collect()
    }

    /// Check if an agent is registered.
    pub async fn is_registered(&self, agent_id: &str) -> bool {
        self.agents.read().await.contains_key(agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_list() {
        let registry = AgentRegistry::new();
        registry.register("agent-1".into(), "phi".into(), vec![]).await;
        registry.register("agent-2".into(), "vibee".into(), vec![]).await;

        let agents = registry.list().await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn update_status() {
        let registry = AgentRegistry::new();
        registry.register("agent-1".into(), "phi".into(), vec![]).await;
        registry.update_status("agent-1", AgentStatus::Busy, Some("training".into())).await;

        let agents = registry.list().await;
        assert!(matches!(agents[0].status, AgentStatus::Busy));
        assert_eq!(agents[0].task.as_deref(), Some("training"));
    }

    #[tokio::test]
    async fn unregister() {
        let registry = AgentRegistry::new();
        registry.register("agent-1".into(), "phi".into(), vec![]).await;
        registry.unregister("agent-1").await;

        let agents = registry.list().await;
        assert!(agents.is_empty());
    }
}
