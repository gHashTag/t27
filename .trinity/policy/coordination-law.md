# Trinity Coordination Law

Before any task, every agent must read `.trinity` Akashic Chronicle, inspect active claims, queue, and swarm state, then acquire an exclusive claim on its target spec_path or graph_node. No mutation without prior read + claim.

## Canonical Trinity Structure

```
.trinity/
├── events/              # Immutable append-only journal
│   └── akashic-log.jsonl
├── claims/              # Temporary ownership
│   ├── active/         # Current active claims
│   └── released/       # Historical claims
├── queue/               # Task management
│   ├── pending.json
│   ├── active.json
│   ├── blocked.json
│   └── done.json
├── experience/           # Learned memory
│   └── episodes.jsonl
├── state/               # Derived current reality
│   ├── queen-health.json
│   ├── swarm-health.json
│   └── ownership-index.json
└── policy/              # Coordination rules
    └── coordination-law.md
```

## Agent Startup Protocol

Every agent must execute this sequence before starting any task:

1. **Read Chronicle**: Append-read `.trinity/events/akashic-log.jsonl` for recent events
2. **Inspect Claims**: Read `.trinity/claims/active/` to see what's claimed
3. **Check Queue**: Read `.trinity/queue/` to find pending tasks
4. **Verify Health**: Read `.trinity/state/queen-health.json` and `.trinity/state/swarm-health.json`
5. **Acquire Claim**: Create exclusive claim on target resource with TTL
6. **Record Intent**: Append task.intent event to events
7. **Begin Mutation**: Only after claim is active

## Task Intent Protocol

Before any mutation, agent must:

1. **Check**: Is task_id already in `.trinity/queue/active.json`?
2. **If claimed**: Do NOT work on it. Options:
   - Wait for release
   - Pick different task_id
   - Request handoff (if owner is stuck)
3. **If not claimed**: Proceed with claim acquisition

## Claim Protocol

### Acquire Claim

```bash
# 1. Check if resource is available
if [[ -f ".trinity/claims/active/<spec_path>.json" ]]; then
    echo "Resource claimed by another agent"
    # Read claim to see owner and TTL
    cat ".trinity/claims/active/<spec_path>.json"
    exit 1
fi

# 2. Create claim with TTL
claim_id=$(uuidgen)
cat > ".trinity/claims/active/<spec_path>.json" <<EOF
{
  "claim_id": "$claim_id",
  "agent_id": "$AGENT_ID",
  "spec_path": "$spec_path",
  "graph_node": "$GRAPH_NODE",
  "task_id": "$TASK_ID",
  "acquired_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "ttl_sec": 1800,
  "expires_at": "$(date -u -d '+30 minutes' +%Y-%m-%dT%H:%M:%SZ)",
  "heartbeat_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

# 3. Record claim event
echo '{"event":"claim.acquire","claim_id":"'$claim_id'","agent_id":"'$AGENT_ID'","resource":"'$spec_path'","ttl_sec":1800}' >> .trinity/events/akashic-log.jsonl
```

### Heartbeat Protocol

Every agent with active claims must heartbeat every 60 seconds:

```bash
# Update heartbeat timestamp
if [[ -f ".trinity/claims/active/<spec_path>.json" ]]; then
    jq --arg now "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        '.heartbeat_at = $now' \
        ".trinity/claims/active/<spec_path>.json" > tmp.json
    mv tmp.json ".trinity/claims/active/<spec_path>.json"
fi
```

### Release Claim

```bash
# 1. Move claim to released
claim_spec_path="$1"
claim_id="$2"
result="$3"  # clean, toxic, cancelled

mkdir -p ".trinity/claims/released/"
mv ".trinity/claims/active/$claim_spec_path.json" \
   ".trinity/claims/released/${claim_id}.json"

# 2. Record release event
echo '{"event":"claim.release","claim_id":"'$claim_id'","agent_id":"'$AGENT_ID'","resource":"'$claim_spec_path'","result":"'$result'"}' >> .trinity/events/akashic-log.jsonl

# 3. Update queue
if [[ "$result" == "clean" ]]; then
    jq --arg task_id "$TASK_ID" \
        'del(.[] | select(.task_id == $task_id))' \
        ".trinity/queue/active.json" > tmp.json
    jq '. + {done: [.done[], {task_id: $task_id, completed_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)", result: "'$result'"}]}' \
        tmp.json > ".trinity/queue/active.json"
    rm tmp.json
fi
```

## Conflict Prevention

### Resource Ownership Rules

- One writable owner per resource (spec_path or graph_node)
- Other agents are read-only on claimed resources
- Claims have TTL; stale claims can be reclaimed after timeout

### Priority Rules

| Priority | Domain | TTL | Reclaim Wait |
|----------|--------|-----|---------------|
| P0 (Critical) | SacredPhysics, Graph, QueenLotus | 30 min | 5 min |
| P1 (High) | Numeric, Compiler | 60 min | 15 min |
| P2 (Normal) | Runtime, CLI | 120 min | 30 min |

### Phi-Critical and Sacred-Core

For `phi_critical` and `sacred_core` nodes:

- Stricter locking required
- Preferred verifier handoff
- Claims persist longer (no auto-reclaim)
- Manual approval for release

## Event Logging

All events are append-only to `.trinity/events/akashic-log.jsonl`:

```jsonl
{"ts":"2026-04-04T06:05:00Z","event":"task.intent","agent":"agent-spec-1","task_id":"NUMERIC-001","spec_path":"specs/numeric/gf16.t27","graph_node":"numericgf16","priority":"P0"}
{"ts":"2026-04-04T06:05:01Z","event":"claim.acquire","agent":"agent-spec-1","claim_id":"claim-001","resource":"specs/numeric/gf16.t27","ttl_sec":1800}
{"ts":"2026-04-04T06:05:03Z","event":"task.blocked","agent":"agent-runtime-1","task_id":"RUNTIME-004","blocked_by":"claim-001","resource":"specs/numeric/gf16.t27"}
{"ts":"2026-04-04T06:12:44Z","event":"claim.release","agent":"agent-spec-1","claim_id":"claim-001","result":"clean"}
```

## State Materialization

`.trinity/state/` contains derived current reality:

| File | Source | Refresh Policy |
|------|--------|----------------|
| `queen-health.json` | Derived from health signals + events | Every minute |
| `swarm-health.json` | Aggregated from all domains | Every minute |
| `ownership-index.json` | Active claims index | Every 30 seconds |

## Short Laws

**Before any task, every agent must read .trinity Akashic Chronicle, inspect active claims, queue, and swarm state, then acquire an exclusive claim on its target spec_path or graph_node. No mutation without prior read + claim.**

**No agent has rights to write to spec_path or graph_node without active claim.**

**One writable owner per resource; all others read-only. This ownership boundary is foundational production-practice for multi-agent coordination.**

**Claim has TTL and heartbeat; if agent dies, claim can be reclaimed after timeout.**

**For phi-critical and sacred-core nodes, stricter lock required; preferably verifier handoff.**

**Experience does not replace event log: events = immutable journal, experience = learned interpretation, state = derived current view.**

## Trinity Law

**.trinity is the canonical coordination source of truth:**
- **events** are immutable history,
- **claims** are temporary ownership,
- **experience** is learned memory,
- **state** is derived current reality.

This is the correct model: .trinity = event-sourced memory spine + coordination blackboard + shared state for the entire swarm.

---

## GPT-5.4 Micro-Laws (Extended Coordination Rules)

### Micro-Law #1: Event-First Enforcement

All agent actions MUST be preceded by appending an event to `.trinity/events/akashic-log.jsonl`. Events are the single source of truth; derived state (`.trinity/state/*`) is computed from events, not the other way around.

**Rationale**: If events and state diverge, events win. This prevents "zombie state" where agents believe one thing is true while events say another.

**Enforcement**: When any `.t27` spec is modified, `tri gen` MUST verify a corresponding `task.started` event exists. Without it, the mutation is rejected.

### Micro-Law #2: State-Is-Derived

`.trinity/state/*` files (queen-health.json, swarm-health.json, ownership-index.json) are ALWAYS derived from events, never primary. No agent may edit state directly; all state changes must come from event materialization.

**Rationale**: Derived state provides "current view" but is reconstructable. Events provide "immutable truth" for replay and forensic analysis.

**Enforcement**: Every 30 seconds, state files are regenerated by aggregating from event log. Direct edits to state files are rejected.

### Micro-Law #3: Stale-Lease Recovery

Claims have TTL (time-to-live). If a claim expires without being renewed via heartbeat, it becomes stale and MAY be reclaimed by another agent.

**Rationale**: Prevents "zombie claims" where agent dies while holding a resource. TTL + heartbeat enables automatic recovery.

**Enforcement**: Before acquiring a claim, agents must check if an expired claim exists. If expired, reclaim is allowed after wait period (P0 = 5min, P1 = 15min, P2 = 30min).

### Micro-Law #4: Lease Conflict Rule

If a resource is already claimed by another agent, the claiming agent MUST NOT proceed with mutation and MUST either:
- Wait for release
- Pick a different task
- Request formal handoff
- Enter blocked state

**Rationale**: One writable owner per resource prevents race conditions and data corruption.

**Enforcement**: Claim acquisition fails if `.trinity/claims/active/<resource>.json` exists and owner != current agent. Blocked tasks go to `.trinity/queue/blocked.json` with `blocked_by` field.

---

## Loop Handoff Protocol (FUTURE with 3 Options)

Each PHI LOOP must end with a `loop.handoff` event containing **three future options**. The next loop reads these options and chooses one.

### Format

**Output format:**

```text
[FUTURE OPTIONS]
  1) NUMERIC-STANDARD-001 — Complete GF6/GF12/GF20/GF24 + TF9 specs + conformance
  2) GRAPH — Add numeric nodes/edges to graphv2.json
  3) RUNTIME — Draft compilerparser.t27 structure
```

**Event format (akashic-log.jsonl):**

```json
{
  "ts": "2026-04-04T06:18:00Z",
  "event": "loop.handoff",
  "loop_id": "2026-04-04T06:18",
  "agent_id": "tri-main",
  "trace_id": "uuid",
  "past": {"summary": "migrated main.zig, created .trinity + tri skill"},
  "present": {"summary": "locked De-Zig Strict + SOUL Laws #6–#7"},
  "future_options": [
    {"id": "numeric-standard-001", "label": "Complete GF6/GF12/GF20/GF24 + TF9 specs + conformance", "priority": "P0", "domain": "Numeric"},
    {"id": "graph-numeric-extension", "label": "Add numeric nodes/edges to graphv2.json", "priority": "P1", "domain": "Graph"},
    {"id": "compilerparser-skeleton", "label": "Draft compilerparser.t27 structure", "priority": "P1", "domain": "Compiler"}
  ],
  "chosen_option": null
}
```

### Next Loop Protocol

When `/tri` starts a new loop:

1. Read last `loop.handoff` from `.trinity/events/akashic-log.jsonl`
2. Print:
   ```text
   [PAST]    <past.summary>
   [PRESENT] <what this loop will do>
   [FUTURE OPTIONS]
     1) <option 1>
     2) <option 2>
     3) <option 3>

   → Choosing option <N> by <rule> (e.g., P0 priority, domain balance)
   ```
3. Agent chooses ONE option by its rules:
   - **P0 Priority**: Always choose highest priority
   - **Domain Balance**: Choose least-worked domain (by queen_health)
   - **Round-Robin**: Rotate between domains
4. Update `loop.handoff` with `chosen_option` field
5. Proceed with chosen task

### No FUTURE = Toxic Completion

If a loop ends WITHOUT:
- `[FUTURE OPTIONS]` section
- `future_options` array in event
- At least 3 options

Then the loop is marked **drifted** and next loop must:
1. Read last successful handoff
2. Ask user for new 3-option plan
3. OR generate default 3 options based on pending tasks

This ensures every loop has reliable connection to next loop.
