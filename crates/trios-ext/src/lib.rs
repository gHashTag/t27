//! Trios Chrome Extension — Rust + WASM
//! Trinity Stack Law Compliant: Zero handwritten JS

pub mod bg;
pub mod dom;
pub mod mcp;

// Note: Dioxus launch requires different setup for Chrome Extensions
// Using direct DOM manipulation for now (L6 compliant)
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    // Initialize the extension
    console_error_panic_hook::set_once();
    log::info!("Trios extension initialized");

    // For MVP, use simple DOM setup
    if let Err(e) = crate::dom::build_ui() {
        log::error!("Failed to build UI: {:?}", e);
    }

    if let Err(e) = crate::mcp::mcp_connect() {
        log::warn!("MCP connection failed (trios-server may not be running): {:?}", e);
    }

    // Load initial data
    let _ = crate::mcp::mcp_list_agents();
    let _ = crate::mcp::mcp_list_tools();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_url_uses_port_9005() {
        assert!(mcp::MCP_WS_URL.contains("9005"));
    }

    #[test]
    fn mcp_client_starts_disconnected() {
        let client = mcp::McpClient::new();
        assert!(!client.is_connected());
    }
}
