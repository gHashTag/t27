//! Trinity Core - HTTP Handlers
//!
//! Axum handlers for all API endpoints.

use crate::broadcaster::{AppState, SseEvent};
use serde::Deserialize;
use tokio::sync::broadcast;
use crate::models::{
    AddPartRequest, AssistantMessage, ContentBlock, CreateMessageRequest, CreateSessionRequest,
    Message, Part, Session, UpdateMessageRequest,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Sse},
    routing::{delete, get, patch, post},
    Router,
};
use chrono::Utc;
use futures_util::Stream;
use std::time::Duration;
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};

/// Health check
async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "healthy": true,
        "version": "0.1.0"
    }))
}

/// SSE stream handler
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, axum::Error>>> {
    let mut receiver = state.broadcaster.subscribe();

    let stream = async_stream::stream! {
        // Send server.connected immediately
        let connected = SseEvent::new(
            "global".to_string(),
            "server.connected",
            serde_json::json!({}),
        );
        if let Ok(data) = serde_json::to_string(&connected) {
            yield Ok(axum::response::sse::Event::default().data(data));
        }

        // Keep-alive ticker
        let mut tick = interval(Duration::from_secs(15));
        tick.tick().await;

        loop {
            tokio::select! {
                // Event from broadcaster
                result = receiver.recv() => {
                    match result {
                        Ok(data) => {
                            yield Ok(axum::response::sse::Event::default().data(data));
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                        Err(_) => {}
                    }
                }
                // Heartbeat
                _ = tick.tick() => {
                    let heartbeat = SseEvent::new(
                        "global".to_string(),
                        "server.heartbeat",
                        serde_json::json!({}),
                    );
                    if let Ok(data) = serde_json::to_string(&heartbeat) {
                        yield Ok(axum::response::sse::Event::default().data(data));
                    }
                }
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
}

// ─── Session Handlers ──────────────────────────────────────────────────────────

/// List sessions
async fn list_sessions_handler(
    State(state): State<AppState>,
    Query(params): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    let sessions = state.store.list_sessions(params.directory.as_deref());
    Json(sessions)
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub directory: Option<String>,
}

/// Get session
async fn get_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_session(&session_id) {
        Some(session) => Json(session).into_response() as axum::response::Response,
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Session not found"
        }))).into_response(),
    }
}

/// Create session
async fn create_session_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let mut session = Session::new(req.directory);
    session.parent_id = req.parent_id;
    session.title = req.title;

    let result = state.store.create_session(session.clone());

    // Emit SSE event
    let session_json = serde_json::to_value(&result).unwrap_or_default();
    state.broadcaster.emit_session_created(&result.directory, &session_json);

    (StatusCode::CREATED, Json(result)).into_response()
}

/// Update session
async fn update_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl IntoResponse {
    use crate::store::SessionUpdate;

    let update = SessionUpdate {
        status: req.status.map(|s| match s.as_str() {
            "idle" => crate::models::SessionState::Idle,
            "busy" => crate::models::SessionState::Busy,
            "running" => crate::models::SessionState::Running,
            "completed" => crate::models::SessionState::Completed,
            "failed" => crate::models::SessionState::Failed,
            _ => crate::models::SessionState::Idle,
        }),
        title: req.title,
        summary: req.summary,
        archived: req.archived.map(|_| Utc::now()),
    };

    match state.store.update_session(&session_id, update) {
        Some(session) => {
            let session_json = serde_json::to_value(&session).unwrap_or_default();
            state.broadcaster.emit_session_updated(&session.directory, &session_json);
            Json(session).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Session not found"
        }))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub status: Option<String>,
    pub title: Option<String>,
    pub summary: Option<crate::models::SessionSummary>,
    pub archived: Option<bool>,
}

/// Delete session
async fn delete_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if state.store.delete_session(&session_id) {
        (StatusCode::NO_CONTENT, ()).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Session not found"
        }))).into_response()
    }
}

// ─── Message Handlers ──────────────────────────────────────────────────────────

/// List messages for a session
async fn list_messages_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let messages = state.store.list_messages(&session_id);
    Json(messages)
}

/// Create message
async fn create_message_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<CreateMessageRequest>,
) -> impl IntoResponse {
    // Get session to ensure it exists and get directory
    let session = match state.store.get_session(&session_id) {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Session not found"
            }))).into_response()
        }
    };

    let message = match req.role.as_str() {
        "user" => {
            let content = req.content.into_iter().map(|b| {
                match b {
                    crate::models::ContentBlock::Text { id, text } => ContentBlock::text(id, text),
                    crate::models::ContentBlock::Thinking { id, thinking } => ContentBlock::thinking(id, thinking),
                    crate::models::ContentBlock::ToolUse { id, name, input } => ContentBlock::tool_use(id, name, input),
                    crate::models::ContentBlock::ToolResult { id, tool_use_id, content } => ContentBlock::tool_result(id, tool_use_id, content),
                }
            }).collect();
            
            let msg = crate::models::UserMessage::new(
                session_id.clone(),
                req.parent_id,
                content,
            );
            Message::User(msg)
        }
        "assistant" => {
            let msg = AssistantMessage::new(
                session_id.clone(),
                req.parent_id,
                req.model_id.unwrap_or_else(|| "claude-sonnet-4-5".to_string()),
                req.provider_id.unwrap_or_else(|| "anthropic".to_string()),
            );
            Message::Assistant(msg)
        }
        _ => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid role. Must be 'user' or 'assistant'"
            }))).into_response()
        }
    };

    match state.store.create_message(&session_id, message.clone()) {
        Some(msg) => {
            let msg_json = serde_json::to_value(&msg).unwrap_or_default();
            state.broadcaster.emit_message_updated(&session.directory, &session_id, &msg_json);
            (StatusCode::CREATED, Json(msg)).into_response()
        }
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to create message"
        }))).into_response(),
    }
}

/// Update message
async fn update_message_handler(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
    Json(req): Json<UpdateMessageRequest>,
) -> impl IntoResponse {
    let session = match state.store.get_session(&session_id) {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Session not found"
            }))).into_response()
        }
    };

    use crate::store::MessageUpdate;
    let update = MessageUpdate {
        completed: req.completed,
        cost: req.cost,
        tokens: req.tokens,
        error: req.error,
    };

    match state.store.update_message(&session_id, &message_id, update) {
        Some(msg) => {
            let msg_json = serde_json::to_value(&msg).unwrap_or_default();
            state.broadcaster.emit_message_updated(&session.directory, &session_id, &msg_json);
            Json(msg).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Message not found"
        }))).into_response(),
    }
}

/// Delete message
async fn delete_message_handler(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let session = match state.store.get_session(&session_id) {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Session not found"
            }))).into_response()
        }
    };

    if state.store.delete_message(&session_id, &message_id) {
        state.broadcaster.emit_message_removed(&session.directory, &session_id, &message_id);
        (StatusCode::NO_CONTENT, ()).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Message not found"
        }))).into_response()
    }
}

// ─── Part Handlers ─────────────────────────────────────────────────────────────

/// Add part to message
async fn add_part_handler(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
    Json(req): Json<AddPartRequest>,
) -> impl IntoResponse {
    // Verify message exists
    match state.store.get_message(&session_id, &message_id) {
        Some(_) => {}
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Message not found"
            }))).into_response()
        }
    };

    let session = state.store.get_session(&session_id).unwrap();

    let part = match req.part_type.as_str() {
        "text" => Part::Text {
            id: format!("part_{}", uuid::Uuid::new_v4()),
            message_id: message_id.clone(),
            text: req.content.unwrap_or_default(),
            time: None,
        },
        "thinking" => Part::Thinking {
            id: format!("part_{}", uuid::Uuid::new_v4()),
            message_id: message_id.clone(),
            thinking: req.content.unwrap_or_default(),
        },
        "tool_use" => Part::ToolUse {
            id: format!("part_{}", uuid::Uuid::new_v4()),
            message_id: message_id.clone(),
            name: req.name.unwrap_or_default(),
            input: req.input.unwrap_or(serde_json::json!({})),
        },
        "tool_result" => Part::ToolResult {
            id: format!("part_{}", uuid::Uuid::new_v4()),
            message_id: message_id.clone(),
            tool_use_id: req.tool_use_id.unwrap_or_default(),
            content: req.content.unwrap_or_default(),
        },
        _ => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid part type"
            }))).into_response()
        }
    };

    match state.store.add_part(&message_id, part.clone()) {
        Some(p) => {
            let part_json = serde_json::to_value(&p).unwrap_or_default();
            state.broadcaster.emit_part_updated(&session.directory, &part_json);
            (StatusCode::CREATED, Json(p)).into_response()
        }
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to add part"
        }))).into_response(),
    }
}

/// List parts for a message
async fn list_parts_handler(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> impl IntoResponse {
    let parts = state.store.list_parts(&message_id);
    Json(parts)
}

// ─── Router Builder ───────────────────────────────────────────────────────────

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/events", get(sse_handler))
        .route("/session", get(list_sessions_handler))
        .route("/session", post(create_session_handler))
        .route("/session/:session_id", get(get_session_handler))
        .route("/session/:session_id", patch(update_session_handler))
        .route("/session/:session_id", delete(delete_session_handler))
        .route("/session/:session_id/message", get(list_messages_handler))
        .route("/session/:session_id/message", post(create_message_handler))
        .route("/session/:session_id/message/:message_id", patch(update_message_handler))
        .route("/session/:session_id/message/:message_id", delete(delete_message_handler))
        .route("/session/:session_id/message/:message_id/part", get(list_parts_handler))
        .route("/session/:session_id/message/:message_id/part", post(add_part_handler))
        .layer(cors)
        .with_state(state)
}
