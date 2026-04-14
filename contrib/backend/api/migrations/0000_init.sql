-- T27 Sandbox Sessions — Initial Migration
-- Generated from contrib/backend/api/src/db/schema.ts (Drizzle ORM)
-- Run: psql $DATABASE_URL < migrations/0000_init.sql

CREATE TABLE IF NOT EXISTS "sessions" (
  "id"                      TEXT        PRIMARY KEY,
  "name"                    TEXT        NOT NULL,
  "status"                  TEXT        NOT NULL DEFAULT 'starting',
  "railway_service_id"      TEXT,
  "railway_account_index"   INTEGER,
  "task_description"        TEXT,
  "repo_url"                TEXT,
  "branch"                  TEXT,
  "created_at"              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at"              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Automatically update updated_at on every row modification.
-- This prevents the bug where code paths forget to set updatedAt manually.
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_sessions_updated_at
  BEFORE UPDATE ON sessions
  FOR EACH ROW
  EXECUTE FUNCTION trigger_set_updated_at();

-- Index for the health polling query (status IN ('starting', 'active'))
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions (status)
  WHERE status IN ('starting', 'active');

-- Index for listing sessions ordered by creation time
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions (created_at DESC);
