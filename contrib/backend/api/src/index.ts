import "dotenv/config";

import { createServer } from "node:http";

import app from "./app.js";
import { config } from "./config.js";
import { pool } from "./db/client.js";
import { pollSandboxHealth } from "./services/health.js";
import { startOrphanCleanup } from "./services/orphanCleanup.js";
import {
  handleProxyRequest,
  handleProxyUpgrade,
  isDirectHost,
  isProxyHost,
} from "./proxy/sandboxProxy.js";
import { logger } from "./utils/logger.js";
import { onShutdown, setupGracefulShutdown } from "./utils/shutdown.js";

// ─────────────────────────────────────────────────────────────
// HTTP server – routes by hostname
// ─────────────────────────────────────────────────────────────

const server = createServer((req, res) => {
  const host = req.headers.host;

  if (isProxyHost(host)) {
    handleProxyRequest(req, res);
    return;
  }

  if (isDirectHost(host)) {
    app(req, res);
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "Not found" }));
});

server.on("upgrade", (req, socket, head) => {
  const host = req.headers.host;

  if (isProxyHost(host)) {
    handleProxyUpgrade(req, socket, head);
    return;
  }

  socket.destroy();
});

// ─────────────────────────────────────────────────────────────
// Graceful shutdown wiring
// ─────────────────────────────────────────────────────────────

setupGracefulShutdown(server);

onShutdown(async () => {
  logger.info("Draining database pool");
  await pool.end();
});

// ─────────────────────────────────────────────────────────────
// Health polling
// ─────────────────────────────────────────────────────────────

const startHealthPolling = (): NodeJS.Timeout => {
  let pollInFlight = false;

  const timer = setInterval(() => {
    if (pollInFlight) return;

    pollInFlight = true;
    pollSandboxHealth()
      .catch((err) => logger.error("Sandbox health poll failed", { error: String(err) }))
      .finally(() => {
        pollInFlight = false;
      });
  }, 5_000);

  return timer;
};

// ─────────────────────────────────────────────────────────────
// Start
// ─────────────────────────────────────────────────────────────

const startServer = async () => {
  const healthTimer = startHealthPolling();
  const orphanTimer = startOrphanCleanup();

  onShutdown(() => {
    clearInterval(healthTimer);
    clearInterval(orphanTimer);
    logger.info("Cleared health poll and orphan cleanup intervals");
  });

  setupGracefulShutdown(server);

  server.listen(config.port, () => {
    logger.info("API server started", {
      port: config.port,
      directHost: config.apiDirectHost,
      proxyHost: config.apiProxyHost,
      railwayAccounts: config.railwayApiTokenPool.length,
    });
  });
};

startServer().catch((err) => {
  logger.error("Failed to start API server", { error: String(err) });
  process.exit(1);
});
