//! Wire protocol for Trinity Agent Bridge.
//!
//! This module defines the message types that flow between:
//! - Chrome Extension (service-worker, content scripts, popup)
//! - Rust WebSocket server (trios-bridge)
//! - CLI commands (tri bridge)
//!
//! All types must be kept in sync with the TypeScript types in
//! `extension/src/shared/types.ts`.

use serde::{Deserialize, Serialize};

/// Agent ID type.
pub type AgentId = String;

/// Agent ID or "broadcast" for sending to all agents.
pub type AgentIdOrBroadcast = String;

/// Agent status - MUST match issue #56 exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is idle, waiting for work
    Idle,
    /// Agent is claiming an issue
    Claiming,
    /// Agent is working on its claimed issue
    Working,
    /// Agent is blocked, waiting for something
    Blocked,
    /// Agent has completed its issue
    Done,
}

impl AgentStatus {
    /// Get the emoji for this status
    pub fn emoji(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "🟢",
            AgentStatus::Claiming => "🟡",
            AgentStatus::Working => "🔵",
            AgentStatus::Blocked => "🔴",
            AgentStatus::Done => "✅",
        }
    }
}

/// Agent state - represents an agent's current status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Unique agent identifier
    pub id: AgentId,
    /// Human-readable agent name
    pub name: String,
    /// GitHub issue number currently claimed (if any)
    pub issue: Option<u64>,
    /// Current status of the agent
    pub status: AgentStatus,
    /// Git branch the agent is working on (if any)
    pub branch: Option<String>,
    /// ISO timestamp of last update
    pub last_update: String,
    /// Last status message or task description
    pub message: String,
}

impl AgentState {
    /// Create a new agent state.
    pub fn new(id: AgentId, name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            name,
            issue: None,
            status: AgentStatus::Idle,
            branch: None,
            last_update: now,
            message: String::new(),
        }
    }

    /// Update the agent's status.
    pub fn with_status(mut self, status: AgentStatus, message: impl Into<String>) -> Self {
        self.status = status;
        self.message = message.into();
        self.last_update = chrono::Utc::now().to_rfc3339();
        self
    }

    /// Claim an issue.
    pub fn with_issue(mut self, issue: u64, branch: String) -> Self {
        self.issue = Some(issue);
        self.branch = Some(branch);
        self.status = AgentStatus::Claiming;
        self.last_update = chrono::Utc::now().to_rfc3339();
        self
    }
}

/// Issue status for the child issues tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatus {
    /// Issue number
    pub number: u64,
    /// Issue title
    pub title: String,
    /// Current status: "todo", "in_progress", "blocked", "done"
    pub status: String,
}

/// Agent event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEvent {
    /// Agent claimed an issue
    Claimed,
    /// Agent completed an issue
    Done,
    /// Agent is blocked
    Blocked,
}

// ============================================================================
// Messages: Client → Server
// ============================================================================

/// Send a command to a specific agent or broadcast to all.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendCommandMsg {
    /// Target agent ID or "broadcast" for all agents
    pub target: AgentIdOrBroadcast,
    /// Command text to send
    pub command: String,
    /// Also post this as a comment to GitHub issue?
    pub issue_comment: bool,
}

/// Claim a GitHub issue for work.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimIssueMsg {
    /// Agent ID claiming the issue
    pub agent_id: AgentId,
    /// GitHub issue number to claim
    pub issue_number: u64,
    /// Git branch to work on
    pub branch: String,
}

/// Update agent status.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStatusMsg {
    /// Agent ID to update
    pub agent_id: AgentId,
    /// New status
    pub status: AgentStatus,
    /// Status message
    pub message: String,
}

/// List connected agents.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListAgentsMsg;

// ============================================================================
// Messages: Server → Client
// ============================================================================

/// Full board state - all agents and issues.
#[derive(Debug, Serialize, Deserialize)]
pub struct BoardStateMsg {
    /// List of all connected agents
    pub agents: Vec<AgentState>,
    /// List of issue #30 child issues status
    pub issues: Vec<IssueStatus>,
}

/// An agent event occurred.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentEventMsg {
    /// Agent that triggered the event
    pub agent_id: AgentId,
    /// Event type
    pub event: AgentEvent,
    /// Related issue number
    pub issue_number: u64,
    /// Event message
    pub message: String,
}

/// Command was delivered to agent(s).
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDeliveredMsg {
    /// Target agent ID
    pub target: AgentId,
    /// Whether delivery succeeded
    pub success: bool,
    /// Error message if success is false
    pub error: Option<String>,
}

/// Error response from server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMsg {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

// ============================================================================
// Unified message type for WebSocket communication
// ============================================================================

/// Unified message type for WebSocket communication.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeMessage {
    /// Send command to agent(s)
    SendCommand(SendCommandMsg),

    /// Claim a GitHub issue
    ClaimIssue(ClaimIssueMsg),

    /// Update agent status
    UpdateStatus(UpdateStatusMsg),

    /// List connected agents
    ListAgents(ListAgentsMsg),

    /// Full board state
    BoardState(BoardStateMsg),

    /// Agent event
    AgentEvent(AgentEventMsg),

    /// Command delivery confirmation
    CommandDelivered(CommandDeliveredMsg),

    /// Error response
    Error(ErrorMsg),
}

impl BridgeMessage {
    /// Create a SendCommand message.
    pub fn send_command(target: AgentIdOrBroadcast, command: String, issue_comment: bool) -> Self {
        Self::SendCommand(SendCommandMsg {
            target,
            command,
            issue_comment,
        })
    }

    /// Create a ClaimIssue message.
    pub fn claim_issue(agent_id: AgentId, issue_number: u64, branch: String) -> Self {
        Self::ClaimIssue(ClaimIssueMsg {
            agent_id,
            issue_number,
            branch,
        })
    }

    /// Create an UpdateStatus message.
    pub fn update_status(agent_id: AgentId, status: AgentStatus, message: String) -> Self {
        Self::UpdateStatus(UpdateStatusMsg {
            agent_id,
            status,
            message,
        })
    }

    /// Create a ListAgents message.
    pub fn list_agents() -> Self {
        Self::ListAgents(ListAgentsMsg)
    }

    /// Create a BoardState message.
    pub fn board_state(agents: Vec<AgentState>, issues: Vec<IssueStatus>) -> Self {
        Self::BoardState(BoardStateMsg { agents, issues })
    }

    /// Create an AgentEvent message.
    pub fn agent_event(agent_id: AgentId, event: AgentEvent, issue_number: u64, message: String) -> Self {
        Self::AgentEvent(AgentEventMsg {
            agent_id,
            event,
            issue_number,
            message,
        })
    }

    /// Create a CommandDelivered message.
    pub fn command_delivered(target: AgentId, success: bool, error: Option<String>) -> Self {
        Self::CommandDelivered(CommandDeliveredMsg {
            target,
            success,
            error,
        })
    }

    /// Create an Error message.
    pub fn error(code: String, message: String) -> Self {
        Self::Error(ErrorMsg { code, message })
    }
}

// ============================================================================
// Constants
// ============================================================================

/// Default WebSocket port: 7474 (T-R-I-N in phone digits).
pub const DEFAULT_PORT: u16 = 7474;

/// WebSocket URL for local connection.
pub const LOCAL_WS_URL: &str = "ws://localhost:7474";

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_emoji() {
        assert_eq!(AgentStatus::Idle.emoji(), "🟢");
        assert_eq!(AgentStatus::Claiming.emoji(), "🟡");
        assert_eq!(AgentStatus::Working.emoji(), "🔵");
        assert_eq!(AgentStatus::Blocked.emoji(), "🔴");
        assert_eq!(AgentStatus::Done.emoji(), "✅");
    }

    #[test]
    fn test_agent_state_creation() {
        let state = AgentState::new("agent-1".into(), "Agent One".into());
        assert_eq!(state.id, "agent-1");
        assert_eq!(state.name, "Agent One");
        assert_eq!(state.status, AgentStatus::Idle);
        assert!(state.issue.is_none());
    }

    #[test]
    fn test_agent_state_with_status() {
        let state = AgentState::new("agent-1".into(), "Agent One".into())
            .with_status(AgentStatus::Working, "Training model");
        assert_eq!(state.status, AgentStatus::Working);
        assert_eq!(state.message, "Training model");
    }

    #[test]
    fn test_bridge_message_serialization() {
        let msg = BridgeMessage::send_command("broadcast".into(), "Hello".into(), false);
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: BridgeMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            BridgeMessage::SendCommand(cmd) => {
                assert_eq!(cmd.target, "broadcast");
                assert_eq!(cmd.command, "Hello");
            }
            _ => panic!("Expected SendCommand"),
        }
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus::Working;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""working""#);
    }
}
