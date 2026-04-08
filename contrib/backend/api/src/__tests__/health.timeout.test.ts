/**
 * Tests for src/services/health.ts - checkSessionTimeout function
 *
 * SANDBOX-010: Session Timeout Enforcement
 *
 * External dependencies are mocked:
 *   - ../sessions.js       → deleteSession spy
 *   - ../../config.js          → maxSessionDurationMs value
 *   - drizzle-orm           → eq/inArray shims
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import type { Session } from "../db/schema.js";

// ─── Hoisted helpers ─────────────────────────────────────────────────────────
// vi.mock factories are hoisted before all top-level code.

const { eqShim, inArrayShim } = vi.hoisted(() => {
  /** Drizzle-like eq predicate factory */
  const eqShim =
    (field: { name: string }, value: unknown) =>
    (row: Record<string, unknown>) => {
      return row[field.name] === value;
    };

  const inArrayShim =
    (field: { name: string }, values: unknown[]) =>
    (row: Record<string, unknown>) => {
      const v = row[field.name];
      return (values as unknown[]).includes(v);
    };

  return {
    eqShim,
    inArrayShim,
  };
});

// ─── Mock store ─────────────────────────────────────────────────────────────

let deletedSessionCalls: string[] = [];

const mockDeleteSession = vi.fn(async (id: string) => {
  deletedSessionCalls.push(id);
});

// ─── Module mocks ─────────────────────────────────────────────────────

vi.mock("../../config.js", () => ({
  config: {
    maxSessionDurationMs: 3_600_000, // 1 hour
  },
}));

vi.mock("drizzle-orm", async (importOriginal) => {
  const original = await importOriginal<typeof import("drizzle-orm")>();
  return {
    ...original,
    eq: eqShim,
    inArray: inArrayShim,
  };
});

vi.mock("../sessions.js", () => ({
  deleteSession: mockDeleteSession,
}));

// ─── Import SUT after mocks ───────────────────────────────────────────

import { checkSessionTimeout } from "../services/health.js";

// ─── Test setup ───────────────────────────────────────────────────────

beforeEach(() => {
  deletedSessionCalls = [];
  mockDeleteSession.mockClear();
});

// ─── checkSessionTimeout tests ───────────────────────────────────────────

describe("checkSessionTimeout", () => {
  it("terminates session older than maxDuration (61 minutes)", async () => {
    const session: Session = {
      id: "test-123",
      name: "test-session",
      status: "active",
      railwayServiceId: "rwy-123",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 3_700_000), // 61 минут назад (> 1 часа)
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).toHaveBeenCalledTimes(1);
    expect(mockDeleteSession).toHaveBeenCalledWith("test-123");
    expect(deletedSessionCalls).toEqual(["test-123"]);
  });

  it("does NOT terminate session within maxDuration (30 minutes)", async () => {
    const session: Session = {
      id: "test-456",
      name: "test-session",
      status: "active",
      railwayServiceId: "rwy-456",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 1_800_000), // 30 минут назад (< 1 часа)
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).not.toHaveBeenCalled();
    expect(deletedSessionCalls).toEqual([]);
  });

  it("does NOT terminate non-active session (starting status)", async () => {
    const session: Session = {
      id: "test-starting",
      name: "test-session",
      status: "starting", // не active
      railwayServiceId: "rwy-starting",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 4_000_000), // 66 минут назад (> 1 часа)
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).not.toHaveBeenCalled();
    expect(deletedSessionCalls).toEqual([]);
  });

  it("does NOT terminate failed session", async () => {
    const session: Session = {
      id: "test-failed",
      name: "test-session",
      status: "failed", // не active
      railwayServiceId: "rwy-failed",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 4_000_000), // 66 минут назад
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).not.toHaveBeenCalled();
    expect(deletedSessionCalls).toEqual([]);
  });

  it("does NOT terminate deleted session", async () => {
    const session: Session = {
      id: "test-deleted",
      name: "test-session",
      status: "deleted", // уже удалена
      railwayServiceId: "rwy-deleted",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 4_000_000),
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).not.toHaveBeenCalled();
    expect(deletedSessionCalls).toEqual([]);
  });

  it("handles exact threshold (exactly 1 hour = terminate)", async () => {
    const session: Session = {
      id: "test-threshold",
      name: "test-session",
      status: "active",
      railwayServiceId: "rwy-threshold",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 3_600_000), // ровно 1 час
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).toHaveBeenCalledTimes(1);
    expect(mockDeleteSession).toHaveBeenCalledWith("test-threshold");
    expect(deletedSessionCalls).toEqual(["test-threshold"]);
  });

  it("handles just under threshold (3599 seconds = no terminate)", async () => {
    const session: Session = {
      id: "test-under",
      name: "test-session",
      status: "active",
      railwayServiceId: "rwy-under",
      railwayAccountIndex: 0,
      taskDescription: null,
      repoUrl: null,
      branch: null,
      createdAt: new Date(Date.now() - 3_599_000), // 3599 сек (< 1 часа)
      updatedAt: new Date(),
    };

    await checkSessionTimeout(session);

    expect(mockDeleteSession).not.toHaveBeenCalled();
    expect(deletedSessionCalls).toEqual([]);
  });
});
