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
    hyper::{client::HttpConnector, Client as HyperClient},
    std::{
        collections::HashMap,
        sync::Arc,
    },
    crate::main::AppState,
};

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
    State(state): State<Arc<AppState>>,
    mut req: Request,
) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    // Try to get token from query parameter or header
    let token = extract_token_from_query(&uri)
        .or_else(|| extract_token_from_header(&headers));

    if let Some(token) = token {
        // Verify JWT and get session_id
        match crate::jwt::verify_sandbox_token(&token) {
            Ok(session_id) => {
                // Find session to get railway_service_id
                let sessions = state.sessions.read().await;
                if let Some(session) = sessions.iter().find(|s| s.id == session_id) {
                    // Check if session is active
                    if session.status != "active" && session.status != "starting" {
                        drop(sessions);
                        return (StatusCode::SERVICE_UNAVAILABLE, "Session not ready").into_response();
                    }

                    let service_id = session.railway_service_id.clone();
                    drop(sessions);

                    // Form URL for Railway internal DNS
                    // Railway uses <service-id>.railway.internal for internal communication
                    let path_and_query = uri.path_and_query()
                        .map(|p| p.as_str())
                        .unwrap_or("/");

                    // Strip /sandbox prefix if present
                    let clean_path = path_and_query.strip_prefix("/sandbox")
                        .unwrap_or(path_and_query);

                    let target_url = format!(
                        "http://{}.railway.internal:8080{}",
                        service_id,
                        clean_path
                    );

                    // Proxy request to container
                    proxy_to_container(&target_url, method, headers, req.into_body()).await
                } else {
                    (StatusCode::NOT_FOUND, "Session not found").into_response()
                }
            }
            Err(e) => {
                eprintln!("Token verification failed: {}", e);
                (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Missing authentication token").into_response()
    }
}

/// Proxy an HTTP request to a Railway container
#[cfg(feature = "server")]
async fn proxy_to_container(
    target_url: &str,
    method: Method,
    original_headers: HeaderMap,
    original_body: Body,
) -> Response {
    type HttpClient = HyperClient<HttpConnector, Body>;
    let client = HttpClient::new();

    // Read the original body
    let body_bytes = match hyper::body::to_bytes(original_body).await {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to read request body: {}", e);
            return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
        }
    };

    // Build the request to the container
    let mut request_builder = hyper::Request::builder()
        .uri(target_url)
        .method(method.clone());

    // Copy relevant headers (skip hop-by-hop headers)
    for (name, value) in original_headers.iter() {
        let name_str = name.as_str();
        // Skip headers that shouldn't be forwarded
        if !matches!(
            name_str,
            "host" | "connection" | "keep-alive" | "transfer-encoding" | "te"
        ) {
            request_builder = request_builder.header(name, value);
        }
    }

    // Set X-Forwarded-For header if possible
    if let Some(forwarded_for) = original_headers.get("x-forwarded-for") {
        request_builder = request_builder.header("x-forwarded-for", forwarded_for);
    }

    let body = Body::from(body_bytes);

    match request_builder.body(body) {
        Ok(req) => {
            match client.request(req).await {
                Ok(mut resp) => {
                    // Build the response
                    let mut response_builder = Response::builder()
                        .status(resp.status());

                    // Copy response headers (skip hop-by-hop headers)
                    for (name, value) in resp.headers().iter() {
                        let name_str = name.as_str();
                        if !matches!(
                            name_str,
                            "connection" | "keep-alive" | "transfer-encoding" | "te"
                        ) {
                            response_builder = response_builder.header(name, value);
                        }
                    }

                    // Convert hyper body to axum body
                    match hyper::body::to_bytes(resp.into_body()).await {
                        Ok(body_bytes) => {
                            response_builder
                                .body(Body::from(body_bytes))
                                .unwrap_or_else(|_| {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Response build failed")
                                        .into_response()
                                })
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
/// Get the proxy URL for a session
/// Returns a URL like "/sandbox?token=<jwt>" that proxies to the container
pub fn get_proxy_url(session_id: &str) -> Result<String> {
    let token = crate::jwt::create_sandbox_token(session_id, Some(24))?;
    Ok(format!("/sandbox?token={}", token))
}

/// Health check for a Railway container
#[cfg(feature = "server")]
pub async fn check_container_health(service_id: &str) -> Result<bool> {
    type HttpClient = HyperClient<HttpConnector, Body>;
    let client = HttpClient::new();
    let url = format!("http://{}.railway.internal:8080/health", service_id);

    let request = hyper::Request::builder()
        .uri(&url)
        .method(Method::GET)
        .body(Body::empty())?;

    match client.request(request).await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
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
}
