//! # trios-bridge — Trinity Agent Bridge
//!
//! WebSocket server on port 7474 (T-R-I-N in phone digits) for multi-agent
//! orchestration. Provides real-time communication between:
//! - Chrome Extension (claude-injector, github-injector, cursor-injector)
//! - CLI (`tri bridge start/status/send/broadcast`)
//! - GitHub Issues (comment parsing for agent status)
//!
//! ## Architecture
//!
//! ```text
//! Chrome Extension ←── WebSocket ──→ trios-bridge ←── WebSocket ──→ CLI
//!                                      │
//!                                      ├── Agent Router (pub/sub)
//!                                      ├── GitHub API (octocrab)
//!                                      └── Broadcast Router
//! ```
//!
//! ## Usage
//!
//! Start the server:
//! ```bash
//! tri bridge start
//! ```
//!
//! Show connected agents:
//! ```bash
//! tri bridge status
//! ```
//!
//! Send command to agent:
//! ```bash
//! tri bridge send agent-1 "deploy to production"
//! ```
//!
//! Broadcast to all:
//! ```bash
//! tri bridge broadcast "STOP asking questions, just push"
//! ```

pub mod protocol;
pub mod server;
pub mod router;
pub mod github;

pub use protocol::{
    AgentId, AgentIdOrBroadcast, AgentState, AgentStatus, AgentEvent,
    IssueStatus, BridgeMessage, DEFAULT_PORT, LOCAL_WS_URL,
};
pub use server::BridgeServer;
pub use router::AgentRouter;
pub use github::GitHubClient;
