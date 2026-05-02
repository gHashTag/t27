-- ============================================================
-- Seed: Phase 0 — Baseline (fp32 IEEE, seed=42, AdamW lr=1e-3)
-- ============================================================
INSERT INTO experiment_queue
    (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json)
VALUES
    ('WAVE-GF-001-P0-baseline-fp32',
     0, 1, 42, 1e-3, 'adamw', 256, 'fp32',
     '{"desc":"IEEE fp32 baseline, AdamW lr=1e-3, seed=42, d=256","batch":64,"steps":27000}');
