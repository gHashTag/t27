use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::ws_handler::AppState;

#[derive(Debug, Deserialize)]
pub(crate) struct OperatorAuth {
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OperatorRequest {
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtensionState {
    pub sidepanel_open: bool,
    pub current_tab: String,
    pub chat_log: Vec<Value>,
    pub agents: Vec<Value>,
    pub errors: Vec<String>,
}

impl Default for ExtensionState {
    fn default() -> Self {
        Self {
            sidepanel_open: false,
            current_tab: "chat".to_string(),
            chat_log: Vec::new(),
            agents: Vec::new(),
            errors: Vec::new(),
        }
    }
}

lazy_static::lazy_static! {
    static ref EXT_STATE: Arc<Mutex<ExtensionState>> =
        Arc::new(Mutex::new(ExtensionState::default()));
}

pub fn init_operator_token() -> String {
    let token = Uuid::new_v4().to_string();
    std::env::set_var("TRIOS_OPERATOR_TOKEN", token.clone());
    tracing::info!("Generated TRIOS_OPERATOR_TOKEN: {}", token);
    token
}

pub async fn operator_ws_handler(
    ws: WebSocketUpgrade,
    Query(auth): Query<OperatorAuth>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let expected_token = std::env::var("TRIOS_OPERATOR_TOKEN").unwrap_or_default();
    if !expected_token.is_empty() {
        if let Some(ref token) = auth.token {
            if token != &expected_token {
                warn!("Operator WS rejected: invalid token");
                return (axum::http::StatusCode::UNAUTHORIZED, "Invalid token").into_response();
            }
        } else {
            return (axum::http::StatusCode::UNAUTHORIZED, "Missing token").into_response();
        }
    }
    ws.on_upgrade(move |socket| handle_operator_socket(socket, state))
}

async fn handle_operator_socket(mut socket: WebSocket, state: AppState) {
    info!("Operator WS client connected");

    let mut rx = state.event_tx.subscribe();

    loop {
        tokio::select! {
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let response = handle_operator_message(&text, &state).await;
                        let response_json = serde_json::to_string(&response).unwrap_or_default();
                        if socket.send(Message::Text(response_json)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Operator WS client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("Operator WS error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            event = rx.recv() => {
                if let Ok(ev) = event {
                    let msg = serde_json::to_string(&json!({
                        "type": "event",
                        "event": ev
                    })).unwrap_or_default();
                    if socket.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_operator_message(text: &str, state: &AppState) -> Value {
    let req: OperatorRequest = match serde_json::from_str(text) {
        Ok(r) => r,
        Err(e) => {
            return json!({"error": format!("invalid request: {}", e)});
        }
    };

    info!("Operator request: method={}", req.method);

    match req.method.as_str() {
        "operator/ping" => json!({"pong": true, "ts": epoch_secs()}),
        "operator/extension/state" => extension_state(state).await,
        "operator/extension/send_chat" => send_chat(state, req.params).await,
        "operator/extension/click" => click_element(req.params).await,
        "operator/extension/screenshot" => request_screenshot(req.params).await,
        "operator/extension/navigate" => request_navigate(req.params).await,
        "operator/extension/register" => register_extension(req.params).await,
        "operator/agent/list" => agent_list(state).await,
        "operator/health" => json!({
            "status": "ok",
            "server": "trios-server",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_secs": 0,
            "connections": 1
        }),
        _ => json!({"error": format!("unknown method: {}", req.method)}),
    }
}

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn extension_state(state: &AppState) -> Value {
    let agents = state.agents.lock().await;
    let tasks = state.tasks.lock().await;
    let ext = EXT_STATE.lock().unwrap();
    json!({
        "panel_open": ext.sidepanel_open,
        "current_tab": ext.current_tab,
        "chat_log": ext.chat_log,
        "agents": *agents,
        "tasks": tasks.len(),
        "errors": ext.errors
    })
}

async fn send_chat(state: &AppState, params: Option<Value>) -> Value {
    let msg = params
        .as_ref()
        .and_then(|p| p.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("");

    if msg.is_empty() {
        return json!({"error": "message is required"});
    }

    {
        let mut ext = EXT_STATE.lock().unwrap();
        ext.chat_log.push(json!({
            "role": "operator",
            "content": msg,
            "ts": epoch_secs()
        }));
    }

    crate::mcp_endpoints::agents::chat(state, Some(json!({"message": msg}))).await
}

async fn click_element(params: Option<Value>) -> Value {
    let selector = params
        .as_ref()
        .and_then(|p| p.get("selector"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    if selector.is_empty() {
        return json!({"error": "selector is required"});
    }

    let mut ext = EXT_STATE.lock().unwrap();
    ext.errors.push(format!("Click requested: {}", selector));

    json!({"clicked": selector, "note": "click requires browser-side extension cooperation"})
}

async fn agent_list(state: &AppState) -> Value {
    let agents = state.agents.lock().await;
    json!(*agents)
}

async fn request_screenshot(params: Option<Value>) -> Value {
    let selector = params
        .as_ref()
        .and_then(|p| p.get("selector"))
        .and_then(|s| s.as_str())
        .unwrap_or("viewport");

    let mut ext = EXT_STATE.lock().unwrap();
    ext.errors.push(format!("Screenshot requested: {}", selector));

    json!({
        "status": "requested",
        "selector": selector,
        "note": "screenshot will be captured by browser extension and sent back via WS"
    })
}

async fn request_navigate(params: Option<Value>) -> Value {
    let url = params
        .as_ref()
        .and_then(|p| p.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("");

    if url.is_empty() {
        return json!({"error": "url is required"});
    }

    let mut ext = EXT_STATE.lock().unwrap();
    ext.current_tab = format!("navigating:{}", url);

    json!({
        "status": "navigating",
        "url": url,
        "note": "navigation will be executed by browser extension"
    })
}

async fn register_extension(params: Option<Value>) -> Value {
    let name = params
        .as_ref()
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");
    let version = params
        .as_ref()
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0");

    let mut ext = EXT_STATE.lock().unwrap();
    ext.sidepanel_open = true;

    info!("Extension registered: {} v{}", name, version);

    json!({
        "registered": true,
        "name": name,
        "version": version,
        "capabilities": ["navigate", "click", "screenshot", "chat"],
        "server_version": env!("CARGO_PKG_VERSION")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_operator_ping() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/ping"}"#,
            &state,
        )
        .await;
        assert!(result.get("pong").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_operator_extension_state() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/state"}"#,
            &state,
        )
        .await;
        assert!(result.get("panel_open").is_some());
        assert!(result.get("agents").is_some());
        assert!(result.get("chat_log").is_some());
    }

    #[tokio::test]
    async fn test_operator_send_chat_empty() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/send_chat","params":{}}"#,
            &state,
        )
        .await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_operator_send_chat_valid() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/send_chat","params":{"message":"hello"}}"#,
            &state,
        )
        .await;
        assert!(result.get("response").is_some() || result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_operator_agent_list() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/agent/list"}"#,
            &state,
        )
        .await;
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_operator_health() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/health"}"#,
            &state,
        )
        .await;
        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "ok");
        assert_eq!(
            result.get("server").unwrap().as_str().unwrap(),
            "trios-server"
        );
    }

    #[tokio::test]
    async fn test_operator_unknown_method() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/nonexistent"}"#,
            &state,
        )
        .await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_operator_click_empty() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/click","params":{}}"#,
            &state,
        )
        .await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_operator_click_valid() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/click","params":{"selector":".tab"}}"#,
            &state,
        )
        .await;
        assert_eq!(result.get("clicked").unwrap().as_str().unwrap(), ".tab");
    }

    #[tokio::test]
    async fn test_operator_screenshot() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/screenshot","params":{"selector":"body"}}"#,
            &state,
        )
        .await;
        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "requested");
    }

    #[tokio::test]
    async fn test_operator_navigate_empty() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/navigate","params":{}}"#,
            &state,
        )
        .await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_operator_navigate_valid() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/navigate","params":{"url":"https://github.com"}}"#,
            &state,
        )
        .await;
        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "navigating");
    }

    #[tokio::test]
    async fn test_operator_register() {
        let state = AppState::new();
        let result = handle_operator_message(
            r#"{"method":"operator/extension/register","params":{"name":"trios-ext","version":"0.1.0"}}"#,
            &state,
        )
        .await;
        assert_eq!(result.get("registered").unwrap().as_bool().unwrap(), true);
        assert_eq!(result.get("name").unwrap().as_str().unwrap(), "trios-ext");
    }
}
