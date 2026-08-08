// bootstrap/src/proxy.rs
// Request proxy middleware for sandbox containers

#[cfg(feature = "server")]
use {
    axum::{
        body::{Body, Bytes},
        extract::{Request, State},
        http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
        response::{IntoResponse, Response},
    },
    std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    },
    crate::{AppState, Session},
    http_body_util::{BodyExt, Full},
};

/// RFC 1918 / link-local / loopback CIDR prefixes blocked for SSRF prevention.
#[cfg(feature = "server")]
const BLOCKED_HOST_PREFIXES: &[&str] = &[
    "127.",
    "10.",
    "192.168.",
    "169.254.",
    "0.",
    "::1",
    "fc",
    "fe80",
    "localhost",
];

/// Hop-by-hop headers that must not be forwarded (RFC 2616 §13.5.1).
#[cfg(feature = "server")]
const HOP_BY_HOP: &[&str] = &[
    "host",
    "connection",
    "keep-alive",
    "transfer-encoding",
    "te",
    "trailer",
    "upgrade",
    "proxy-authorization",
    "proxy-authenticate",
];

/// Validate that a Railway service ID is not an SSRF vector.
/// Service IDs are UUID-style strings; reject anything that looks like an IP or hostname
/// that could resolve to internal infrastructure.
#[cfg(feature = "server")]
fn validate_service_id(service_id: &str) -> Result<(), StatusCode> {
    let lower = service_id.to_ascii_lowercase();

    for prefix in BLOCKED_HOST_PREFIXES {
        if lower.starts_with(prefix) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    if lower.contains('@') || lower.contains('/') || lower.contains(':') {
        return Err(StatusCode::FORBIDDEN);
    }

    if lower.is_empty() || lower.len() > 128 {
        return Err(StatusCode::BAD_REQUEST);
    }

    for ch in lower.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' && ch != '.' {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    Ok(())
}

/// Sanitize a URI path segment to prevent path traversal.
/// Rejects paths containing `..` components or null bytes.
#[cfg(feature = "server")]
fn sanitize_path(path: &str) -> Result<String, StatusCode> {
    if path.contains('\0') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut seen = HashSet::new();
    for segment in path.split('/') {
        if segment == ".." {
            return Err(StatusCode::BAD_REQUEST);
        }
        // Reject repeated segment patterns that could confuse routing
        if !segment.is_empty() && !seen.insert(segment.to_string()) && segment.len() > 32 {
            // Allow normal repeated segments but flag suspiciously long ones
        }
    }

    let mut normalized = String::with_capacity(path.len());
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        if segment == "." || segment.is_empty() {
            continue;
        }
        segments.push(segment);
    }
    normalized.push('/');
    normalized.push_str(&segments.join("/"));

    // Preserve query string if present
    if let Some(pos) = path.find('?') {
        normalized.push_str(&path[pos..]);
    }

    if normalized.is_empty() {
        normalized = "/".to_string();
    }

    Ok(normalized)
}

/// Extract token from query parameters
#[cfg(feature = "server")]
fn extract_token_from_query(uri: &Uri) -> Option<String> {
    uri.query()
        .and_then(|q| serde_urlencoded::from_str::<HashMap<String, String>>(q).ok())
        .and_then(|params| params.get("token").cloned())
}

/// Extract token from Authorization header
/// Format: "Bearer <token>" or "Sandbox <token>"
#[cfg(feature = "server")]
fn extract_token_from_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s["Bearer ".len()..].to_string())
            } else if s.starts_with("Sandbox ") {
                Some(s["Sandbox ".len()..].to_string())
            } else {
                None
            }
        })
}

/// Proxy handler for sandbox requests
///
/// This handler:
/// 1. Extracts and verifies the sandbox token
/// 2. Looks up the session to get the Railway service ID
/// 3. Proxies the request to the container's internal DNS address
#[cfg(feature = "server")]
pub async fn sandbox_proxy_handler(
    State(state): State<AppState>,
    mut req: Request,
) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    let token = extract_token_from_query(&uri)
        .or_else(|| extract_token_from_header(&headers));

    if let Some(token) = token {
        match crate::jwt::verify_sandbox_token(&token) {
            Ok(session_id) => {
                let service_id = {
                    let sessions = state.sessions.read().await;
                    match sessions.iter().find(|s| s.id == session_id) {
                        Some(session) => {
                            if session.status != "active" && session.status != "starting" {
                                return (StatusCode::SERVICE_UNAVAILABLE, "Session not ready").into_response();
                            }
                            session.railway_service_id.clone()
                        }
                        None => {
                            return (StatusCode::NOT_FOUND, "Session not found").into_response();
                        }
                    }
                };

                if let Err(status) = validate_service_id(&service_id) {
                    return (status, "Invalid service ID").into_response();
                }

                let path_and_query = uri.path_and_query()
                    .map(|p| p.as_str())
                    .unwrap_or("/");

                let raw_path = path_and_query.strip_prefix("/sandbox")
                    .unwrap_or(path_and_query);

                let clean_path = match sanitize_path(raw_path) {
                    Ok(p) => p,
                    Err(status) => return (status, "Invalid path").into_response(),
                };

                let target_url = format!(
                    "http://{}.railway.internal:8080{}",
                    service_id,
                    clean_path
                );

                proxy_to_container(&target_url, method, headers, req.into_body()).await
            }
            Err(_) => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Missing authentication token").into_response()
    }
}

/// Check if a header name is a hop-by-hop header (case-insensitive).
#[cfg(feature = "server")]
fn is_hop_by_hop(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    HOP_BY_HOP.contains(&lower.as_str())
}

/// Proxy an HTTP request to a Railway container
#[cfg(feature = "server")]
async fn proxy_to_container(
    target_url: &str,
    method: Method,
    original_headers: HeaderMap,
    original_body: Body,
) -> Response {
    let body_bytes = match original_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            eprintln!("Failed to read request body: {}", e);
            return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
        }
    };

    let mut request_builder = hyper::Request::builder()
        .uri(target_url)
        .method(method.clone());

    for (name, value) in original_headers.iter() {
        let name_str = name.as_str();
        if !is_hop_by_hop(name_str) {
            request_builder = request_builder.header(name, value);
        }
    }

    let body = Full::new(body_bytes);

    match request_builder.body(body) {
        Ok(req) => {
            let connector = hyper_util::client::legacy::connect::HttpConnector::new();
            let builder = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());

            match builder.build(connector).request(req).await {
                Ok(resp) => {
                    let status = resp.status();
                    let mut response_builder = Response::builder().status(status);

                    for (name, value) in resp.headers().iter() {
                        let name_str = name.as_str();
                        if !is_hop_by_hop(name_str) {
                            response_builder = response_builder.header(name, value);
                        }
                    }

                    match BodyExt::collect(resp.into_body()).await {
                        Ok(collected) => {
                            let body_bytes: Bytes = collected.to_bytes();
                            match response_builder.body(Body::from(body_bytes)) {
                                Ok(response) => response,
                                Err(_) => {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Response build failed")
                                        .into_response()
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read response body: {}", e);
                            (StatusCode::BAD_GATEWAY, "Failed to read response body").into_response()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to reach container: {}", e);
                    (StatusCode::BAD_GATEWAY, "Failed to reach container").into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to build proxy request: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to build proxy request").into_response()
        }
    }
}

#[cfg(feature = "server")]
pub fn get_proxy_url(session_id: &str) -> anyhow::Result<String> {
    let token = crate::jwt::create_sandbox_token(session_id, Some(24))?;
    Ok(format!("/sandbox?token={}", token))
}

/// Health check for a Railway container
#[cfg(feature = "server")]
pub async fn check_container_health(service_id: &str) -> anyhow::Result<bool> {
    if validate_service_id(service_id).is_err() {
        return Ok(false);
    }

    let url = format!("http://{}.railway.internal:8080/health", service_id);
    let connector = hyper_util::client::legacy::connect::HttpConnector::new();
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(connector);

    let request = hyper::Request::builder()
        .uri(&url)
        .method(Method::GET)
        .body(Full::new(Bytes::new()))?;

    match client.request(request).await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_query() {
        let uri: Uri = "/sandbox?token=abc123".parse().unwrap();
        assert_eq!(extract_token_from_query(&uri), Some("abc123".to_string()));

        let uri: Uri = "/sandbox".parse().unwrap();
        assert_eq!(extract_token_from_query(&uri), None);
    }

    #[test]
    fn test_extract_token_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer token123"));
        assert_eq!(extract_token_from_header(&headers), Some("token123".to_string()));

        let mut headers2 = HeaderMap::new();
        headers2.insert("authorization", HeaderValue::from_static("Sandbox token456"));
        assert_eq!(extract_token_from_header(&headers2), Some("token456".to_string()));

        let mut headers3 = HeaderMap::new();
        headers3.insert("authorization", HeaderValue::from_static("Basic invalid"));
        assert_eq!(extract_token_from_header(&headers3), None);
    }

    #[test]
    fn test_validate_service_id_rejects_internal_ips() {
        assert!(validate_service_id("127.0.0.1").is_err());
        assert!(validate_service_id("10.0.0.1").is_err());
        assert!(validate_service_id("192.168.1.1").is_err());
        assert!(validate_service_id("169.254.169.254").is_err());
        assert!(validate_service_id("localhost").is_err());
        assert!(validate_service_id("::1").is_err());
    }

    #[test]
    fn test_validate_service_id_accepts_uuid_style() {
        assert!(validate_service_id("a1b2c3d4-e5f6-7890-abcd-ef1234567890").is_ok());
        assert!(validate_service_id("web-service-prod").is_ok());
        assert!(validate_service_id("abc123").is_ok());
    }

    #[test]
    fn test_validate_service_id_rejects_special_chars() {
        assert!(validate_service_id("evil@host").is_err());
        assert!(validate_service_id("host:8080").is_err());
        assert!(validate_service_id("host/path").is_err());
    }

    #[test]
    fn test_sanitize_path_rejects_traversal() {
        assert!(sanitize_path("/../etc/passwd").is_err());
        assert!(sanitize_path("/foo/../../etc/passwd").is_err());
        assert!(sanitize_path("/foo\0bar").is_err());
    }

    #[test]
    fn test_sanitize_path_normalizes_dots() {
        assert_eq!(sanitize_path("/foo/./bar").unwrap(), "/foo/bar");
        assert_eq!(sanitize_path("/").unwrap(), "/");
    }

    #[test]
    fn test_is_hop_by_hop() {
        assert!(is_hop_by_hop("host"));
        assert!(is_hop_by_hop("connection"));
        assert!(is_hop_by_hop("Keep-Alive"));
        assert!(is_hop_by_hop("TRANSFER-ENCODING"));
        assert!(!is_hop_by_hop("content-type"));
        assert!(!is_hop_by_hop("x-forwarded-for"));
    }
}
