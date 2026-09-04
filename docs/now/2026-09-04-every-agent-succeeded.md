# NOW -- Every agent succeeded and the aggregate was twenty times wrong (2026-09-04)

## The failure was in the script, and no agent counter covers the script

- A fan-out over 81 identifiers returned `{"judged": 4, ... "confirmed": 0}`. The journal held
  **81 judged, 56 of them `ABSENT_AND_CLAIMED`** -- the aggregate under-reported by **20.2x** and
  inverted the shape, printing 0% accusatory where the real tally is 69%.
- Cause: the second pipeline stage was written `(res) => {...}` while referencing a free `b`.
  Stage callbacks receive `(prevResult, originalItem, index)`. It threw `b is not defined` for 20
  of 21 items, and a stage that throws drops its item to `null` and skips the rest of its chain.
- **Every health signal was green and every one was correct**: `agents_done: 21`,
  `agents_error: 0`, `agents_empty_result: 0`. All 21 agents ran and returned non-empty. The
  agents were fine; the script was not, and the agent tallies do not cover the script. The only
  surface that showed it was the `failures` block beside the result.
- **A pipeline's returned number is not a measurement until the failures block is empty.**
- Nothing was lost: `journal.jsonl` carries every agent's full return value, so all 81 verdicts
  were recovered without re-running one agent. The callback was fixed and the run resumed, which
  replayed the judges from cache and ran only the repaired stage.

A small returned number and a small real population look identical in the result object, and are
trivially distinguishable one file over.

Refs #3140
