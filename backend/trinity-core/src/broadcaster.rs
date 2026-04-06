//! Trinity Core - SSE Broadcaster
//!
//! Broadcasts events to all connected SSE clients.

use crate::store::Store;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use parking_lot::RwLock;

/// SSE Event wrapper
#[derive(Debug, Clone, Serialize)]
pub struct SseEvent {
    pub directory: String,
    pub payload: SsePayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct SsePayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

impl SseEvent {
    pub fn new(directory: String, event_type: &str, properties: serde_json::Value) -> Self {
        Self {
            directory,
            payload: SsePayload {
                event_type: event_type.to_string(),
                properties,
            },
        }
    }

    pub fn session_created(directory: &str, session: &serde_json::Value) -> Self {
        Self::new(
            directory.to_string(),
            "session.created",
            serde_json::json!({ "info": session }),
        )
    }

    pub fn session_updated(directory: &str, session: &serde_json::Value) -> Self {
        Self::new(
            directory.to_string(),
            "session.updated",
            serde_json::json!({ "info": session }),
        )
    }

    pub fn session_status(directory: &str, session_id: &str, status: &str) -> Self {
        Self::new(
            directory.to_string(),
            "session.status",
            serde_json::json!({
                "sessionID": session_id,
                "status": { "type": status }
            }),
        )
    }

    pub fn message_updated(directory: &str, session_id: &str, message: &serde_json::Value) -> Self {
        Self::new(
            directory.to_string(),
            "message.updated",
            serde_json::json!({
                "sessionID": session_id,
                "info": message
            }),
        )
    }

    pub fn message_removed(directory: &str, session_id: &str, message_id: &str) -> Self {
        Self::new(
            directory.to_string(),
            "message.removed",
            serde_json::json!({
                "sessionID": session_id,
                "messageID": message_id
            }),
        )
    }

    pub fn message_part_updated(directory: &str, part: &serde_json::Value) -> Self {
        Self::new(
            directory.to_string(),
            "message.part.updated",
            serde_json::json!({ "part": part }),
        )
    }

    pub fn part_delta(directory: &str, message_id: &str, part_id: &str, field: &str, delta: &str) -> Self {
        Self::new(
            directory.to_string(),
            "message.part.delta",
            serde_json::json!({
                "messageID": message_id,
                "partID": part_id,
                "field": field,
                "delta": delta
            }),
        )
    }
}

/// SSE Broadcaster for distributing events to all connected clients
pub struct SseBroadcaster {
    subscribers: Arc<RwLock<Vec<broadcast::Sender<String>>>>,
}

impl SseBroadcaster {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Subscribe to SSE events. Returns a channel receiver.
    pub fn subscribe(self: &Arc<Self>) -> broadcast::Receiver<String> {
        let (tx, rx) = broadcast::channel(100);
        self.subscribers.write().push(tx);
        rx
    }

    /// Broadcast an event to all subscribers
    pub fn broadcast(self: &Arc<Self>, event: &SseEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            for tx in self.subscribers.read().iter() {
                let _ = tx.send(json.clone());
            }
        }
    }

    /// Emit session created event
    pub fn emit_session_created(self: &Arc<Self>, directory: &str, session: &serde_json::Value) {
        self.broadcast(&SseEvent::session_created(directory, session));
    }

    /// Emit session updated event
    pub fn emit_session_updated(self: &Arc<Self>, directory: &str, session: &serde_json::Value) {
        self.broadcast(&SseEvent::session_updated(directory, session));
    }

    /// Emit session status event
    pub fn emit_session_status(self: &Arc<Self>, directory: &str, session_id: &str, status: &str) {
        self.broadcast(&SseEvent::session_status(directory, session_id, status));
    }

    /// Emit message updated event
    pub fn emit_message_updated(self: &Arc<Self>, directory: &str, session_id: &str, message: &serde_json::Value) {
        self.broadcast(&SseEvent::message_updated(directory, session_id, message));
    }

    /// Emit message removed event
    pub fn emit_message_removed(self: &Arc<Self>, directory: &str, session_id: &str, message_id: &str) {
        self.broadcast(&SseEvent::message_removed(directory, session_id, message_id));
    }

    /// Emit part updated event
    pub fn emit_part_updated(self: &Arc<Self>, directory: &str, part: &serde_json::Value) {
        self.broadcast(&SseEvent::message_part_updated(directory, part));
    }

    /// Emit part delta event
    pub fn emit_part_delta(self: &Arc<Self>, directory: &str, message_id: &str, part_id: &str, field: &str, delta: &str) {
        self.broadcast(&SseEvent::part_delta(directory, message_id, part_id, field, delta));
    }
}

impl Default for SseBroadcaster {
    fn default() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub broadcaster: Arc<SseBroadcaster>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: Store::new(),
            broadcaster: SseBroadcaster::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
