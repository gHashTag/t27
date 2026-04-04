/**
 * Tests for src/railway/client.ts
 *
 * We mock global fetch with vi.stubGlobal so that no real HTTP requests are
 * made.  The config module is mocked to provide stable token pool and URL
 * values without needing real env vars.
 */

import { describe, it, expect, afterEach, vi } from "vitest";

// ─── Hoisted constants ────────────────────────────────────────────────────────
// vi.mock factories are hoisted before all top-level code, so any value they
// reference must also be hoisted.

const { TOKEN_POOL } = vi.hoisted(() => ({
  TOKEN_POOL: ["token-account-0", "token-account-1"],
}));

// ─── Mock config ──────────────────────────────────────────────────────────────

vi.mock("../config.js", () => ({
  config: {
    railwayGraphqlUrl: "https://backboard.railway.app/graphql/v2",
    railwayApiTokenPool: TOKEN_POOL,
  },
  getRailwayToken: (idx?: number) => TOKEN_POOL[(idx ?? 0) % TOKEN_POOL.length],
  railwayAccountCount: () => TOKEN_POOL.length,
}));

// ─── Import SUT after mocks ───────────────────────────────────────────────────

import { railwayRequest } from "../railway/client.js";
import { HttpError } from "../utils/errors.js";

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Build a minimal fetch mock that returns the given status and JSON body. */
function makeFetchMock(status: number, body: unknown) {
  return vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
  });
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe("railwayRequest", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("returns data on a successful response", async () => {
    const mockData = { serviceCreate: { id: "svc-1", name: "my-service" } };
    vi.stubGlobal("fetch", makeFetchMock(200, { data: mockData }));

    const result = await railwayRequest("query { ... }", {}, 0);

    expect(result).toEqual(mockData);
  });

  it("throws HttpError(502) when the HTTP response is not OK (500)", async () => {
    vi.stubGlobal("fetch", makeFetchMock(500, {}));

    await expect(railwayRequest("query { ... }", {}, 0)).rejects.toMatchObject({
      status: 502,
      message: expect.stringContaining("500"),
    });
  });

  it("throws HttpError(502) on a 401 response", async () => {
    vi.stubGlobal("fetch", makeFetchMock(401, {}));

    await expect(railwayRequest("query { ... }", {}, 0)).rejects.toBeInstanceOf(HttpError);
  });

  it("throws HttpError(502) when the response contains GraphQL-level errors", async () => {
    vi.stubGlobal(
      "fetch",
      makeFetchMock(200, {
        errors: [{ message: "Not authorized" }, { message: "Rate limited" }],
      }),
    );

    await expect(railwayRequest("query { ... }", {}, 0)).rejects.toMatchObject({
      status: 502,
      message: expect.stringContaining("Not authorized"),
    });
  });

  it("includes all error messages joined with semicolons", async () => {
    vi.stubGlobal(
      "fetch",
      makeFetchMock(200, {
        errors: [{ message: "error one" }, { message: "error two" }],
      }),
    );

    const err = await railwayRequest("query { ... }", {}, 0).catch((e) => e);
    expect(err.message).toContain("error one");
    expect(err.message).toContain("error two");
  });

  it("throws HttpError(502) when data field is null", async () => {
    vi.stubGlobal("fetch", makeFetchMock(200, { data: null }));

    await expect(railwayRequest("query { ... }", {}, 0)).rejects.toMatchObject({
      status: 502,
      message: expect.stringContaining("missing response data"),
    });
  });

  it("throws HttpError(502) when response body has neither data nor errors", async () => {
    vi.stubGlobal("fetch", makeFetchMock(200, {}));

    await expect(railwayRequest("query { ... }", {}, 0)).rejects.toMatchObject({
      status: 502,
    });
  });

  it("uses the correct token for accountIndex=1", async () => {
    const fetchMock = makeFetchMock(200, { data: { result: true } });
    vi.stubGlobal("fetch", fetchMock);

    await railwayRequest("query { ... }", {}, 1);

    const [, options] = fetchMock.mock.calls[0] as [string, RequestInit];
    const auth = (options.headers as Record<string, string>)["Authorization"];
    expect(auth).toBe(`Bearer ${TOKEN_POOL[1]}`);
  });

  it("uses the correct token for accountIndex=0", async () => {
    const fetchMock = makeFetchMock(200, { data: { ok: true } });
    vi.stubGlobal("fetch", fetchMock);

    await railwayRequest("query { ... }", {}, 0);

    const [, options] = fetchMock.mock.calls[0] as [string, RequestInit];
    const auth = (options.headers as Record<string, string>)["Authorization"];
    expect(auth).toBe(`Bearer ${TOKEN_POOL[0]}`);
  });

  it("sends requests to the configured Railway GraphQL URL", async () => {
    const fetchMock = makeFetchMock(200, { data: { ok: true } });
    vi.stubGlobal("fetch", fetchMock);

    await railwayRequest("query { ... }", { foo: "bar" }, 0);

    const [url] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe("https://backboard.railway.app/graphql/v2");
  });

  it("includes query and variables in the request body", async () => {
    const fetchMock = makeFetchMock(200, { data: { ok: true } });
    vi.stubGlobal("fetch", fetchMock);

    const query = "mutation { serviceCreate { id } }";
    const variables = { input: { projectId: "p1" } };
    await railwayRequest(query, variables, 0);

    const [, options] = fetchMock.mock.calls[0] as [string, RequestInit];
    const body = JSON.parse(options.body as string);
    expect(body.query).toBe(query);
    expect(body.variables).toEqual(variables);
  });

  it("round-robin advances the counter on each call without explicit accountIndex", async () => {
    const capturedTokens: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn().mockImplementation((_url: string, opts: RequestInit) => {
        const auth = (opts.headers as Record<string, string>)["Authorization"];
        capturedTokens.push(auth);
        return Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve({ data: { ok: true } }),
        });
      }),
    );

    // Make two requests without specifying accountIndex
    await railwayRequest("query { ... }", {});
    await railwayRequest("query { ... }", {});

    // Both tokens should belong to our pool
    const poolBearer = TOKEN_POOL.map((t) => `Bearer ${t}`);
    expect(poolBearer).toContain(capturedTokens[0]);
    expect(poolBearer).toContain(capturedTokens[1]);
    // Round-robin means consecutive calls use different tokens
    expect(capturedTokens[0]).not.toBe(capturedTokens[1]);
  });
});
