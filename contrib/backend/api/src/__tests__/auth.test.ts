/**
 * Tests for src/utils/auth.ts
 *
 * The auth module reads config.authTokenSecret at call time (not at import
 * time) so we can mock the config module once and adjust the secret value per
 * test if needed.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import jwt from "jsonwebtoken";

// ─── Mock config ──────────────────────────────────────────────────────────────

const mockConfig = {
  authTokenSecret: "test-secret-key-for-unit-tests",
};

vi.mock("../config.js", () => ({
  get config() {
    return mockConfig;
  },
}));

// ─── Import SUT after mock ────────────────────────────────────────────────────

import {
  createAdminToken,
  createSandboxToken,
  getAuthTokenPayload,
  authTokenExpiresInSeconds,
  TOKEN_TTL_SECONDS,
} from "../utils/auth.js";

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Build a minimal Express-like request object with a Bearer token. */
function requestWithBearer(token: string) {
  return {
    header: (name: string) =>
      name.toLowerCase() === "authorization" ? `Bearer ${token}` : undefined,
    url: "/",
    headers: { authorization: `Bearer ${token}` },
  };
}

/** Build a request with the token in the query string. */
function requestWithQuery(token: string) {
  return {
    header: (_name: string) => undefined,
    url: `/path?token=${token}`,
    headers: {},
  };
}

/** Build a request with the token in a cookie. */
function requestWithCookie(token: string) {
  return {
    header: (_name: string) => undefined,
    url: "/",
    headers: { cookie: `sandbox_token=${token}` },
  };
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe("authTokenExpiresInSeconds / TOKEN_TTL_SECONDS", () => {
  it("is 86400 (24 hours)", () => {
    expect(TOKEN_TTL_SECONDS).toBe(86400);
    expect(authTokenExpiresInSeconds).toBe(86400);
  });

  it("is a reasonable value (between 1 hour and 7 days)", () => {
    const ONE_HOUR = 3600;
    const SEVEN_DAYS = 7 * 24 * 3600;
    expect(authTokenExpiresInSeconds).toBeGreaterThanOrEqual(ONE_HOUR);
    expect(authTokenExpiresInSeconds).toBeLessThanOrEqual(SEVEN_DAYS);
  });
});

describe("createAdminToken", () => {
  it("returns a valid JWT string", () => {
    const token = createAdminToken();
    expect(typeof token).toBe("string");
    expect(token.split(".")).toHaveLength(3);
  });

  it("payload has role=admin and sub=admin", () => {
    const token = createAdminToken();
    const payload = jwt.verify(token, mockConfig.authTokenSecret) as Record<string, unknown>;
    expect(payload.role).toBe("admin");
    expect(payload.sub).toBe("admin");
  });

  it("token includes an expiry (exp) claim", () => {
    const token = createAdminToken();
    const payload = jwt.decode(token) as Record<string, unknown>;
    expect(payload.exp).toBeDefined();
    expect(typeof payload.exp).toBe("number");
  });

  it("token expires approximately 24 hours from now", () => {
    const before = Math.floor(Date.now() / 1000);
    const token = createAdminToken();
    const payload = jwt.decode(token) as Record<string, number>;
    const after = Math.floor(Date.now() / 1000);

    const expectedExp = before + TOKEN_TTL_SECONDS;
    // Allow ±5 seconds of clock skew
    expect(payload.exp).toBeGreaterThanOrEqual(expectedExp - 5);
    expect(payload.exp).toBeLessThanOrEqual(after + TOKEN_TTL_SECONDS + 5);
  });
});

describe("createSandboxToken", () => {
  it("returns a valid JWT string", () => {
    const token = createSandboxToken("my-sandbox");
    expect(typeof token).toBe("string");
    expect(token.split(".")).toHaveLength(3);
  });

  it("includes sessionName in the payload", () => {
    const token = createSandboxToken("my-sandbox");
    const payload = jwt.verify(token, mockConfig.authTokenSecret) as Record<string, unknown>;
    expect(payload.sessionName).toBe("my-sandbox");
  });

  it("sets role=sandbox", () => {
    const token = createSandboxToken("my-sandbox");
    const payload = jwt.verify(token, mockConfig.authTokenSecret) as Record<string, unknown>;
    expect(payload.role).toBe("sandbox");
  });

  it("uses the sessionName as sub", () => {
    const token = createSandboxToken("session-xyz");
    const payload = jwt.verify(token, mockConfig.authTokenSecret) as Record<string, unknown>;
    expect(payload.sub).toBe("session-xyz");
  });

  it("token includes an expiry claim", () => {
    const token = createSandboxToken("s");
    const payload = jwt.decode(token) as Record<string, unknown>;
    expect(payload.exp).toBeDefined();
  });
});

describe("getAuthTokenPayload", () => {
  it("decodes a valid admin token from Bearer header", () => {
    const token = createAdminToken();
    const req = requestWithBearer(token);
    const payload = getAuthTokenPayload(req);

    expect(payload.role).toBe("admin");
    expect(payload.sub).toBe("admin");
  });

  it("decodes a valid sandbox token from Bearer header", () => {
    const token = createSandboxToken("sandbox-session");
    const req = requestWithBearer(token);
    const payload = getAuthTokenPayload(req);

    expect(payload.role).toBe("sandbox");
    expect(payload.sessionName).toBe("sandbox-session");
  });

  it("decodes a token from the query string ?token=", () => {
    const token = createAdminToken();
    const req = requestWithQuery(token);
    const payload = getAuthTokenPayload(req);

    expect(payload.role).toBe("admin");
  });

  it("decodes a token from the sandbox_token cookie", () => {
    const token = createSandboxToken("cookie-session");
    const req = requestWithCookie(token);
    const payload = getAuthTokenPayload(req);

    expect(payload.sessionName).toBe("cookie-session");
  });

  it("throws HttpError(401) when no token is present", () => {
    const req = {
      header: (_: string) => undefined,
      url: "/",
      headers: {},
    };
    expect(() => getAuthTokenPayload(req)).toThrow(
      expect.objectContaining({ status: 401 }),
    );
  });

  it("throws HttpError(401) for an invalid/tampered token", () => {
    const req = requestWithBearer("this.is.not.a.valid.jwt");
    expect(() => getAuthTokenPayload(req)).toThrow(
      expect.objectContaining({ status: 401 }),
    );
  });

  it("throws HttpError(401) for a token signed with a different secret", () => {
    const wrongToken = jwt.sign({ sub: "admin", role: "admin" }, "wrong-secret");
    const req = requestWithBearer(wrongToken);
    expect(() => getAuthTokenPayload(req)).toThrow(
      expect.objectContaining({ status: 401 }),
    );
  });

  it("throws HttpError(401) for an expired token", async () => {
    // Create a token that expired 1 second ago
    const expiredToken = jwt.sign(
      { sub: "admin", role: "admin" },
      mockConfig.authTokenSecret,
      { expiresIn: -1 },
    );
    const req = requestWithBearer(expiredToken);
    expect(() => getAuthTokenPayload(req)).toThrow(
      expect.objectContaining({ status: 401 }),
    );
  });

  it("returns payload with iat and exp fields", () => {
    const token = createAdminToken();
    const req = requestWithBearer(token);
    const payload = getAuthTokenPayload(req);

    expect(typeof payload.iat).toBe("number");
    expect(typeof payload.exp).toBe("number");
  });
});
