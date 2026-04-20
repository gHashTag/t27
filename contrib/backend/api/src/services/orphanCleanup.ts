import { eq, and, lt, not, inArray } from "drizzle-orm";

import { config } from "../config.js";
import { db } from "../db/client.js";
import { sessions } from "../db/schema.js";
import { logger } from "../utils/logger.js";
import { deleteSession } from "./sessions.js";

const ORPHAN_IDLE_MS = config.maxSessionDurationMs;
const CLEANUP_INTERVAL_MS = 5 * 60_000;

export const cleanupOrphanedSessions = async (): Promise<number> => {
  const now = Date.now();

  const orphans = await db
    .select()
    .from(sessions)
    .where(
      and(
        inArray(sessions.status, ["starting", "active"]),
        lt(sessions.updatedAt, new Date(now - ORPHAN_IDLE_MS)),
      ),
    );

  if (orphans.length === 0) return 0;

  logger.info("Orphaned sessions found", { count: orphans.length });

  let deleted = 0;
  await Promise.all(
    orphans.map(async (session) => {
      try {
        await deleteSession(session.id);
        deleted++;
      } catch (err) {
        logger.error("Failed to delete orphaned session", {
          sessionId: session.id,
          error: String(err),
        });
      }
    }),
  );

  return deleted;
};

export const startOrphanCleanup = (): NodeJS.Timeout => {
  const timer = setInterval(() => {
    cleanupOrphanedSessions().catch((err) =>
      logger.error("Orphan cleanup failed", { error: String(err) }),
    );
  }, CLEANUP_INTERVAL_MS);

  if (timer.unref) timer.unref();
  return timer;
};
