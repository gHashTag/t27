use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::signal;
use tokio::time::{timeout, Duration};
use tracing::info;
use tracing_subscriber::EnvFilter;

const MAX_CAPTURE: usize = 256 * 1024;
const DEFAULT_EXEC_TIMEOUT_SECS: u64 = 120;
const DEFAULT_READ_MAX_BYTES: u64 = 1024 * 1024;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) token: String,
    pub(crate) workspace_root: PathBuf,
    pub(crate) home_dir: PathBuf,
}

pub(crate) fn path_allowed(p: &Path, state: &AppState) -> bool {
    let canonical = match std::fs::canonicalize(p) {
        Ok(c) => c,
        Err(_) => match p.parent() {
            Some(parent) => match std::fs::canonicalize(parent) {
                Ok(c) => c.join(p.file_name().unwrap_or_default()),
                Err(_) => return false,
            },
            None => return false,
        },
    };

    for allowed in &[&state.home_dir, &PathBuf::from("/tmp"), &state.workspace_root] {
        if canonical.starts_with(allowed) {
            return true;
        }
    }
    false
}

pub(crate) fn check_auth(headers: &HeaderMap, token: &str) -> Result<(), StatusCode> {
    match headers.get("X-Trios-Token") {
        Some(v) => match v.to_str() {
            Ok(s) if s == token => Ok(()),
            _ => Err(StatusCode::UNAUTHORIZED),
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Serialize)]
struct HealthResp {
    ok: bool,
    host: String,
    cwd: String,
    version: String,
}

async fn get_health(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<HealthResp>, StatusCode> {
    check_auth(&headers, &state.token)?;
    let host = gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "unknown".into());
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    Ok(Json(HealthResp {
        ok: true,
        host,
        cwd,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

#[derive(Deserialize)]
struct ExecReq {
    cmd: String,
    cwd: Option<String>,
    timeout_secs: Option<u64>,
}

#[derive(Serialize)]
struct ExecResp {
    exit_code: i32,
    stdout: String,
    stderr: String,
    truncated: bool,
}

async fn post_exec(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ExecReq>,
) -> Result<(StatusCode, Json<ExecResp>), StatusCode> {
    check_auth(&headers, &state.token)?;

    let secs = body.timeout_secs.unwrap_or(DEFAULT_EXEC_TIMEOUT_SECS);
    let cwd = body.cwd.as_deref().unwrap_or("/");

    let child_result = Command::new("bash")
        .arg("-lc")
        .arg(&body.cmd)
        .current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    let mut child = match child_result {
        Ok(c) => c,
        Err(e) => {
            return Ok((
                StatusCode::OK,
                Json(ExecResp {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    truncated: false,
                }),
            ));
        }
    };

    let result = timeout(Duration::from_secs(secs), async {
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        if let Some(mut out) = child.stdout.take() {
            let mut tmp = [0u8; 8192];
            loop {
                match out.read(&mut tmp).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if stdout_buf.len() + n <= MAX_CAPTURE {
                            stdout_buf.extend_from_slice(&tmp[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        if let Some(mut err) = child.stderr.take() {
            let mut tmp = [0u8; 8192];
            loop {
                match err.read(&mut tmp).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if stderr_buf.len() + n <= MAX_CAPTURE {
                            stderr_buf.extend_from_slice(&tmp[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        let status = child.wait().await;
        (stdout_buf, stderr_buf, status)
    })
    .await;

    match result {
        Ok((stdout_bytes, stderr_bytes, status_res)) => {
            let exit_code = status_res.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
            let (stdout_str, trunc_out) = truncate_bytes(&stdout_bytes, MAX_CAPTURE);
            let (stderr_str, trunc_err) = truncate_bytes(&stderr_bytes, MAX_CAPTURE);
            Ok((
                StatusCode::OK,
                Json(ExecResp {
                    exit_code,
                    stdout: stdout_str,
                    stderr: stderr_str,
                    truncated: trunc_out || trunc_err,
                }),
            ))
        }
        Err(_) => {
            let _ = child.kill().await;
            Ok((
                StatusCode::OK,
                Json(ExecResp {
                    exit_code: -9,
                    stdout: String::new(),
                    stderr: format!("timed out after {}s", secs),
                    truncated: false,
                }),
            ))
        }
    }
}

fn truncate_bytes(data: &[u8], max: usize) -> (String, bool) {
    if data.len() > max {
        (String::from_utf8_lossy(&data[..max]).into_owned(), true)
    } else {
        (String::from_utf8_lossy(data).into_owned(), false)
    }
}

#[derive(Deserialize)]
struct ReadReq {
    path: String,
    max_bytes: Option<u64>,
}

#[derive(Serialize)]
struct ReadResp {
    path: String,
    size: u64,
    content_b64: String,
    truncated: bool,
}

type ApiError = (StatusCode, Json<serde_json::Value>);

async fn post_read(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ReadReq>,
) -> Result<(StatusCode, Json<ReadResp>), ApiError> {
    check_auth(&headers, &state.token).map_err(|e| (e, Json(serde_json::json!({"error":"unauthorized"}))))?;

    let p = PathBuf::from(&body.path);
    if !path_allowed(&p, &state) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error":"path outside allowlist"})),
        ));
    }

    let max_bytes = body.max_bytes.unwrap_or(DEFAULT_READ_MAX_BYTES) as usize;
    let result = tokio::fs::read(&p).await;
    match result {
        Ok(data) => {
            let size = data.len() as u64;
            let truncated = data.len() > max_bytes;
            let slice = if truncated { &data[..max_bytes] } else { &data };
            Ok((
                StatusCode::OK,
                Json(ReadResp {
                    path: body.path,
                    size,
                    content_b64: base64::engine::general_purpose::STANDARD.encode(slice),
                    truncated,
                }),
            ))
        }
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

#[derive(Deserialize)]
struct WriteReq {
    path: String,
    content_b64: String,
    mkdirs: Option<bool>,
}

#[derive(Serialize)]
struct WriteResp {
    path: String,
    bytes: u64,
}

async fn post_write(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<WriteReq>,
) -> Result<(StatusCode, Json<WriteResp>), ApiError> {
    check_auth(&headers, &state.token).map_err(|e| (e, Json(serde_json::json!({"error":"unauthorized"}))))?;

    let p = PathBuf::from(&body.path);
    if !path_allowed(&p, &state) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error":"path outside allowlist"})),
        ));
    }

    let data: Vec<u8> = match base64::engine::general_purpose::STANDARD.decode(&body.content_b64) {
        Ok(d) => d,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("base64 decode: {}", e)})),
            ));
        }
    };

    if body.mkdirs == Some(true) {
        if let Some(parent) = p.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
            })?;
        }
    }

    tokio::fs::write(&p, &data).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(WriteResp {
            path: body.path,
            bytes: data.len() as u64,
        }),
    ))
}

#[derive(Serialize)]
struct TailResp {
    path: String,
    lines: Vec<String>,
}

async fn get_tail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<(StatusCode, Json<TailResp>), ApiError> {
    check_auth(&headers, &state.token).map_err(|e| (e, Json(serde_json::json!({"error":"unauthorized"}))))?;

    let path_str = params.get("path").ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error":"missing path"})),
    ))?;
    let n: usize = params
        .get("lines")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let p = PathBuf::from(path_str);
    if !path_allowed(&p, &state) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error":"path outside allowlist"})),
        ));
    }

    let output = Command::new("tail")
        .arg(format!("-n{}", n))
        .arg(&p)
        .output()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    let content = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    Ok((
        StatusCode::OK,
        Json(TailResp {
            path: path_str.clone(),
            lines,
        }),
    ))
}

async fn logging_middleware(
    req: axum::http::Request<Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();
    let resp = next.run(req).await;
    let elapsed = start.elapsed();
    let status = resp.status();
    info!(method = %method, path = %path, status = %status, elapsed_ms = elapsed.as_millis() as u64);
    resp
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C"),
        _ = terminate => info!("received SIGTERM"),
    }
}

pub(crate) fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(get_health))
        .route("/exec", post(post_exec))
        .route("/read", post(post_read))
        .route("/write", post(post_write))
        .route("/tail", get(get_tail))
        .layer(middleware::from_fn(logging_middleware))
        .with_state(state)
}

fn resolve_workspace_root() -> PathBuf {
    std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8(o.stdout).ok()?;
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let token = std::env::var("TRIOS_BRIDGE_TOKEN").expect("TRIOS_BRIDGE_TOKEN env var required");
    let workspace_root = resolve_workspace_root();
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

    let state = Arc::new(AppState {
        token,
        workspace_root,
        home_dir,
    });

    let app = build_app(state);
    let addr: SocketAddr = "127.0.0.1:7878".parse().unwrap();
    info!("trios-bridge listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body as AxumBody;
    use axum::http::{Request, StatusCode as SC};
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            token: "test-secret-token".to_string(),
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp")),
            home_dir: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")),
        })
    }

    fn auth_header() -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            "X-Trios-Token",
            HeaderValue::from_static("test-secret-token"),
        );
        h
    }

    fn no_auth_header() -> HeaderMap {
        HeaderMap::new()
    }

    #[tokio::test]
    async fn health_happy_path() {
        let state = test_state();
        let app = build_app(state);
        let req = Request::builder()
            .uri("/health")
            .method("GET")
            .header("X-Trios-Token", "test-secret-token")
            .body(AxumBody::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), SC::OK);
        let body = axum::body::to_bytes(resp.into_body(), 4096)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["ok"], true);
        assert!(v["host"].is_string());
        assert!(v["version"].is_string());
    }

    #[tokio::test]
    async fn health_401_without_token() {
        let state = test_state();
        let app = build_app(state);
        let req = Request::builder()
            .uri("/health")
            .method("GET")
            .body(AxumBody::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), SC::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn exec_echo_hi() {
        let state = test_state();
        let app = build_app(state);
        let body = serde_json::json!({"cmd": "echo hi"});
        let req = Request::builder()
            .uri("/exec")
            .method("POST")
            .header("X-Trios-Token", "test-secret-token")
            .header("content-type", "application/json")
            .body(AxumBody::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), SC::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 65536)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["exit_code"], 0);
        let stdout = v["stdout"].as_str().unwrap();
        assert!(
            stdout.contains("hi"),
            "expected stdout to contain 'hi', got: {:?}",
            stdout
        );
    }

    #[tokio::test]
    async fn exec_timeout_kills_sleep() {
        let state = test_state();
        let app = build_app(state);
        let body = serde_json::json!({"cmd": "sleep 60", "timeout_secs": 1});
        let req = Request::builder()
            .uri("/exec")
            .method("POST")
            .header("X-Trios-Token", "test-secret-token")
            .header("content-type", "application/json")
            .body(AxumBody::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), SC::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 65536)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["exit_code"], -9);
        assert!(
            v["stderr"]
                .as_str()
                .unwrap()
                .contains("timed out after 1s")
        );
    }

    #[tokio::test]
    async fn read_refuses_outside_allowlist() {
        let state = test_state();
        let app = build_app(state);
        let body = serde_json::json!({"path": "/etc/passwd"});
        let req = Request::builder()
            .uri("/read")
            .method("POST")
            .header("X-Trios-Token", "test-secret-token")
            .header("content-type", "application/json")
            .body(AxumBody::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), SC::FORBIDDEN);
    }
}
