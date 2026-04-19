//! WebSocket server for Trinity Agent Bridge.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

use crate::router::AgentRouter;
use crate::protocol::BridgeMessage;

/// Bridge server state.
#[derive(Clone)]
pub struct BridgeServer {
    router: AgentRouter,
    tx: broadcast::Sender<String>,
    github_repo: String,
    github_token: Option<String>,
}

impl BridgeServer {
    /// Create a new bridge server.
    pub fn new(github_repo: &str, github_token: Option<String>) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            router: AgentRouter::new(),
            tx,
            github_repo: github_repo.to_string(),
            github_token,
        }
    }

    /// Get the agent router.
    pub fn router(&self) -> &AgentRouter {
        &self.router
    }

    /// Get the GitHub repo.
    pub fn github_repo(&self) -> &str {
        &self.github_repo
    }

    /// Get the GitHub token.
    pub fn github_token(&self) -> Option<&str> {
        self.github_token.as_deref()
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast_message(&self, msg: &BridgeMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        let _ = self.tx.send(json);
        Ok(())
    }

    /// Start the WebSocket server.
    pub async fn serve(self: Arc<Self>, addr: SocketAddr) -> Result<()> {
        tracing::info!("🔌 Trinity Agent Bridge starting on ws://{}", addr);

        let listener = TcpListener::bind(addr).await?;
        tracing::info!("✅ Bridge listening on port {}", addr.port());

        while let Ok((stream, peer)) = listener.accept().await {
            tracing::info!("📥 New connection from {}", peer);

            let server = self.clone();
            let mut rx = self.tx.subscribe();

            tokio::spawn(async move {
                let ws_stream = tokio_tungstenite::accept_async(stream).await;
                match ws_stream {
                    Ok(ws_stream) => {
                        let (mut write, mut read) = ws_stream.split();

                        // Forward broadcast messages to this client
                        let server_fwd = server.clone();
                        let write_task = tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                if write.send(Message::Text(msg)).await.is_err() {
                                    break;
                                }
                            }
                        });

                        // Handle incoming messages
                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    if let Err(e) = server_fwd.handle_message(&text).await {
                                        tracing::warn!("Error handling message: {}", e);
                                    }
                                }
                                Ok(Message::Close(_)) => break,
                                Err(e) => {
                                    tracing::warn!("WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }

                        write_task.abort();
                        tracing::info!("📤 Client disconnected");
                    }
                    Err(e) => {
                        tracing::warn!("WebSocket handshake failed: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Handle an incoming message from a client.
    async fn handle_message(&self, text: &str) -> Result<()> {
        let msg: BridgeMessage = serde_json::from_str(text)?;

        match &msg {
            BridgeMessage::SendCommand(cmd) => {
                tracing::info!("📨 SendCommand: {} -> {}", cmd.command, cmd.target);
                if cmd.target != "broadcast" && !self.router.is_registered(&cmd.target).await {
                    let err = BridgeMessage::error(
                        "agent_not_found".into(),
                        format!("Agent '{}' not found", cmd.target),
                    );
                    self.broadcast_message(&err)?;
                } else {
                    self.broadcast_message(&msg)?;
                }
            }

            BridgeMessage::ClaimIssue(cmd) => {
                tracing::info!("🎯 ClaimIssue: agent {} claims #{}", cmd.agent_id, cmd.issue_number);
                self.router.claim_issue(&cmd.agent_id, cmd.issue_number, cmd.branch.clone()).await;
                let agents = self.router.list().await;
                self.broadcast_message(&BridgeMessage::board_state(agents, vec![]))?;
            }

            BridgeMessage::UpdateStatus(cmd) => {
                tracing::info!("📊 UpdateStatus: agent {} -> {:?}", cmd.agent_id, cmd.status);
                self.router.update_status(&cmd.agent_id, cmd.status, cmd.message.clone()).await;
                let agents = self.router.list().await;
                self.broadcast_message(&BridgeMessage::board_state(agents, vec![]))?;
            }

            BridgeMessage::ListAgents(_) => {
                let agents = self.router.list().await;
                self.broadcast_message(&BridgeMessage::board_state(agents, vec![]))?;
            }

            _ => {
                tracing::debug!("Unhandled message type, forwarding");
                self.broadcast_message(&msg)?;
            }
        }

        Ok(())
    }
}
