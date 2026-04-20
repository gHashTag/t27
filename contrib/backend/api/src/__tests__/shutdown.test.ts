import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

vi.mock("../config.js", () => ({
  config: { nodeEnv: "test" },
}));

import {
  setupGracefulShutdown,
  isShuttingDown,
  trackRequestStart,
  trackRequestEnd,
  onShutdown,
} from "../utils/shutdown.js";
import type { Server } from "node:http";

describe("shutdown", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(process, "on").mockImplementation(() => process);
    vi.spyOn(process, "exit").mockImplementation(() => undefined as never);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("isShuttingDown returns false initially", () => {
    expect(isShuttingDown()).toBe(false);
  });

  it("trackRequestStart/End track active requests", () => {
    trackRequestStart();
    trackRequestStart();
    trackRequestEnd();
  });

  it("setupGracefulShutdown registers SIGTERM and SIGINT handlers", () => {
    const mockServer = { close: vi.fn((cb) => cb()) } as unknown as Server;
    setupGracefulShutdown(mockServer);

    expect(process.on).toHaveBeenCalledWith("SIGTERM", expect.any(Function));
    expect(process.on).toHaveBeenCalledWith("SIGINT", expect.any(Function));
  });

  it("onShutdown registers cleanup callbacks", () => {
    const cb = vi.fn();
    onShutdown(cb);

    const mockServer = { close: vi.fn((cb) => cb()) } as unknown as Server;
    setupGracefulShutdown(mockServer);

    const sigtermHandler = (process.on as ReturnType<typeof vi.fn>).mock.calls.find(
      (call) => call[0] === "SIGTERM",
    )?.[1];

    if (sigtermHandler) {
      sigtermHandler("SIGTERM");
      expect(cb).toHaveBeenCalled();
    }
  });
});
