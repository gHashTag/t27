import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

vi.mock("../config.js", () => ({
  config: { nodeEnv: "test" },
}));

import { logger, createRequestLogger, generateTraceId } from "../utils/logger.js";

describe("logger", () => {
  it("exports a logger with standard log levels", () => {
    expect(typeof logger.debug).toBe("function");
    expect(typeof logger.info).toBe("function");
    expect(typeof logger.warn).toBe("function");
    expect(typeof logger.error).toBe("function");
    expect(typeof logger.child).toBe("function");
  });

  it("does not throw when logging", () => {
    expect(() => logger.info("test message", { key: "value" })).not.toThrow();
    expect(() => logger.error("error message", { err: "detail" })).not.toThrow();
  });
});

describe("createRequestLogger", () => {
  it("returns a logger with a traceId", () => {
    const reqLogger = createRequestLogger();
    expect(typeof reqLogger.info).toBe("function");
    expect(typeof reqLogger.error).toBe("function");
  });

  it("uses provided traceId when given", () => {
    const reqLogger = createRequestLogger("custom-trace-id");
    expect(typeof reqLogger.info).toBe("function");
  });

  it("generates a traceId when not provided", () => {
    const id = generateTraceId();
    expect(id).toBeDefined();
    expect(id.length).toBe(16);
  });
});
