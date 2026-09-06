# NOW -- A refused dispatch is not a missing one (2026-09-06)

## A refused dispatch is not a missing one (Closes #3355)

- `tri gates unmeasured` could not tell a workflow MISSING a `workflow_dispatch:` from one that REFUSED one, so it printed `dispatch: NO` beside the advice to add one -- advising the next reader to undo #3325 and put a dispatch back in front of `cargo publish` on a live registry.
- `has_dispatch` returns three states now; a workflow records a deliberate refusal with `# tri:no-dispatch`, where the tool looks rather than where a human reads. `release.yml` carries it.
- A present dispatch still wins over the marker, so a stale comment cannot hide a real one.
