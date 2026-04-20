import type { Server } from "node:http";

import { logger } from "./logger.js";

const SHUTDOWN_TIMEOUT_MS = 30_000;

let shuttingDown = false;
let activeRequests = 0;

const cleanupCallbacks: Array<() => Promise<void> | void> = [];

export const isShuttingDown = (): boolean => shuttingDown;

export const trackRequestStart = (): void => {
  activeRequests++;
};

export const trackRequestEnd = (): void => {
  activeRequests--;
};

export const onShutdown = (cb: () => Promise<void> | void): void => {
  cleanupCallbacks.push(cb);
};

export const setupGracefulShutdown = (server: Server): void => {
  const shutdown = async (signal: string) => {
    if (shuttingDown) return;
    shuttingDown = true;

    logger.info("Shutdown initiated", {
      signal,
      activeRequests,
    });

    for (const cb of cleanupCallbacks) {
      try {
        await cb();
      } catch (err) {
        logger.error("Cleanup callback failed", {
          error: String(err),
        });
      }
    }

    server.close(() => {
      logger.info("All connections closed, exiting");
      process.exit(0);
    });

    setTimeout(() => {
      logger.error("Shutdown timeout, forcing exit", {
        timeoutMs: SHUTDOWN_TIMEOUT_MS,
        activeRequests,
      });
      process.exit(1);
    }, SHUTDOWN_TIMEOUT_MS);
  };

  process.on("SIGTERM", () => shutdown("SIGTERM"));
  process.on("SIGINT", () => shutdown("SIGINT"));
};
