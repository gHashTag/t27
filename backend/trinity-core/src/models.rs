//! Trinity Core - Data Models
//!
//! Core data structures for session and message management.
//! Aligned with .t27 specs - Constitutional Law #4
//!
//! The .t27 specs in specs/server/ are the source of truth
//! This file bridges the gap with additional fields needed by handlers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════════
// Session State (from .t27 specs/server/session.t27)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Idle,
    Busy,
    Running,
    Completed,
    Failed,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

// ════════════════════════════════════════════════════════════════════
// Message Role (from .t27 specs/server/session.t27)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

// ════════════════════════════════════════════════════════════════════
// Session (from .t27 specs + handlers needed fields)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub slug: String,
    pub project_id: Option<String>,
    pub workspace_id: Option<String>,
    pub directory: String,
    pub parent_id: Option<String>,
    pub title: Option<String>,
    pub state: SessionState,
    pub time: SessionTime,
    pub message_count: u32,
    pub model: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<SessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionTime {
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compacting: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub additions: u32,
    pub deletions: u32,
    pub files: u32,
}

impl Session {
    pub fn new(directory: String) -> Self {
        let now = Utc::now();
        Self {
            id: format!("ses_{}", Uuid::new_v4()),
            slug: format!("session-{}", now.timestamp() % 1000000),
            project_id: None,
            workspace_id: None,
            directory,
            parent_id: None,
            title: None,
            state: SessionState::Idle,
            time: SessionTime {
                created: now,
                updated: now,
                archived: None,
                compacting: None,
            },
            message_count: 0,
            model: String::new(),
            provider: String::new(),
            summary: None,
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Extended Message (from .t27 specs + handlers needed fields)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", content = "data")]
pub enum Message {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
}

impl Message {
    pub fn session_id(&self) -> &str {
        match self {
            Message::User(m) => &m.session_id,
            Message::Assistant(m) => &m.session_id,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Message::User(m) => &m.id,
            Message::Assistant(m) => &m.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub id: String,
    pub session_id: String,
    pub parent_id: String,
    pub time: MessageTime,
    pub content: Vec<ContentBlock>,
}

impl UserMessage {
    pub fn new(session_id: String, parent_id: String, content: Vec<ContentBlock>) -> Self {
        Self {
            id: format!("msg_{}", Uuid::new_v4()),
            session_id,
            parent_id,
            time: MessageTime::now(),
            content,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub id: String,
    pub session_id: String,
    pub time: MessageTime,
    pub error: Option<MessageError>,
    pub parent_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub mode: String,
    pub agent: String,
    pub path: MessagePath,
    pub cost: f64,
    pub tokens: MessageTokens,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish: Option<String>,
}

impl AssistantMessage {
    pub fn new(
        session_id: String,
        parent_id: String,
        model_id: String,
        provider_id: String,
    ) -> Self {
        Self {
            id: format!("msg_{}", Uuid::new_v4()),
            session_id,
            time: MessageTime::default(),
            error: None,
            parent_id,
            model_id,
            provider_id,
            mode: "chat".to_string(),
            agent: "zai".to_string(),
            path: MessagePath {
                cwd: "/app".to_string(),
                root: "/app".to_string(),
            },
            cost: 0.0,
            tokens: MessageTokens::default(),
            structured: None,
            variant: None,
            finish: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.time.completed.is_some()
    }

    pub fn complete(&mut self) {
        self.time.completed = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageTime {
    pub created: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<DateTime<Utc>>,
}

impl MessageTime {
    pub fn now() -> Self {
        Self {
            created: Utc::now(),
            completed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePath {
    pub cwd: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageTokens {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub input: u64,
    pub output: u64,
    #[serde(default)]
    pub reasoning: u64,
    pub cache: CacheTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheTokens {
    pub read: u64,
    pub write: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageError {
    pub name: String,
    pub message: String,
}

// ════════════════════════════════════════════════════════════════════
// Content Block (from .t27 specs)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { id: String, text: String },
    Thinking { id: String, thinking: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { id: String, tool_use_id: String, content: String },
}

impl ContentBlock {
    pub fn text(id: String, text: String) -> Self {
        Self::Text { id, text }
    }

    pub fn thinking(id: String, thinking: String) -> Self {
        Self::Thinking { id, thinking }
    }

    pub fn tool_use(id: String, name: String, input: serde_json::Value) -> Self {
        Self::ToolUse { id, name, input }
    }

    pub fn tool_result(id: String, tool_use_id: String, content: String) -> Self {
        Self::ToolResult { id, tool_use_id, content }
    }
}

// ════════════════════════════════════════════════════════════════════
// Part (from .t27 specs)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text {
        id: String,
        message_id: String,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        time: Option<PartTime>,
    },
    Thinking {
        id: String,
        message_id: String,
        thinking: String,
    },
    ToolUse {
        id: String,
        message_id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        message_id: String,
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartTime {
    pub start: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
}

impl Part {
    pub fn id(&self) -> &str {
        match self {
            Part::Text { ref id, .. } => id,
            Part::Thinking { ref id, .. } => id,
            Part::ToolUse { ref id, .. } => id,
            Part::ToolResult { ref id, .. } => id,
        }
    }

    pub fn message_id(&self) -> &str {
        match self {
            Part::Text { ref message_id, .. } => message_id,
            Part::Thinking { ref message_id, .. } => message_id,
            Part::ToolUse { ref message_id, .. } => message_id,
            Part::ToolResult { ref message_id, .. } => message_id,
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// API Request/Response types
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub directory: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub role: String,
    pub parent_id: String,
    pub content: Vec<ContentBlock>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub completed: Option<bool>,
    pub cost: Option<f64>,
    pub tokens: Option<MessageTokens>,
    pub error: Option<MessageError>,
}

#[derive(Debug, Deserialize)]
pub struct AddPartRequest {
    #[serde(rename = "type")]
    pub part_type: String,
    pub name: Option<String>,
    pub input: Option<serde_json::Value>,
    pub content: Option<String>,
    pub tool_use_id: Option<String>,
}
