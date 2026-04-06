//! Trinity Core - Main Entry Point
//!
//! High-performance Session & Message management service for Trinity Orchestrator.

mod broadcaster;
mod handlers;
mod models;
mod store;

use anyhow::Result;
use std::env;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(false)
        .init();

    // Get port from environment
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .unwrap_or(8082);

    let addr = format!("0.0.0.0:{}", port);

    // Create application state
    let state = broadcaster::AppState::new();

    // Build router
    let app = handlers::create_router(state);

    // Start server
    tracing::info!("Starting Trinity Core on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
