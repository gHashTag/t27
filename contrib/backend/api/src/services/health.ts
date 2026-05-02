import { eq, inArray } from "drizzle-orm";

import { config } from "../config.js";
import { db } from "../db/client.js";
import { sessions } from "../db/schema.js";
import { resolveSandboxHealthUrl } from "../utils/sandboxTarget.js";
import { deleteSession } from "./sessions.js";

const HEALTH_TIMEOUT_MS = 3_000;
const STARTUP_TIMEOUT_MS = 90_000;
const HEALTH_CHECK_STATUSES = ["starting", "active"] as const;

// ─────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────

const checkSandboxHealth = async (healthUrl: string): Promise<boolean> => {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), HEALTH_TIMEOUT_MS);

  try {
    const response = await fetch(healthUrl, { signal: controller.signal });
    return response.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(timeout);
  }
};

const updateSessionStatus = async (id: string, status: string) => {
  await db
    .update(sessions)
    .set({ status, updatedAt: new Date() })
    .where(eq(sessions.id, id));
};

// ─────────────────────────────────────────────────────────────
// Exported functions
// ─────────────────────────────────────────────────────────────

export const checkSessionTimeout = async (session: { id: string; createdAt: Date; status: string }): Promise<void> => {
  const maxDuration = config.maxSessionDurationMs;
  const elapsed = Date.now() - session.createdAt.getTime();

  if (elapsed > maxDuration && session.status === "active") {
    await deleteSession(session.id);
  }
};

// ─────────────────────────────────────────────────────────────
// Exported poller
// ─────────────────────────────────────────────────────────────

/**
 * Check health for all sessions in "starting" or "active" state and update
 * the DB accordingly.  Call this on a periodic interval from the server.
 */
export const pollSandboxHealth = async (): Promise<void> => {
  const candidates = await db
    .select()
    .from(sessions)
    .where(inArray(sessions.status, [...HEALTH_CHECK_STATUSES]));

  const now = Date.now();

  await Promise.all(
    candidates.map(async (session) => {
      const healthUrl = resolveSandboxHealthUrl(session.name);
      const isHealthy = await checkSandboxHealth(healthUrl);

      if (isHealthy) {
        if (session.status !== "active") {
          await updateSessionStatus(session.id, "active");
        }
        // Check session timeout
        await checkSessionTimeout(session);
        return;
      }

      // Still starting – check if we've exceeded the startup timeout
      if (
        session.status === "starting" &&
        session.createdAt &&
        now - session.createdAt.getTime() > STARTUP_TIMEOUT_MS
      ) {
        await updateSessionStatus(session.id, "failed");
        return;
      }

      // Was active but went unhealthy – flip back to starting so we keep
      // checking; callers can detect this as a degraded state.
      if (session.status === "active") {
        await updateSessionStatus(session.id, "starting");
      }
    }),
  );
};
