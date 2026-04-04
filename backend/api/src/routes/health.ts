import { Router } from "express";
import { sql } from "drizzle-orm";
import { db } from "../db/client.js";

const router = Router();

/**
 * GET /health
 *
 * Returns { status: "ok" } when everything is up.
 * Returns 503 with { status: "degraded" } if the database is unreachable.
 */
router.get("/", async (_req, res) => {
  try {
    await db.execute(sql`select 1`);
    res.json({ status: "ok", database: "ok" });
  } catch {
    res.status(503).json({ status: "degraded", database: "unavailable" });
  }
});

export default router;
