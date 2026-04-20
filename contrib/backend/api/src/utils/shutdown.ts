import type { Server } from "node:http";

import { config } from "../config.js";

const SHUTDOWN_TIMEOUT_MS = 30_000;

let shuttingDown = false;
let activeRequests = 0;

export const isShuttingDown = (): boolean => shuttingDown;

export const trackRequestStart = (): void => {
  activeRequests++;
};

export const trackRequestEnd = (): void => {
  activeRequests--;
};

export const setupGracefulShutdown = (server: Server): void => {
  const shutdown = (signal: string) => {
    if (shuttingDown) return;
    shuttingDown = true;

    console.log(
      `[shutdown] ${signal} received, draining ${activeRequests} active requests...`,
    );

    server.close(() => {
      console.log("[shutdown] all connections closed, exiting");
      process.exit(0);
    });

    setTimeout(() => {
      console.log(
        `[shutdown] timeout after ${SHUTDOWN_TIMEOUT_MS}ms with ${activeRequests} active requests, forcing exit`,
      );
      process.exit(1);
    }, SHUTDOWN_TIMEOUT_MS);
  };

  process.on("SIGTERM", () => shutdown("SIGTERM"));
  process.on("SIGINT", () => shutdown("SIGINT"));
};
