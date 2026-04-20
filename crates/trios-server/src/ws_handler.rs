use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

use crate::mcp::McpService;
use crate::mcp_endpoints;

/// Event types broadcasted to all connected clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BusEvent {
    TaskAssigned { task_id: String, agent_id: String },
    TaskUpdated { task_id: String, status: String },
    AgentConnected { agent_id: String },
    AgentDisconnected { agent_id: String },
}

#[derive(Clone)]
pub struct AppState {
    pub mcp: McpService,
    pub agents: Arc<Mutex<Vec<AgentState>>>,
    pub tasks: Arc<Mutex<Vec<TaskEntry>>>,
    /// Broadcast channel for event streaming to all connected clients
    pub event_tx: broadcast::Sender<BusEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntry {
    pub id: String,
    pub agent_id: String,
    pub task: String,
    pub status: String,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            mcp: McpService::new(),
            agents: Arc::new(Mutex::new(Vec::new())),
            tasks: Arc::new(Mutex::new(Vec::new())),
            event_tx: tx,
        }
    }

    /// Broadcast an event to all connected clients
    pub fn broadcast_event(&self, event: BusEvent) {
        // Errors here mean no receivers, which is fine
        let _ = self.event_tx.send(event);
    }
}

#[derive(Debug, Deserialize)]
struct WsRequest {
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct WsResponse {
    result: Value,
}

pub async fn ws_handler(ws: WebSocketUpgrade, axum::extract::State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    info!("WebSocket client connected");

    // Subscribe to event broadcasts
    let mut event_rx = state.event_tx.subscribe();

    loop {
        tokio::select! {
            // Handle incoming requests
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let response = handle_message(&text, &state).await;
                        let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                            serde_json::to_string(&WsResponse {
                                result: json!({"error": format!("serialize error: {}", e)}),
                            }).unwrap_or_default()
                        });
                        if socket.send(Message::Text(response_json)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None | Some(Ok(_)) => {}
                }
            }
            // Handle broadcast events
            event_result = event_rx.recv() => {
                match event_result {
                    Ok(event) => {
                        let event_json = serde_json::to_string(&json!({"event": event})).unwrap_or_default();
                        if socket.send(Message::Text(event_json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        info!("Event channel lagged, skipped {} events", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Event channel closed");
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_message(text: &str, state: &AppState) -> WsResponse {
    let req: WsRequest = match serde_json::from_str(text) {
        Ok(r) => r,
        Err(e) => {
            return WsResponse {
                result: json!({"error": format!("invalid request: {}", e)}),
            }
        }
    };

    info!("WS request: method={}", req.method);

    let result = match req.method.as_str() {
        "agents/list" => mcp_endpoints::agents::list(state).await,
        "agents/chat" => mcp_endpoints::agents::chat(state, req.params).await,
        "tasks/assign" => mcp_endpoints::tasks::assign(state, req.params).await,
        "tasks/status" => mcp_endpoints::tasks::status(state, req.params).await,
        "tasks/update_status" => mcp_endpoints::tasks::update_status(state, req.params).await,
        "experience/read" => mcp_endpoints::experience::read(state, req.params).await,
        "tools/list" => tools_list(state).await,
        "tools/call" => tools_call(state, req.params).await,
        _ => json!({"error": format!("unknown method: {}", req.method)}),
    };

    WsResponse { result }
}

async fn tools_list(state: &AppState) -> Value {
    let list = state.mcp.list_tools();
    serde_json::to_value(list.tools).unwrap_or(json!({"error": "serialize failed"}))
}

async fn tools_call(state: &AppState, params: Option<Value>) -> Value {
    let params_val = params.unwrap_or(json!({}));
    let tool_name = params_val.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params_val.get("arguments").cloned();

    use rust_mcp_schema::CallToolRequestParams;
    let call_params = CallToolRequestParams {
        name: tool_name.to_string(),
        arguments: arguments.and_then(|a| {
            if a.is_object() {
                Some(a.as_object().cloned().unwrap_or_default())
            } else {
                None
            }
        }),
        meta: None,
        task: None,
    };

    let result = state.mcp.call_tool(call_params).await;
    serde_json::to_value(result).unwrap_or(json!({"error": "serialize failed"}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agents_list_empty() {
        let state = AppState::new();
        let result = mcp_endpoints::agents::list(&state).await;
        assert_eq!(result, json!([]));
    }

    #[tokio::test]
    async fn test_tasks_assign() {
        let state = AppState::new();
        let params = json!({"agent_id": "agent-1", "task": "test task", "context": "test"});
        let result = mcp_endpoints::tasks::assign(&state, Some(params)).await;
        assert!(result.get("task_id").is_some());
    }

    #[tokio::test]
    async fn test_tasks_status_not_found() {
        let state = AppState::new();
        let params = json!({"task_id": "nonexistent"});
        let result = mcp_endpoints::tasks::status(&state, Some(params)).await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_tasks_update_status_broadcasts_event() {
        let state = AppState::new();
        let mut rx = state.event_tx.subscribe();

        // First assign a task
        let params = json!({"agent_id": "agent-1", "task": "test task"});
        let assign_result = mcp_endpoints::tasks::assign(&state, Some(params.clone())).await;
        let task_id = assign_result.get("task_id").unwrap().as_str().unwrap();

        // Check for TaskAssigned event
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv()
        ).await;
        assert!(event.is_ok(), "Should receive TaskAssigned event");
        let event = event.unwrap().unwrap();
        match event {
            BusEvent::TaskAssigned { task_id: evt_id, .. } => {
                assert_eq!(evt_id, task_id);
            }
            _ => panic!("Expected TaskAssigned event"),
        }

        // Update task status
        let update_params = json!({"task_id": task_id, "status": "completed"});
        mcp_endpoints::tasks::update_status(&state, Some(update_params)).await;

        // Check for TaskUpdated event
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv()
        ).await;
        assert!(event.is_ok(), "Should receive TaskUpdated event");
        let event = event.unwrap().unwrap();
        match event {
            BusEvent::TaskUpdated { task_id: evt_id, status } => {
                assert_eq!(evt_id, task_id);
                assert_eq!(status, "completed");
            }
            _ => panic!("Expected TaskUpdated event"),
        }
    }

    #[tokio::test]
    async fn test_experience_read() {
        let state = AppState::new();
        let result = mcp_endpoints::experience::read(&state, None).await;
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_tools_list_returns_array() {
        let state = AppState::new();
        let result = tools_list(&state).await;
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let state = AppState::new();
        let response = handle_message(r#"{"method":"unknown","params":{}}"#, &state).await;
        assert!(response.result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_handle_invalid_json() {
        let state = AppState::new();
        let response = handle_message("not json", &state).await;
        assert!(response.result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_broadcast_event() {
        let state = AppState::new();
        let mut rx = state.event_tx.subscribe();

        let event = BusEvent::AgentConnected { agent_id: "test-agent".to_string() };
        state.broadcast_event(event.clone());

        let received = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv()
        ).await;
        assert!(received.is_ok());
        let received = received.unwrap().unwrap();
        match received {
            BusEvent::AgentConnected { agent_id } => {
                assert_eq!(agent_id, "test-agent");
            }
            _ => panic!("Expected AgentConnected event"),
        }
    }

    #[tokio::test]
    async fn test_tasks_update_status_requires_task_id() {
        let state = AppState::new();
        let params = json!({"status": "completed"});
        let result = mcp_endpoints::tasks::update_status(&state, Some(params)).await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_tasks_update_status_requires_status() {
        let state = AppState::new();
        let params = json!({"task_id": "task-123"});
        let result = mcp_endpoints::tasks::update_status(&state, Some(params)).await;
        assert!(result.get("error").is_some());
    }
}
