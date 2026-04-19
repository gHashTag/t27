//! Trinity Agent Bridge — CLI entry point.
//!
//! ```bash
//! trios-bridge                          # Start server on port 7474
//! trios-bridge --port 8080              # Custom port
//! trios-bridge --repo gHashTag/trios    # GitHub repo for issue parsing
//! ```

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

use trios_bridge::BridgeServer;

/// Trinity Agent Bridge — WebSocket server for multi-agent orchestration.
#[derive(Parser, Debug)]
#[command(name = "trios-bridge")]
#[command(version)]
struct Args {
    /// Port to listen on (default: 7474 = T-R-I-N)
    #[arg(short, long, default_value_t = 7474)]
    port: u16,

    /// GitHub repository (owner/repo) for issue parsing
    #[arg(short, long, default_value = "gHashTag/trios")]
    repo: String,

    /// GitHub personal access token (optional, for private repos)
    #[arg(short, long)]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("trios_bridge=info"))
        )
        .init();

    let args = Args::parse();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], args.port));

    tracing::info!("🚀 Starting Trinity Agent Bridge");
    tracing::info!("   Repo: {}", args.repo);
    tracing::info!("   Port: {}", args.port);

    let server = Arc::new(BridgeServer::new(&args.repo, args.token));
    server.serve(addr).await?;

    Ok(())
}
