/**
 * Tests for src/utils/sandboxTarget.ts
 *
 * We mock the config module to control localMode, sandboxLocalBaseUrl,
 * sandboxLocalMap, sandboxInternalDomain, and sandboxPort for each scenario.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";

// ─── Mutable config object ────────────────────────────────────────────────────

const mockConfig = {
  localMode: false,
  sandboxLocalBaseUrl: undefined as string | undefined,
  sandboxLocalMap: {} as Record<string, string>,
  sandboxInternalDomain: "railway.internal",
  sandboxPort: 8080,
};

vi.mock("../config.js", () => ({
  get config() {
    return mockConfig;
  },
}));

// ─── Import SUT after mock ────────────────────────────────────────────────────

import {
  resolveSandboxTarget,
  resolveSandboxHealthUrl,
} from "../utils/sandboxTarget.js";

// ─── Tests ────────────────────────────────────────────────────────────────────

beforeEach(() => {
  // Reset to production-like defaults
  mockConfig.localMode = false;
  mockConfig.sandboxLocalBaseUrl = undefined;
  mockConfig.sandboxLocalMap = {};
  mockConfig.sandboxInternalDomain = "railway.internal";
  mockConfig.sandboxPort = 8080;
});

describe("resolveSandboxTarget", () => {
  it("returns Railway internal URL in production mode", () => {
    mockConfig.localMode = false;

    const url = resolveSandboxTarget("my-session");

    expect(url).toBe("http://my-session.railway.internal:8080");
  });

  it("uses configured sandboxInternalDomain and sandboxPort", () => {
    mockConfig.localMode = false;
    mockConfig.sandboxInternalDomain = "custom.internal";
    mockConfig.sandboxPort = 9090;

    const url = resolveSandboxTarget("svc-name");

    expect(url).toBe("http://svc-name.custom.internal:9090");
  });

  it("returns local base URL when localMode is true and SANDBOX_LOCAL_BASE_URL is set", () => {
    mockConfig.localMode = true;
    mockConfig.sandboxLocalBaseUrl = "http://localhost:8080";

    const url = resolveSandboxTarget("any-session");

    expect(url).toBe("http://localhost:8080");
  });

  it("returns mapped URL when session name is in SANDBOX_LOCAL_MAP", () => {
    mockConfig.localMode = true;
    mockConfig.sandboxLocalBaseUrl = "http://localhost:8080";
    mockConfig.sandboxLocalMap = {
      "special-session": "http://localhost:9001",
    };

    const url = resolveSandboxTarget("special-session");

    // Map entry takes precedence over base URL
    expect(url).toBe("http://localhost:9001");
  });

  it("falls back to sandboxLocalBaseUrl when session is not in the map", () => {
    mockConfig.localMode = true;
    mockConfig.sandboxLocalBaseUrl = "http://localhost:8080";
    mockConfig.sandboxLocalMap = {
      "other-session": "http://localhost:9002",
    };

    const url = resolveSandboxTarget("unknown-session");

    expect(url).toBe("http://localhost:8080");
  });

  it("returns Railway internal URL when localMode is false even if localBaseUrl is set", () => {
    // localMode controls whether local overrides are used
    mockConfig.localMode = false;
    mockConfig.sandboxLocalBaseUrl = "http://localhost:8080";

    const url = resolveSandboxTarget("svc");

    expect(url).toBe("http://svc.railway.internal:8080");
  });

  it("returns Railway internal URL when localMode is true but no local URL is configured", () => {
    // If localMode but no override URLs, falls through to Railway URL
    mockConfig.localMode = true;
    mockConfig.sandboxLocalBaseUrl = undefined;
    mockConfig.sandboxLocalMap = {};

    const url = resolveSandboxTarget("svc");

    expect(url).toBe("http://svc.railway.internal:8080");
  });
});

describe("resolveSandboxHealthUrl", () => {
  it("appends /healthz to the production Railway URL", () => {
    mockConfig.localMode = false;

    const url = resolveSandboxHealthUrl("my-session");

    expect(url).toBe("http://my-session.railway.internal:8080/healthz");
  });

  it("appends /healthz to the local base URL", () => {
    mockConfig.localMode = true;
    mockConfig.sandboxLocalBaseUrl = "http://localhost:3000";

    const url = resolveSandboxHealthUrl("my-session");

    expect(url).toBe("http://localhost:3000/healthz");
  });

  it("appends /healthz to a session-specific mapped URL", () => {
    mockConfig.localMode = true;
    mockConfig.sandboxLocalMap = {
      "mapped-session": "http://localhost:7777",
    };

    const url = resolveSandboxHealthUrl("mapped-session");

    expect(url).toBe("http://localhost:7777/healthz");
  });

  it("always ends with /healthz", () => {
    mockConfig.localMode = false;
    const url = resolveSandboxHealthUrl("some-session");
    expect(url.endsWith("/healthz")).toBe(true);
  });
});
