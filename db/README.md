# db/ — Neon PostgreSQL Migrations & Seeds

## Structure

```
db/
  migrations/
    001_create_experiment_queue.sql   — DDL: experiment_queue table + indexes
    002_experiment_queue_trigger.sql  — auto-update updated_at trigger
  seeds/
    wave_gf001_phase0_baseline.sql    — Phase 0: fp32 IEEE baseline (1 row)
    wave_gf001_phase1_phi_lr.sql      — Phase 1: phi-LR ladder Quick-3 (18 rows)
```

## Apply to Neon

```bash
# 1. Set connection string
export DATABASE_URL="postgres://..."

# 2. Run migrations (order matters)
psql $DATABASE_URL -f db/migrations/001_create_experiment_queue.sql
psql $DATABASE_URL -f db/migrations/002_experiment_queue_trigger.sql

# 3. Seed
psql $DATABASE_URL -f db/seeds/wave_gf001_phase0_baseline.sql
psql $DATABASE_URL -f db/seeds/wave_gf001_phase1_phi_lr.sql
```

## experiment_queue schema

| Column       | Type             | Notes                                          |
|-------------|-----------------|------------------------------------------------|
| id          | SERIAL PK       | Auto-increment                                 |
| canon_name  | TEXT UNIQUE     | e.g. `WAVE-GF-001-P1-k0-s34`                 |
| phase       | SMALLINT        | 0=Baseline 1=phi-LR 2=Opt 3=Arch 4=Dtype 5=Hybrid |
| status      | TEXT            | pending / running / done / failed / skip       |
| priority    | SMALLINT        | 1=highest … 9=lowest                          |
| seed        | INTEGER         | phi-series: 34, 55, 89, 144, 233…             |
| lr          | DOUBLE PRECISION| Learning rate (phi-ladder: alpha_phi·φ^(-k/2)) |
| optimizer   | TEXT            | adamw / lion / sgdm                            |
| d_model     | INTEGER         | 128 / 192 / 256 / 384 / 512 / 618             |
| dtype       | TEXT            | fp32 / gf16 / bf16 / fp16 / gf32              |
| config_json | JSONB           | Full run config                                |
| result_json | JSONB           | Populated after run completes                  |
| error_msg   | TEXT            | Populated on failure                           |
| created_at  | TIMESTAMPTZ     | Auto                                           |
| updated_at  | TIMESTAMPTZ     | Auto via trigger (002)                         |

## Useful queries

```sql
-- Dequeue next pending job (worker-safe)
SELECT * FROM experiment_queue
WHERE status = 'pending'
ORDER BY priority, id
LIMIT 1
FOR UPDATE SKIP LOCKED;

-- Mark as running
UPDATE experiment_queue SET status='running' WHERE canon_name='WAVE-GF-001-P1-k0-s34';

-- Save result
UPDATE experiment_queue
SET status='done', result_json='{"val_loss":2.341,"step":27000}'
WHERE canon_name='WAVE-GF-001-P1-k0-s34';

-- Phase 1 leaderboard
SELECT canon_name, seed, lr, status,
       result_json->>'val_loss' AS val_loss
FROM experiment_queue
WHERE phase = 1
ORDER BY (result_json->>'val_loss')::float NULLS LAST;
```

---
*Generated: 2026-05-02 | WAVE-GF-001 | phi² + phi⁻² = 3 · TRINITY*
