// t27 Language Server
// Language Server Protocol implementation for .t27 specification files

use anyhow::Result;
use tower_lsp::{LspService, Server};

mod backend;
mod services;
mod types;
mod config;

use backend::Backend;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("tower_lsp=debug".parse()?),
        )
        .init();

    tracing::info!("t27 Language Server starting...");

    // Create LSP service
    let (service, socket) = LspService::new(|client| Backend::new(client));

    // Run server on stdio
    tracing::info!("Listening on stdio...");
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;

    Ok(())
}
