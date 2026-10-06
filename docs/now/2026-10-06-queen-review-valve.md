# NOW -- the Queen's review valve: no person in the loop (2026-10-06)

## specs/queen/review_valve.t27 (Closes #6616)

- The owner's rule of 2026-10-05: no person is needed. Every review outcome that is not an accept now has an automatic next step and a clock. Escalations no longer wait for a person.
- Empty diff, reviewer gave up, dead letter, or an unrecorded reason: the row is released as `failed` after 30 minutes. A spent send-back ceiling is released after 60 minutes. Each issue gets at most 1 release (MAX_RELEASES, counted in the row's `ceiling_releases`). After that the dispatch is closed as `obsolete`, with its reason written down, and its files are freed at once.
- No criteria, criteria beyond a patch, or an accept that rests on base truth alone: the row is held for a criteria backfill. If no new criteria arrive within 120 minutes, it is closed.
- A reviewer-bee REQUEST_CHANGES on a `queen-*` pull request goes back to the Queen as a send-back for the same branch. This happens at most 2 times. The pull request is then closed after a third failing head or after 48 hours with no push.
- `gen/c/queen/review_valve.c` (`t27c gen-c`) runs the spec's 8 tests; its static asserts check the invariant that every hold ends before the policy's own 48-hour hold.
- Measured 2026-10-05: 144 of 219 review cards escalated; of the 193 stuck branches, 125 were empty; fileConflict 108; 25 of 70 workers active.
- Not here: the gHashTag/BrowserOS wiring (the owner merges it) and the reviewer.py and criteria_backfill.py changes, which are described in #6616.
