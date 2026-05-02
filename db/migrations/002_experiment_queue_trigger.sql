-- ============================================================
-- Migration 002: auto-update updated_at on experiment_queue
-- ============================================================

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_experiment_queue_updated_at
    ON experiment_queue;

CREATE TRIGGER trg_experiment_queue_updated_at
    BEFORE UPDATE ON experiment_queue
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
