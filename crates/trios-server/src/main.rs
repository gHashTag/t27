mod mcp;
mod mcp_endpoints;
mod operator;
mod security;
mod tools;
mod ws_handler;

use axum::Router;
use axum::response::Json;
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;
use ws_handler::AppState;

#[derive(Deserialize)]
struct McpRequest {
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("trios_server=debug,tower_http=debug")
        .init();

    let operator_token = operator::init_operator_token();
    info!("Operator token: {}", operator_token);

    let state = AppState::new();
    let port: u16 = std::env::var("TRIOS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9005);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler::ws_handler))
        .route("/operator", get(operator::operator_ws_handler))
        .route("/mcp", post(mcp_http_handler))
        .route("/health", get(health))
        .route("/", get(health))
        .layer(cors)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(security::auth_middleware))
                .layer(axum::middleware::from_fn(security::timeout_middleware)),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("trios-server listening on ws://0.0.0.0:{}/ws", port);
    info!("MCP HTTP: http://0.0.0.0:{}/mcp", port);
    info!("Operator bridge: ws://0.0.0.0:{}/operator?token=...", port);
    info!("MCP tools: {count} registered", count = tools::count());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn mcp_http_handler(Json(req): Json<McpRequest>) -> Json<Value> {
    match req.method.as_str() {
        "tools/list" => {
            let result = mcp::McpService.list_tools();
            Json(json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": 1
            }))
        }
        "tools/call" => {
            let tool_name = req.params
                .as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tool_input = req.params
                .as_ref()
                .and_then(|p| p.get("arguments").cloned())
                .unwrap_or(json!({}));
            match tools::dispatch(tool_name, tool_input).await {
                Ok(value) => Json(json!({
                    "jsonrpc": "2.0",
                    "result": { "content": [{ "type": "text", "text": serde_json::to_string(&value).unwrap_or_default() }], "isError": false },
                    "id": 1
                })),
                Err(e) => Json(json!({
                    "jsonrpc": "2.0",
                    "result": { "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true },
                    "id": 1
                })),
            }
        }
        _ => Json(json!({
            "jsonrpc": "2.0",
            "error": { "code": -32601, "message": format!("Method not found: {}", req.method) },
            "id": 1
        })),
    }
}
