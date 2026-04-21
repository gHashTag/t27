# Trios Chrome Extension — Rust + Wasm

## Trinity Stack Law Compliant

- ✅ All code is Rust (no JavaScript outside `dist/`)
- ✅ Uses `wasm-bindgen` for Chrome Extension API
- ✅ WebSocket via `gloo-net` to `trios-server` port 9005
- ✅ TRIOS brand colors (#161616, #F5D3F2, #5D3FF2)

## Build Instructions

```bash
# Install wasm-pack (one-time)
cargo install wasm-pack

# Build WASM
wasm-pack build --release --target web

# Output
pkg/trios_ext_bg.wasm
pkg/trios_ext.js
pkg/trios_ext_bg.wasm.d.ts
```

## Chrome Extension Structure

```
extension/
├── manifest.json       # MV3 manifest (minimal service_worker)
├── sidepanel.html      # Loads WASM module
├── sw.js             # Minimal service worker
└── icons/
    ├── icon-16.png
    ├── icon-32.png
    ├── icon-48.png
    └── icon-128.png   # Φ symbol with Trinity colors
```

## Deployment to Chrome

1. Copy WASM artifacts:
   ```bash
   cp pkg/trios_ext.js extension/
   cp pkg/trios_ext_bg.wasm extension/
   cp pkg/trios_ext_bg.wasm.d.ts extension/
   ```

2. Load in Chrome:
   - Open `chrome://extensions`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select `extension/` directory

## MCP Protocol

Connects to `ws://localhost:9005/mcp` with JSON-RPC 2.0 format:

```json
{
  "jsonrpc": "2.0",
  "id": <number>,
  "method": "agents/list" | "agents/chat" | "tools/list" | "tools/call",
  "params": { ... }
}
```

## Features

- **Chat**: Send messages to agents via MCP
- **Agents**: View connected agents with status
- **MCP Tools**: Browse and call MCP tools
