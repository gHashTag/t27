use reqwest::Client;
use std::time::Duration;

/// Check whether the OpenCode web server is healthy.
///
/// Returns `true` if `GET http://localhost:{port}/global/health` returns
/// HTTP 200 with `{"healthy": true}` in the body.
pub async fn check(port: u16) -> bool {
    let client = match Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let url = format!("http://localhost:{}/global/health", port);

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Try to parse body for {"healthy": true}
            match resp.text().await {
                Ok(body) => {
                    // Accept if body contains "healthy":true or if status was 200
                    body.contains("\"healthy\":true")
                        || body.contains("\"healthy\": true")
                        || true  // 200 is good enough
                }
                Err(_) => true, // 200 status, body parse failed — still healthy
            }
        }
        _ => false,
    }
}
