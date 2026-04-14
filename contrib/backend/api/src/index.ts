import "dotenv/config";

import { createServer } from "node:http";

import app from "./app.js";
import { config } from "./config.js";
import { pollSandboxHealth } from "./services/health.js";
import {
  handleProxyRequest,
  handleProxyUpgrade,
  isDirectHost,
  isProxyHost,
} from "./proxy/sandboxProxy.js";

// ─────────────────────────────────────────────────────────────
// HTTP server – routes by hostname
// ─────────────────────────────────────────────────────────────

const server = createServer((req, res) => {
  const host = req.headers.host;

  if (isProxyHost(host)) {
    // Requests to the proxy hostname are forwarded to the appropriate sandbox
    handleProxyRequest(req, res);
    return;
  }

  if (isDirectHost(host)) {
    // Requests to the direct hostname are handled by the Express app
    app(req, res);
    return;
  }

  // Unknown host – reject
  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "Not found" }));
});

// WebSocket upgrade routing
server.on("upgrade", (req, socket, head) => {
  const host = req.headers.host;

  if (isProxyHost(host)) {
    handleProxyUpgrade(req, socket, head);
    return;
  }

  socket.destroy();
});

// ─────────────────────────────────────────────────────────────
// Health polling
// ─────────────────────────────────────────────────────────────

const startHealthPolling = () => {
  let pollInFlight = false;

  setInterval(() => {
    if (pollInFlight) return;

    pollInFlight = true;
    pollSandboxHealth()
      .catch((err) => console.error("Sandbox health poll failed", err))
      .finally(() => {
        pollInFlight = false;
      });
  }, 5_000);
};

// ─────────────────────────────────────────────────────────────
// Start
// ─────────────────────────────────────────────────────────────

const startServer = async () => {
  startHealthPolling();

  server.listen(config.port, () => {
    console.log(`API listening on port ${config.port}`);
    console.log(`  Direct:  http://${config.apiDirectHost}:${config.port}`);
    console.log(`  Proxy:   http://${config.apiProxyHost}:${config.port}`);
    console.log(
      `  Railway accounts in pool: ${config.railwayApiTokenPool.length}`,
    );
  });
};

startServer().catch((err) => {
  console.error("Failed to start API server", err);
  process.exit(1);
});
