-- ============================================================
-- Migration 001: experiment_queue table
-- Project : t27 / GoldenFloat Family — WAVE-GF-001
-- Date    : 2026-05-02
-- Engine  : Neon (PostgreSQL)
-- ============================================================

CREATE TABLE IF NOT EXISTS experiment_queue (
    id            SERIAL PRIMARY KEY,
    canon_name    TEXT        NOT NULL UNIQUE,   -- e.g. "WAVE-GF-001-P1-k0-s34"
    phase         SMALLINT    NOT NULL,           -- 0=Baseline 1=phi-LR 2=Opt 3=Arch 4=Dtype 5=Hybrid
    status        TEXT        NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending','running','done','failed','skip')),
    priority      SMALLINT    NOT NULL DEFAULT 5, -- 1=highest … 9=lowest
    seed          INTEGER,
    lr            DOUBLE PRECISION,
    optimizer     TEXT,
    d_model       INTEGER,
    dtype         TEXT,
    config_json   JSONB,
    result_json   JSONB,
    error_msg     TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eq_status_priority
    ON experiment_queue (status, priority);

CREATE INDEX IF NOT EXISTS idx_eq_phase
    ON experiment_queue (phase);

COMMENT ON TABLE experiment_queue IS
    'GoldenFloat WAVE-GF-001 experiment queue — phi-seeded training runs';
