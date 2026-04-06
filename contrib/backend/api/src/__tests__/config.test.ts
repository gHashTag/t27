/**
 * Tests for src/config.ts
 *
 * The config module reads process.env at import time, so each test that needs
 * different env values must dynamically re-import the module after setting up
 * the environment.  We use vi.resetModules() + dynamic import() to achieve
 * a clean slate for each scenario.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";

// Save original env so we can restore it after each test
const ORIGINAL_ENV = { ...process.env };

const resetEnv = () => {
  // Wipe all test keys
  for (const key of Object.keys(process.env)) {
    if (
      key.startsWith("RAILWAY_") ||
      key === "LOCAL_MODE" ||
      key === "SANDBOX_LOCAL_BASE_URL" ||
      key === "SANDBOX_LOCAL_MAP" ||
      key === "NODE_ENV"
    ) {
      delete process.env[key];
    }
  }
  // Restore originals
  Object.assign(process.env, ORIGINAL_ENV);
};

/**
 * Helper: set env vars, reset the module registry, and dynamically import
 * the config module so it re-executes with the new environment.
 */
async function loadConfig(env: Record<string, string | undefined>) {
  vi.resetModules();
  resetEnv();
  for (const [k, v] of Object.entries(env)) {
    if (v === undefined) {
      delete process.env[k];
    } else {
      process.env[k] = v;
    }
  }
  // Dynamic import gives us a fresh module evaluation
  return import("../config.js");
}

// Minimal required env for config to load without throwing
const BASE_ENV: Record<string, string> = {
  RAILWAY_API_TOKEN: "base-token",
  RAILWAY_PROJECT_ID: "proj-123",
  RAILWAY_ENVIRONMENT_ID: "env-456",
};

describe("buildRailwayTokenPool", () => {
  afterEach(() => {
    vi.resetModules();
    resetEnv();
  });

  it("returns numbered tokens when RAILWAY_API_TOKEN_0/1/2 are set", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN_0: "token-0",
      RAILWAY_API_TOKEN_1: "token-1",
      RAILWAY_API_TOKEN_2: "token-2",
    });
    expect(config.railwayApiTokenPool).toEqual(["token-0", "token-1", "token-2"]);
  });

  it("stops at the first gap in the numbered sequence", async () => {
    // _0 and _2 set but not _1 – should only collect _0
    const { config } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN_0: "token-0",
      RAILWAY_API_TOKEN_2: "token-2", // gap at 1 stops iteration
    });
    expect(config.railwayApiTokenPool).toEqual(["token-0"]);
  });

  it("falls back to RAILWAY_API_TOKEN when no numbered tokens", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN: "fallback-token",
    });
    expect(config.railwayApiTokenPool).toEqual(["fallback-token"]);
  });

  it("throws when neither numbered tokens nor RAILWAY_API_TOKEN are set", async () => {
    await expect(
      loadConfig({
        RAILWAY_PROJECT_ID: "proj",
        RAILWAY_ENVIRONMENT_ID: "env",
        // Explicitly unset the token so even the CLI-provided value is absent
        RAILWAY_API_TOKEN: undefined,
      }),
    ).rejects.toThrow("Missing required environment variable: RAILWAY_API_TOKEN");
  });
});

describe("getRailwayToken", () => {
  afterEach(() => {
    vi.resetModules();
    resetEnv();
  });

  it("returns the correct token for a given accountIndex", async () => {
    const { getRailwayToken } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN_0: "token-a",
      RAILWAY_API_TOKEN_1: "token-b",
    });
    expect(getRailwayToken(0)).toBe("token-a");
    expect(getRailwayToken(1)).toBe("token-b");
  });

  it("does modular indexing – wraps around pool length", async () => {
    const { getRailwayToken } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN_0: "token-a",
      RAILWAY_API_TOKEN_1: "token-b",
    });
    // index 2 should wrap to 0
    expect(getRailwayToken(2)).toBe("token-a");
    // index 3 should wrap to 1
    expect(getRailwayToken(3)).toBe("token-b");
  });

  it("defaults to index 0 when accountIndex is undefined", async () => {
    const { getRailwayToken } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN_0: "only-token",
    });
    expect(getRailwayToken(undefined)).toBe("only-token");
  });
});

describe("railwayAccountCount", () => {
  afterEach(() => {
    vi.resetModules();
    resetEnv();
  });

  it("returns 1 when only RAILWAY_API_TOKEN is set", async () => {
    const { railwayAccountCount } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN: "single",
    });
    expect(railwayAccountCount()).toBe(1);
  });

  it("returns 3 when three numbered tokens are set", async () => {
    const { railwayAccountCount } = await loadConfig({
      ...BASE_ENV,
      RAILWAY_API_TOKEN_0: "t0",
      RAILWAY_API_TOKEN_1: "t1",
      RAILWAY_API_TOKEN_2: "t2",
    });
    expect(railwayAccountCount()).toBe(3);
  });
});

describe("localMode", () => {
  afterEach(() => {
    vi.resetModules();
    resetEnv();
  });

  it("is true when LOCAL_MODE=true (regardless of other vars)", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      LOCAL_MODE: "true",
    });
    expect(config.localMode).toBe(true);
  });

  it("is false when LOCAL_MODE is not set and no local overrides", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      NODE_ENV: "development",
    });
    expect(config.localMode).toBe(false);
  });

  it("is false when LOCAL_MODE=false", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      LOCAL_MODE: "false",
    });
    expect(config.localMode).toBe(false);
  });

  it("is false when SANDBOX_LOCAL_BASE_URL is set without LOCAL_MODE=true (explicit activation required)", async () => {
    // After review fix: localMode requires explicit LOCAL_MODE=true.
    // Setting SANDBOX_LOCAL_BASE_URL alone does NOT activate it.
    const { config } = await loadConfig({
      ...BASE_ENV,
      SANDBOX_LOCAL_BASE_URL: "http://localhost:8080",
      NODE_ENV: "development",
    });
    expect(config.localMode).toBe(false);
  });

  it("is true when SANDBOX_LOCAL_BASE_URL is set AND LOCAL_MODE=true", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      SANDBOX_LOCAL_BASE_URL: "http://localhost:8080",
      LOCAL_MODE: "true",
      NODE_ENV: "production",
    });
    expect(config.localMode).toBe(true);
  });

  it("is false when SANDBOX_LOCAL_MAP is set without LOCAL_MODE=true", async () => {
    const { config } = await loadConfig({
      ...BASE_ENV,
      SANDBOX_LOCAL_MAP: "my-session=http://localhost:9000",
      NODE_ENV: "test",
    });
    expect(config.localMode).toBe(false);
  });
});
