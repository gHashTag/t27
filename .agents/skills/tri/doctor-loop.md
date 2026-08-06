# Doctor Loop — Watchdog, Self-Healing, and Safety

Doctor agent runs independently of task agents. Never takes regular PHI LOOP tasks.

## Agent Definition

```json
{
  "agent_id": "tri-doctor",
  "role": "Swarm Health & Recovery",
  "type": "watchdog",
  "runs_independently": true,
  "takes_feature_tasks": false,
  "schedule": "periodic: 60s"
}
```

## Scope

Doctor monitors:
- **SacredPhysics** — TRINITY, G, ΩΛ, tpresent, γ conformance
- **Numeric** — GoldenFloat family, TF, IPS formats
- **Graph** — Dependency graph integrity, topology, tier violations
- **Compiler** — Parser, codegen, type checking
- **Runtime** — CLI commands, execution, validation
- **QueenLotus** — Swarm orchestration, task queue, claim ownership
- **Swarm** — Overall: toxic rate, repeat failures, stuck tasks

## Continuous Monitoring

Every 60 seconds, Doctor:

1. **Reads state:**
   - `.trinity/state/queen-health.json`
   - `.trinity/state/swarm-health.json`
   - `.trinity/state/ownership-index.json`

2. **Analyzes events:**
   - Scans `.trinity/events/akashic-log.jsonl` for anomalies
   - Checks for toxic verdict trends
   - Identifies stale claims (no heartbeat > TTL)
   - Detects missing FUTURE in loop.handoff

3. **Emits health event:**
   ```json
   {"event":"health.snapshot","queen_health":0.95,"swarm_health":0.92,"toxic_rate":0.02}
   ```

## Anomaly Detection

Doctor detects these anomalies:

| Type | Trigger | Severity | Action |
|-------|---------|----------|--------|
| `toxic_rate_increase` | Toxic rate > 5% | Warning: emit health.anomaly |
| `stale_claim_detected` | Claim expired, no heartbeat | Critical: emit health.anomaly + reclaim |
| `missing_future_handoff` | Loop completed without FUTURE | Warning: emit health.anomaly |
| `domain_below_threshold` | Domain health < 0.5 | Critical: emit health.anomaly + domain recovery |
| `state_divergence` | State != events.replay | Error: emit health.anomaly + emergency alert |
| `queen_below_critical` | queen_health < 0.5 | Critical: BLOCK all mutations |

## Self-Healing Procedures

### 1. Stale Claim Recovery

When stale claim detected:
```bash
# Reclaim claim
./scripts/replay-step.sh --reclaim-claim <claim_id>

# Write claim.reclaim event
echo '{"event":"claim.reclaim",...}' >> .trinity/events/akashic-log.jsonl
```

### 2. Domain Recovery

When domain health drops below threshold:
```bash
# Domain-specific recovery
./scripts/replay-step.sh --domain <affected>

# Emit recovery.start event
echo '{"event":"recovery.start","type":"domain","domain":"<domain>"}' >> .trinity/events/akashic-log.jsonl
```

### 3. Queen Recovery

When queen_health < 0.5:
```bash
# Full swarm recovery
./scripts/replay-step.sh --last-clean

# BLOCK all new mutations
echo "QUEEN CRITICAL: Only recovery allowed" >> .trinity/state/queen-health.json

# Notify agents
for agent in $(ls .trinity/agents/); do
    # Send notification (implementation-dependent)
done
```

## Doctor Loop Schedule

```bash
# Every 60 seconds
while true; do
    # Read state and events
    read .trinity/state/*.json
    tail -100 .trinity/events/akashic-log.jsonl

    # Analyze for anomalies
    # ... (analysis logic)

    # If anomaly detected:
    #   - Emit health.anomaly event
    #   - Trigger appropriate recovery
    #   - Log action taken

    sleep 60
done
```

## Integration with /tri

Doctor Loop should be integrated into tri CLI:

```bash
# Start Doctor Loop
tri doctor start

# Stop Doctor Loop
tri doctor stop

# Show Doctor status
tri doctor status

# Show recent anomalies
tri doctor anomalies
```

## Status Output

```
╔══════════════════════════════════════════════════════╗
║                    Tri Doctor Status                           ║
╚════════════════════════════════════════════════════════╝

Queen Health:     0.95 (GREEN)
Swarm Health:      0.92 (GREEN)
Toxic Rate:       0.02 (2% of 127 steps)
Stale Claims:      0
Missing FUTURE:    0

Recent Anomalies:
- [2026-04-04T13:00:00Z] health.anomaly: Numeric toxic_rate 15%↑ → WARNING
- [2026-04-03T10:30:00Z] claim.reclaim: expired claim on specs/base/ops.t27 → RECLAIMED

Active Recoveries:
- [recovery-001] Started at 12:00 (GF8 toxic recovery) — IN PROGRESS

─────────────────────────────────────
Doctor Loop: ACTIVE (running since 2026-04-04T12:00:00Z)
Next check in: 58 seconds
```

## Permissions

| Permission | Description |
|-----------|-------------|
| `read.trinity` | Read all .trinity files (events, claims, queue, state, experience) |
| `write.state` | Write .trinity/state/* files |
| `write.events` | Append to .trinity/events/akashic-log.jsonl |
| `write.claims` | Write .trinity/claims/{active,released}/* |
| `reclaim` | Reclaim stale claims (write + release) |
| `schedule` | Add recovery tasks to .trinity/queue/ |
| `emit.health` | Emit health events to .trinity/events |

Doctor NEVER needs:
- `write.spec` — Not allowed to modify specs
- `tri.skill.*` — Not allowed to run PHI LOOP
- Direct mutation — Only triggers recovery
