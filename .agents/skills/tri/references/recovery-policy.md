# Recovery Policy Reference

This document defines recovery and rollback procedures for Trinity S³AI.

## Recovery Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│ Recovery Priority (Queen-First)                            │
├─────────────────────────────────────────────────────────────────┤
│ 1. Queen Critical     │ 2. Domain Critical  │ 3. Local │
│    - queen_health < 0.5    │    - spec_health = 0 │    - Single skill │
│    - Sacred physics violation│    - graph_cycle      │    - Retry once   │
│    - Swarm toxic > 50%   │    - Tier violation   │                │
└─────────────────────────────────────────────────────────────────┘
```

## Recovery Procedures

### Level 1: Queen Critical (queen_health < 0.5)

**Condition:** Trinity Queen is unhealthy across multiple domains.

**Actions:**
1. **Stop all mutations**: Block feature development
2. **Enable recovery mode**: Only repair operations allowed
3. **Identify root cause**: Check health signals
4. **Isolate failing domain**: Prevent cascade
5. **Replay last clean seal**: `tri replay-step --last-clean`
6. **Re-verify**: Run full health check
7. **Manual approval**: Required before resuming normal ops

**Command:**
```bash
tri recovery queen-critical
```

### Level 2: Domain Critical

**Condition:** Single domain is critically unhealthy.

| Domain | Trigger | Recovery |
|---------|----------|------------|
| SacredPhysics | TRINITY/G/γ/ΩΛ violation | Restore spec constants |
| Numeric | GF/TF/IPS conformance fail | Rollback format spec |
| Graph | Cycle or tier violation | Fix graph.tri |
| Compiler | Generation failure or type error | Fix syntax, regenerate |
| Runtime | Execution error or crash | Debug runtime, retry |
| QueenLotus | Orchestration failure | Restart orchestrator |

**Command:**
```bash
tri recovery domain --affected <domain>
```

### Level 3: Local Failure (Single Skill)

**Condition:** Individual skill fails, other domains healthy.

**Actions:**
1. **Identify failure**: Check verdict, logs, hashes
2. **Retry once**: Same mutation, fresh attempt
3. **If retry fails**: Rollback to spec_hash_before
4. **Rollback spec**: `git restore <spec-file>`
5. **Invalidate skill**: Mark skill as failed in registry
6. **Record mistake**: `tri experience record --mistake`

**Command:**
```bash
tri recovery skill --id <skill_id> --retry-once
```

## Replay vs Restart

**Best Practice:** Use replay instead of full restart.

| Method | When to Use | Advantages |
|--------|--------------|-------------|
| **Replay** | Last clean seal known | Preserves state, faster |
| **Restart** | No clean seal exists | Clean slate, loss of progress |

**Replay Procedure:**
```bash
# Find last clean seal
tri seal list --status clean

# Replay from that seal
tri replay-step --seal <seal_id>

# Continue workflow
tri gen
tri test
tri verdict --toxic
```

## Toxic Verdict Recovery

When `tri verdict --toxic` returns toxic:

```bash
# Step 1: Record mistake
tri experience record --mistake \
  --reason "toxic verdict" \
  --spec <spec_path> \
  --verdict toxic \
  --details "$(cat toxic-report.json)"

# Step 2: Rollback SPEC (never generated code)
git restore <spec_path>

# Step 3: Replay last clean seal
tri replay-step --last-clean

# Step 4: Continue with corrected mutation
```

## Cascade Prevention

**Prevent failure cascade across domains:**

```
if spec_health == 0:
    BLOCK: graph mutations (graph_health could be affected)
    BLOCK: compiler generation (invalid input)
    ALLOW: spec repair only

if graph_health == 0:
    BLOCK: compiler generation (invalid structure)
    BLOCK: runtime execution (unsafe)
    ALLOW: graph repair only

if compiler_health == 0:
    BLOCK: runtime execution (invalid code)
    ALLOW: compiler repair only
```

## Self-Healing Loop

Automated recovery for common failures:

```
while queen_health < 0.9:
    1. Identify lowest health domain
    2. Apply domain-specific recovery
    3. Re-verify health
    4. If improved: continue
       Else: escalate to recovery level up
    5. If queen_critical: stop, notify
```

**Command:**
```bash
tri self-heal --max-iterations 5
```

## Recovery Logs

All recovery actions are logged for observability:

```json
{
  "recovery_id": "uuid",
  "timestamp": "ISO-8601",
  "level": "queen-critical|domain-critical|local",
  "domain_affected": "sacred|numeric|graph|compiler|runtime|lotus",
  "health_before": {...},
  "health_after": {...},
  "action_taken": "replay|rollback|repair|escalate",
  "result": "success|failure|escalated",
  "skill_affected": "<skill_id>",
  "seal_used": "<seal_id>"
}
```

## See Also

- `references/queen-health.md` — Health monitoring
- `scripts/replay-step.sh` — Replay implementation
- `examples/queen-recovery.md` — Recovery scenarios
