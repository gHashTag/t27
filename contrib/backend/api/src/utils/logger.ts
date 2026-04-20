import { randomUUID } from "node:crypto";

import pino from "pino";

import { config } from "../config.js";

const isProduction = config.nodeEnv === "production";

const generateTraceId = (): string =>
  randomUUID().slice(0, 16);

const rootLogger = pino({
  level: process.env.LOG_LEVEL ?? (isProduction ? "info" : "debug"),
  ...(isProduction
    ? {}
    : {
        transport: {
          target: "pino/file",
          options: { destination: 1 },
        },
        formatters: {
          level(label) {
            return { level: label };
          },
        },
      }),
});

export interface Logger {
  debug: (msg: string, data?: Record<string, unknown>) => void;
  info: (msg: string, data?: Record<string, unknown>) => void;
  warn: (msg: string, data?: Record<string, unknown>) => void;
  error: (msg: string, data?: Record<string, unknown>) => void;
  child: (bindings: Record<string, unknown>) => Logger;
}

const wrap = (logger: pino.Logger): Logger => ({
  debug: (msg, data) => logger.debug(data ?? {}, msg),
  info: (msg, data) => logger.info(data ?? {}, msg),
  warn: (msg, data) => logger.warn(data ?? {}, msg),
  error: (msg, data) => logger.error(data ?? {}, msg),
  child: (bindings) => wrap(logger.child(bindings)),
});

export const logger: Logger = wrap(rootLogger);

export const createRequestLogger = (traceId?: string): Logger => {
  const id = traceId ?? generateTraceId();
  return wrap(rootLogger.child({ traceId: id }));
};

export { generateTraceId };
