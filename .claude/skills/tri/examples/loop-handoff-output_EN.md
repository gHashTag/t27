# Loop Handoff Output — for /tri CLI

This block is output at the end of each PHI LOOP lap and recorded as `loop.handoff` event in `.trinity/events/akashic-log.jsonl`.

## Output Format

```
═════════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═════════════════════════════════════════════════════════╝
```

## Fields

[PAST]   <completed task/skill IDs from previous loop>
          <summary: one-line description>

[PRESENT] <completed task/skill IDs from current loop>
          <summary: one-line description>

[FUTURE] <planned task/skill IDs for next loop>
          <summary: one-line description>
          <drifted: true|false>  # only if plan changed
          <drift_reason: why plan changed>

─────────────────────────────────────────────────────────────
Last handoff recorded at: <timestamp>
Loop session ID: <loop-uuid>

PHI LOOP completed
```

## When to Output

1. **At the end of a successful PHI LOOP** — after `tri skill commit` or `tri git commit`

2. **At loop pause** — when `/tri loop pause` or SIGTERM received

3. **At loop stop** — when `/tri loop stop` received

## Rules

1. **[PAST]** Always shows what was done in previous loop. If previous loop was interrupted, shows partial.

2. **[PRESENT]** Always shows what was done in current loop. If loop paused, shows partial.

3. **[FUTURE]** Always shows what was planned for next loop. If loop paused, shows planned.

4. **If [FUTURE] differs from what was planned**:
   - Add `<drifted: true>`
   - Add `<drift_reason: why plan changed>`
   - Status changes to "drifted"

5. **When loop paused (interrupted)**:
   - [PRESENT] shows only up to interruption
   - [FUTURE] remains as was planned
   - No drift status

6. **When loop stopped**:
   - [PRESENT] shows all completed
   - [FUTURE] remains as was planned

7. **When loop resumes**:
   - Add new [PRESENT] entries
   - Keep [FUTURE] as planned

8. **At next loop start**:
   - Move [FUTURE] to [PRESENT] (if it was ready)
   - If it wasn't ready, keep in [FUTURE]

## Summary Table

| Status | Purpose | Example |
|--------|-----------|---------|
| [PAST] | Track previous loop work | `NUMERIC-001, RUNTIME-004` |
| [PRESENT] | Track current loop work | `NUMERIC-002, CHIMERA-003` |
| [FUTURE] | Plan next loop work | `BASE-001, GF-FORMAT-005` |

## Example Handoff

```
╔═══════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═════════════════════════════════════════════════════════╝
```

[PAST]   NUMERIC-001, RUNTIME-004
          <summary: Fixed GF8 exponent bias and added CLI routing>

[PRESENT] NUMERIC-002, CHIMERA-003
          <summary: Created GF16 format specification>

[FUTURE] BASE-001, BASE-002
          <summary: Add Trit types and operations>
          <drifted: false>

─────────────────────────────────────────────────────────────
Last handoff recorded at: 2026-04-04T12:30:00Z
Loop session ID: 550e8400-1234-4b5a-9c6d-7e8f9a0b1c2d3e4

PHI LOOP completed
```
