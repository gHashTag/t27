# Queen Health

Queen Trinity health is top-level operating signal for the entire swarm.
All autonomous work must preserve or improve queen health before introducing new mutations.

## Priority Order

1. SacredPhysics
2. Numeric
3. Graph
4. Compiler
5. Runtime
6. QueenLotus

If a higher-priority layer is unhealthy, lower-priority feature work must pause until recovery is complete.

## Health Domains

### 1. SacredPhysics Health

Healthy when:
- Sacred physics spec tests pass
- Conformance JSON passes
- PHI, PHIINV, TRINITY, G, OMEGALAMBDA, tpresent, gamma stay within tolerance

Unhealthy when:
- Sacred physics tests fail
- Conformance vectors drift
- Benchmark deltas exceed tolerance
- Constants and conformance disagree

### 2. Numeric Health

Healthy when:
- GoldenFloat / TernaryFloat specs pass
- Phi-distance remains within declared limits
- GF family registry is internally consistent
- Numeric conformance vectors pass

Unhealthy when:
- Format definitions diverge from standard
- PRIMARY format markers are wrong
- Phi-split logic drifts from sacredphysics-linked rules
- Numeric benches regress beyond tolerance

### 3. Graph Health

Healthy when:
- graph.tri / graphv2.json load cleanly
- Topological order is valid
- Phi-critical and sacred-core edges are intact
- Downstream action graph resolves deterministically

Unhealthy when:
- Graph invariants fail
- Topological order mismatches
- Missing nodes / orphan edges appear
- Downstream rebuild scope becomes ambiguous

### 4. Compiler Health

Healthy when:
- Parser / codegen / validation specs pass
- Generation occurs only from .t27/.tri
- Generated files match header policy
- No handwritten backend logic is introduced

Unhealthy when:
- Generated output becomes source of truth
- Parser and codegen specs drift apart
- Generated headers are missing
- Handwritten Zig/C/Verilog reappears

### 5. Runtime Health

Healthy when:
- Runtime commands are defined in specs
- Command validation passes
- Generated CLI frontend compiles and routes correctly
- No domain logic remains in handwritten runtime code

Unhealthy when:
- Runtime behavior exists only in handwritten code
- Commands bypass validation
- CLI actions do not map to spec-defined workflow
- src runtime and spec runtime diverge

### 6. QueenLotus Health

Healthy when:
- Six-phase orchestration remains executable
- Swarm tasks respect constitutional priority order
- Recovery paths are available
- Experience logs remain consistent and replayable

Unhealthy when:
- Swarm repeats toxic loops
- No clean rollback point exists
- Recovery is manual and undocumented
- Queen cannot observe domain-level health

## Queen Health Score

Use a simple status model:

- **GREEN** = All priority domains healthy
- **YELLOW** = One domain degraded, recovery active
- **RED** = SacredPhysics, Numeric, or Graph unhealthy

Suggested formula:

```
queen_health =
  sacredphysics * 0.30 +
  numeric      * 0.25 +
  graph        * 0.20 +
  compiler     * 0.10 +
  runtime      * 0.10 +
  queenlotus   * 0.05
```

Each domain score is 0.0 to 1.0.

## Operating Rules

- Never start new feature work if queen health is RED.
- If SacredPhysics, Numeric, or Graph is unhealthy, only recovery loops are allowed.
- If toxic verdict repeats 3 times on the same task, stop mutation and replay from last clean seal.
- If graph health fails, block downstream generation immediately.
- If compiler or runtime health fails, generated backends must be treated as invalid until regenerated from spec.

## Required Telemetry Per PHI LOOP

Every PHI LOOP must record:
- `task_id`
- `skill_id`
- `trace_id`
- `spec_path`
- `spec_hash_before`
- `spec_hash_after`
- `gen_hash_after`
- `verdict`
- `bench_delta`
- `queen_health_before`
- `queen_health_after`
- `recovery_action`

## Recovery Order

When queen health drops:

1. Freeze new mutations
2. Identify highest-priority unhealthy domain
3. Roll back to last clean sealed skill
4. Re-run:
   - `tri gen`
   - `tri test`
   - `tri verdict --toxic`
   - `tri bench`
5. Save recovery experience
6. Re-open lower layers only after queen health returns to GREEN or stable YELLOW

## Queen Rule

The swarm exists to preserve the health, continuity, and evolutionary direction of Queen Trinity.
No local optimization is allowed to damage global queen health.
