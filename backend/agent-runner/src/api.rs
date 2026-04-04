use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::config::Config;
use crate::logger;

// ─── Data Structures ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
}

impl ContentBlock {
    pub fn type_name(&self) -> &'static str {
        match self {
            ContentBlock::Text { .. } => "text",
            ContentBlock::ToolUse { .. } => "tool_use",
            ContentBlock::ToolResult { .. } => "tool_result",
            ContentBlock::Thinking { .. } => "thinking",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// ─── Request Builder ─────────────────────────────────────────────────────────

#[derive(Serialize, Debug)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: &'a Vec<Message>,
    tools: Vec<Value>,
    stream: bool,
}

// ─── Client ──────────────────────────────────────────────────────────────────

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(AnthropicClient {
            client,
            api_key: config.effective_api_key().to_string(),
            base_url: config.anthropic_base_url.trim_end_matches('/').to_string(),
        })
    }

    pub async fn send_message(
        &self,
        config: &Config,
        system: &str,
        messages: &Vec<Message>,
        tools: Vec<Value>,
    ) -> Result<(ApiResponse, u64)> {
        let url = format!("{}/v1/messages", self.base_url);

        let request_body = ApiRequest {
            model: &config.model,
            max_tokens: config.max_tokens,
            system,
            messages,
            tools,
            stream: false,
        };

        // Log the outgoing request (without API key)
        let body_value = serde_json::to_value(&request_body)
            .context("Failed to serialize request")?;

        debug!(
            request = %serde_json::to_string_pretty(&body_value).unwrap_or_default(),
            "Sending API request"
        );

        if config.verbose {
            logger::log_info(&format!(
                "API request: model={}, messages={}, tools={}",
                config.model,
                messages.len(),
                request_body.tools.len()
            ));
        }

        let max_retries = 3u32;
        let mut attempt = 0u32;

        loop {
            let start = Instant::now();
            let result = self
                .client
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body_value)
                .send()
                .await;

            match result {
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    if attempt < max_retries {
                        let backoff = backoff_secs(attempt);
                        warn!(
                            attempt = attempt,
                            error = %e,
                            backoff_s = backoff,
                            "Request failed, retrying"
                        );
                        logger::log_error(
                            "API request",
                            &format!("Network error (attempt {}/{}), retry in {}s: {}", attempt + 1, max_retries + 1, backoff, e),
                        );
                        sleep(Duration::from_secs(backoff)).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(anyhow!("Request failed after {} attempts: {} ({}ms)", max_retries + 1, e, elapsed));
                }
                Ok(response) => {
                    let status = response.status();
                    let elapsed = start.elapsed().as_millis() as u64;

                    // Retry on transient server errors
                    if status == 429 || status == 500 || status == 502 || status == 503 {
                        if attempt < max_retries {
                            let backoff = backoff_secs(attempt);
                            let body_text = response.text().await.unwrap_or_default();
                            warn!(
                                status = %status,
                                attempt = attempt,
                                backoff_s = backoff,
                                body = %body_text,
                                "Retryable status, backing off"
                            );
                            logger::log_error(
                                "API status",
                                &format!("HTTP {} (attempt {}/{}), retry in {}s: {}", status, attempt + 1, max_retries + 1, backoff, body_text),
                            );
                            sleep(Duration::from_secs(backoff)).await;
                            attempt += 1;
                            continue;
                        }
                    }

                    if !status.is_success() {
                        let body_text = response.text().await.unwrap_or_default();
                        return Err(anyhow!(
                            "API error: HTTP {} — {}",
                            status,
                            body_text
                        ));
                    }

                    // Parse response
                    let body_text = response
                        .text()
                        .await
                        .context("Failed to read response body")?;

                    debug!(
                        response = %body_text,
                        "Received API response"
                    );

                    let api_response: ApiResponse = serde_json::from_str(&body_text)
                        .with_context(|| format!("Failed to parse API response: {}", &body_text[..body_text.len().min(500)]))?;

                    return Ok((api_response, elapsed));
                }
            }
        }
    }
}

fn backoff_secs(attempt: u32) -> u64 {
    // 1s, 2s, 4s, 8s (capped)
    let secs = 1u64 << attempt.min(3);
    secs
}
