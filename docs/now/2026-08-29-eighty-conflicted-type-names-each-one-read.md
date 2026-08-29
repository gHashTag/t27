# NOW -- Eighty conflicted type names, each one read (2026-08-29)

## Eighty conflicted type names, each one read (Refs #2774)

- `tri types dup` counts conflicted names; it cannot say what KIND of conflict
  each one is. All 80 were opened and split: 46 DRIFT (one concept that grew a
  second definition) and 34 DISTINCT (two concepts that collided on a name).
  The two want opposite repairs, so the count alone was not actionable.
- `docs/TYPE_CONFLICTS.md` is the summary;
  `docs/reports/type_conflicts_classified.json` carries the per-name reading
  that decided each verdict.
- One row is fixable today with no cross-module decision: `AdamWConfig` has
  both definitions in ONE file, `specs/ml/optimizer/adamw.t27` lines 28 and 483,
  six of seven fields identical, and the file's own comment says the second was
  appended.
- New command `tri types classified` cross-checks the document against a live
  reading and fails in BOTH directions -- UNJUDGED for a conflict nobody has
  read, STALE for a row about a name that is no longer conflicting. Wired into
  corpus-ratchet.yml. Proven red both ways before landing, then restored.
- It earned itself on the first execution: `HealthStatus`, the eightieth name,
  appeared when #2802 taught the field reader that `pub name: T` is a field,
  and the classification predated that change.
- Recorded honestly: four verdicts (`Agent`, `AgentStatus`, `Color`,
  `HealthStatus`) are reported CONFLICTED by the tool only because it cannot
  parse the `variants : ,` enum idiom on one side. The verdicts came from
  reading the source; the tool's agreement is a coincidence, not corroboration.
