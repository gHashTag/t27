# Example: Queen Recovery — Unhealthy to Healthy

This example demonstrates queen recovery when queen health drops below threshold.

## Scenario

Queen health drops due to repeated toxic verdicts in SacredPhysics domain.

## Initial State

After several failed mutations:

```
./scripts/swarm-health.sh --json

{
  "queen_health": 0.42,
  "status": "RED",
  "domains": {
    "sacredphysics": 0.0,
    "numeric": 0.8,
    "graph": 1.0,
    "compiler": 1.0,
    "runtime": 1.0,
    "queenlotus": 0.9
  },
  "timestamp": "2026-04-04T12:00:00Z"
}
```

**Status:** RED (Critical)
- SacredPhysics domain: 0.0 (unhealthy)
- Queen health: 0.42 (< 0.5 threshold)

## 1. Queen-Critical Recovery Activated

**Decision:** Block feature mutations, enable recovery mode only.

```bash
# Check toxic verdict statistics
./scripts/swarm-health.sh

📊 Toxic Verdict Statistics:
  Toxic Rate: 25% (3/12)

🔄 Repeated Failures:
    specs/math/sacred_physics.t27: 3 times

🔒 Last Clean Seal:
  Seal: seal_abc123
  Spec: specs/math/sacred_physics.t27
  At: 2026-04-04T11:30:00Z

⚡ Operating Status:
  CRITICAL — Only recovery loops allowed
  → Run: tri replay-step --last-clean
```

## 2. Isolate Failing Domain

Identify SacredPhysics as the failing domain.

```bash
# Check domain-specific health
tri health sacred

SacredPhysics Health: 0.0 (RED)

Failing tests:
  - test_trinity_exact_tolerance: FAILED
  - test_gamma_relative_tolerance: FAILED
  - test_g_absolute_tolerance: FAILED
```

**Root Cause:** Recent mutation to sacred_physics.t27 modified constants incorrectly.

## 3. Replay from Last Clean Seal

```bash
./scripts/replay-step.sh --last-clean

🔄 Replaying from clean seal...
  Seal ID: seal_abc123
  Spec: specs/math/sacred_physics.t27
  Skill: sacred_constants_v1

⚠️  Warning: Spec hash differs from seal
  Current: e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0
  Sealed: a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3

Restore spec from seal? (y/N) y
  Restoring spec to sealed state...

🔄 Re-running PHI LOOP from seal...
  tri gen
  tri test
  tri verdict --sacred

✅ Replay complete. Continue PHI LOOP from here.
```

## 4. Re-Verify Health

```bash
./scripts/swarm-health.sh

╔═══════════════════════════════════════════════════════════════╗
║         Queen Trinity Health Report                        ║
╚════════════════════════════════════════════════════════════════╝

👑 Queen Health: 0.98 (GREEN)

📋 Domain Health:
  Sacredphysics     1.00 (GREEN)
  Numeric           0.80 (YELLOW)
  Graph             1.00 (GREEN)
  Compiler          1.00 (GREEN)
  Runtime           1.00 (GREEN)
  Queenlotus        0.90 (GREEN)

⚡ Operating Status:
  HEALTHY — All operations allowed
```

## 5. Decision Log Entry

```jsonl
{"trace_id":"550e8405-6789-9f0e-4a0b-1c2d3e4f5a6","timestamp":"2026-04-04T12:00:00Z","task_id":"queen-recovery","skill_id":null,"event_type":"recovery","spec_path":"specs/math/sacred_physics.t27","spec_hash_before":"e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0","spec_hash_after":"a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3","gen_hash_after":null,"test_vector_hash":null,"tool":"replay-step","decision_summary":"Queen recovery: replay from last clean seal","guardrail_hit":false,"policy_version":"1.0.0","queen_health_before":{"spec":0.0,"graph":1.0,"bench":1.0,"swarm":0.4,"queen":0.42},"queen_health_after":{"spec":1.0,"graph":1.0,"bench":1.0,"swarm":0.95,"queen":0.98},"verdict":"clean","bench_delta":null,"recovery_action":"replay_from_last_clean","sealed_at":"2026-04-04T11:30:00Z","duration_ms":2345}
```

## 6. Resume Normal Operations

After queen health returns to GREEN:

```bash
# Feature mutations allowed again
tri skill begin "new-feature-task"
tri spec edit <module>
tri skill seal --hash
tri gen
tri test
tri verdict --toxic
tri experience save
tri skill commit
tri git commit -m "feat(...): ..."

# Queen health tracked automatically:
# queen_health_before: 0.98
# queen_health_after: 0.99
```

## Key Takeaways

- **Queen-first recovery**: SacredPhysics unhealthy → only recovery allowed
- **Replay over restart**: Preserves context, faster than full restart
- **Health telemetry**: Every step records queen_health_before/after
- **Decision logging**: Structured JSONL for observability and root-cause analysis
- **Guardrails**: Automatic blocking when queen_health < threshold

## Self-Healing Loop (Automated)

```bash
# Automated recovery for repeated failures
tri self-heal --max-iterations 5

# Executes:
# 1. Check queen health
# 2. Identify lowest domain
# 3. Apply domain-specific recovery
# 4. Re-verify health
# 5. Continue or escalate
# 6. Stop after 5 iterations if still unhealthy
```

## See Also

- `references/queen-health.md` — Health domains and thresholds
- `references/recovery-policy.md` — Recovery procedures
- `scripts/replay-step.sh` — Replay implementation
- `scripts/swarm-health.sh` — Health aggregation
