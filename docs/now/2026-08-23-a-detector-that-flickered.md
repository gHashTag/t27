# NOW — a detector keyed on a value recomputed while you look at it (2026-08-23)

`tri gates prs` flagged three pull requests one iteration ago. Run again today, it flagged **one** — and nothing about the other two had changed.

- `mergeable` is computed on demand. Between two runs, two pull requests moved from CONFLICTING to UNKNOWN and back, and the detector — testing `m == "CONFLICTING"` — lost them and found them again. **The alarm was intermittent for a condition that was not.**
- **The observable is the short check list.** Three checks against a median of twenty-one is the finding, whatever GitHub currently believes about mergeability. The state is the *explanation*, and belongs in the row rather than the test.
- The reference had the same defect one layer down: the median of the *non-conflicting* rows read 21 on one run and 35 on the next, because two rows crossed the filter in between. A median over **every** row is unmoved by a few short lists. Two consecutive runs now agree.
- **And `UNKNOWN` is not `fine`** — it means GitHub has not finished computing. A short list with UNKNOWN beside it is the same finding as one with CONFLICTING, seen a moment earlier.

**The rule:** before keying an alarm on a field, ask whether the field is *measured* or *computed on demand*. A derived, cached, or lazily-evaluated value makes a detector that reports the weather rather than the climate — and the first symptom is a finding that comes and goes without anything changing.

Filed for the owner: three pull requests have run almost no CI, and two of them change `bootstrap/src/compiler.rs` — the exact file whose gate carries the comment about a C-emitter rewrite merging with the cross-target proof never running.
