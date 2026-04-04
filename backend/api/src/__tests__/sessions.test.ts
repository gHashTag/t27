/**
 * Tests for src/services/sessions.ts
 *
 * External dependencies are mocked:
 *   - ../railway/client.js  →  railwayRequest spy (via vi.hoisted to avoid TDZ)
 *   - ../db/client.js       →  in-memory store mirroring the drizzle API
 *   - ../config.js          →  stable config values
 *   - ../railway/mutations.js → plain string exports (no side-effects)
 *   - drizzle-orm           →  eq/desc/inArray shims (also hoisted)
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import type { Session } from "../db/schema.js";
import { HttpError } from "../utils/errors.js";

// ─── Hoisted helpers ─────────────────────────────────────────────────────────
// vi.mock factories are hoisted before all top-level code.  Any variable used
// inside a vi.mock factory must therefore also be hoisted via vi.hoisted().

const { mockRailwayRequest, eqShim, inArrayShim, toCamel } = vi.hoisted(() => {
  /** snake_case → camelCase */
  function toCamel(s: string): string {
    return s.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase());
  }

  /** Drizzle-like eq predicate factory */
  const eqShim =
    (field: { name: string }, value: unknown) =>
    (row: Record<string, unknown>) => {
      const camel = toCamel(field.name);
      return row[field.name] === value || row[camel] === value;
    };

  const inArrayShim =
    (field: { name: string }, values: unknown[]) =>
    (row: Record<string, unknown>) => {
      const camel = toCamel(field.name);
      const v = row[field.name] ?? row[camel];
      return (values as unknown[]).includes(v);
    };

  return {
    mockRailwayRequest: vi.fn(),
    eqShim,
    inArrayShim,
    toCamel,
  };
});

// ─── In-memory store ──────────────────────────────────────────────────────────

let store: Session[] = [];

/**
 * Minimal chainable query builder that mirrors the drizzle API used in
 * sessions.ts.  The update chain in sessions.ts sometimes ends with
 * .returning() and sometimes doesn't, so we return a thenable that also
 * exposes .returning().
 */
const createDb = () => ({
  insert: (_table: unknown) => ({
    values: (row: Session) => ({
      returning: async () => {
        const saved = { ...row };
        store.push(saved);
        return [saved];
      },
    }),
  }),

  select: () => ({
    from: (_table: unknown) => ({
      // listSessions uses .orderBy() as terminal call
      orderBy: async () => [...store],
      // getSession uses .where() as terminal call
      where: async (pred: (s: Session) => boolean) => store.filter(pred),
    }),
  }),

  update: (_table: unknown) => ({
    set: (patch: Partial<Session>) => ({
      where: (pred: (s: Session) => boolean) => {
        const apply = () => {
          store = store.map((s) => (pred(s) ? { ...s, ...patch } : s));
        };
        // Thenable so `await db.update(...).set(...).where(...)` works
        return {
          then(
            resolve: (v: Session[]) => void,
            reject: (e: unknown) => void,
          ): void {
            try {
              apply();
              resolve(store);
            } catch (e) {
              reject(e);
            }
          },
          // .returning() variant used in the final deleteSession update
          returning: async () => {
            apply();
            return store.filter(pred);
          },
        };
      },
    }),
  }),
});

// ─── Module mocks ─────────────────────────────────────────────────────────────

vi.mock("../railway/client.js", () => ({
  railwayRequest: mockRailwayRequest,
}));

vi.mock("../db/client.js", () => ({
  get db() {
    return createDb();
  },
}));

vi.mock("../config.js", () => ({
  config: {
    localMode: false,
    railwayProjectId: "proj-test",
    railwayEnvironmentId: "env-test",
    railwayServiceImage: "ghcr.io/test/image:latest",
    sandboxRepoUrl: "https://github.com/test/repo.git",
    githubToken: undefined,
    anthropicApiKey: undefined,
    openaiApiKey: undefined,
  },
  railwayAccountCount: () => 2,
}));

vi.mock("../railway/mutations.js", () => ({
  serviceCreateMutation: "mutation serviceCreate { id name }",
  serviceDeleteMutation: "mutation serviceDelete($id: String!) { serviceDelete(id: $id) }",
  variableCollectionUpsertMutation: "mutation variableCollectionUpsert { variableCollectionUpsert }",
}));

vi.mock("drizzle-orm", async (importOriginal) => {
  const original = await importOriginal<typeof import("drizzle-orm")>();
  return {
    ...original,
    eq: eqShim,
    desc: (field: unknown) => field,
    inArray: inArrayShim,
  };
});

// ─── Import SUT after mocks ───────────────────────────────────────────────────

import {
  createSession,
  deleteSession,
  getSession,
  listSessions,
} from "../services/sessions.js";

// ─── Test setup ───────────────────────────────────────────────────────────────

beforeEach(() => {
  store = [];
  mockRailwayRequest.mockReset();
});

// ─── createSession ────────────────────────────────────────────────────────────

describe("createSession", () => {
  it("creates a Railway service and returns a session with status 'starting'", async () => {
    mockRailwayRequest
      .mockResolvedValueOnce({ serviceCreate: { id: "rwy-svc-001", name: "t27-sandbox-test" } })
      .mockResolvedValueOnce({ variableCollectionUpsert: true });

    const session = await createSession({ name: "t27-sandbox-test" });

    expect(session.status).toBe("starting");
    expect(session.name).toBe("t27-sandbox-test");
    expect(session.railwayServiceId).toBe("rwy-svc-001");
    expect(session.id).toBeTruthy();
  });

  it("uses the session name provided by the caller", async () => {
    mockRailwayRequest.mockResolvedValue({
      serviceCreate: { id: "svc-named", name: "my-custom-name" },
    });

    const session = await createSession({ name: "my-custom-name" });
    expect(session.name).toBe("my-custom-name");
  });

  it("generates a name when none is provided (matches t27-sandbox-<timestamp> pattern)", async () => {
    mockRailwayRequest.mockResolvedValue({
      serviceCreate: { id: "svc-gen", name: "t27-sandbox-12345" },
    });

    const session = await createSession({});
    expect(session.name).toMatch(/^t27-sandbox-\d+$/);
  });

  it("calls variableCollectionUpsert when environment variables are present", async () => {
    mockRailwayRequest
      .mockResolvedValueOnce({ serviceCreate: { id: "svc-vars", name: "svc" } })
      .mockResolvedValueOnce({ variableCollectionUpsert: true });

    await createSession({
      name: "svc",
      taskDescription: "Build a feature",
      repoUrl: "https://github.com/x/y.git",
      branch: "main",
    });

    expect(mockRailwayRequest).toHaveBeenCalledTimes(2);
    const [, vars, acctIdx] = mockRailwayRequest.mock.calls[1];
    expect(vars.input.variables).toMatchObject({
      SANDBOX_REPO_URL: "https://github.com/x/y.git",
      SANDBOX_BRANCH: "main",
      TASK_DESCRIPTION: "Build a feature",
    });
    // Both Railway calls must use the same accountIndex
    expect(acctIdx).toBe(mockRailwayRequest.mock.calls[0][2]);
  });

  it("persists the railwayAccountIndex in the session record", async () => {
    mockRailwayRequest.mockResolvedValue({
      serviceCreate: { id: "svc-acct", name: "svc" },
    });

    const session = await createSession({ name: "svc" });
    expect(typeof session.railwayAccountIndex).toBe("number");
  });

  it("uses round-robin account selection (alternates between accounts)", async () => {
    mockRailwayRequest.mockResolvedValue({
      serviceCreate: { id: "svc-rr", name: "svc" },
    });

    const s1 = await createSession({ name: "svc-1" });
    const s2 = await createSession({ name: "svc-2" });

    // With a pool of 2, consecutive sessions should use different accounts
    expect(s1.railwayAccountIndex).not.toBe(s2.railwayAccountIndex);
  });

  it("throws HttpError(502) when Railway returns no service id", async () => {
    mockRailwayRequest.mockResolvedValueOnce({
      serviceCreate: { id: null, name: null },
    });

    await expect(createSession({ name: "svc" })).rejects.toMatchObject({
      status: 502,
      message: expect.stringContaining("missing service id"),
    });
  });
});

// ─── getSession ───────────────────────────────────────────────────────────────

describe("getSession", () => {
  it("returns the session when it exists", async () => {
    const id = "session-known";
    store.push({
      id,
      name: "my-session",
      status: "active",
      railwayServiceId: "rwy-svc",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    const session = await getSession(id);
    expect(session.id).toBe(id);
  });

  it("throws HttpError(404) for an unknown session ID", async () => {
    await expect(getSession("nonexistent-id")).rejects.toMatchObject({
      status: 404,
      message: expect.stringContaining("not found"),
    });
  });
});

// ─── listSessions ─────────────────────────────────────────────────────────────

describe("listSessions", () => {
  it("returns all sessions from the store", async () => {
    store.push(
      {
        id: "a",
        name: "svc-a",
        status: "active",
        railwayServiceId: null,
        railwayAccountIndex: null,
        taskDescription: null,
        repoUrl: null,
        branch: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      {
        id: "b",
        name: "svc-b",
        status: "starting",
        railwayServiceId: null,
        railwayAccountIndex: null,
        taskDescription: null,
        repoUrl: null,
        branch: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      },
    );

    const result = await listSessions();
    expect(result.length).toBe(2);
  });
});

// ─── deleteSession ────────────────────────────────────────────────────────────

const makeSession = (overrides: Partial<Session> = {}): Session => ({
  id: "session-del",
  name: "svc-del",
  status: "active",
  railwayServiceId: "rwy-del",
  railwayAccountIndex: 0,
  taskDescription: null,
  repoUrl: null,
  branch: null,
  createdAt: new Date(),
  updatedAt: new Date(),
  ...overrides,
});

describe("deleteSession", () => {
  it("calls serviceDelete and marks the session as 'deleted'", async () => {
    store.push(makeSession());
    mockRailwayRequest.mockResolvedValueOnce({ serviceDelete: true });

    await deleteSession("session-del");

    expect(mockRailwayRequest).toHaveBeenCalledOnce();
    const [mutation] = mockRailwayRequest.mock.calls[0];
    expect(mutation).toContain("serviceDelete");

    const stored = store.find((s) => s.id === "session-del");
    expect(stored?.status).toBe("deleted");
  });

  it("is a no-op when session is already deleted", async () => {
    store.push(makeSession({ status: "deleted" }));

    const result = await deleteSession("session-del");

    expect(mockRailwayRequest).not.toHaveBeenCalled();
    expect(result.status).toBe("deleted");
  });

  it("restores the previous status when the Railway API call fails", async () => {
    store.push(makeSession({ status: "active" }));
    mockRailwayRequest.mockRejectedValueOnce(new HttpError(502, "Railway error"));

    await expect(deleteSession("session-del")).rejects.toMatchObject({
      status: 502,
    });

    // Status should be restored to 'active', not stuck at 'terminating'
    const stored = store.find((s) => s.id === "session-del");
    expect(stored?.status).toBe("active");
  });

  it("throws HttpError(404) when the session does not exist", async () => {
    await expect(deleteSession("ghost-session")).rejects.toMatchObject({
      status: 404,
    });
  });

  it("skips the Railway API call when railwayServiceId is null", async () => {
    store.push(makeSession({ railwayServiceId: null }));

    await deleteSession("session-del");

    expect(mockRailwayRequest).not.toHaveBeenCalled();
    const stored = store.find((s) => s.id === "session-del");
    expect(stored?.status).toBe("deleted");
  });
});
