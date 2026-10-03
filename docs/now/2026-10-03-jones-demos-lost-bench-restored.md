# NOW -- jones demos: lost bench restored, broken blocks readable (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `specs/demos/jones_topology_decision_gate.t27` discarded 62 tokens, `specs/demos/jones_topology_filter.t27` 16.
- In decision_gate, line 324 was a 51,780-character banner comment with `14    bench jones_signature_throughput` glued to its end: the bench header sat inside the comment, so the bench did not exist and its `measure`/`target` lines were discarded.
- Invariant `cosine_similarity_bounds` built its two structures as a six-line array literal; the clause parser counted it as discarded and `t27c gen` emitted a `NOT CHECKED -- body was not lowered (T43)` marker next to the check.
- Two bench `measure:` lines (`decision_gate_TH_02_throughput`, `signatures_match_computation`) were split over several lines; the continuations were discarded.

## What changed

- Line 324 is the section's banner again (same as its opening line 322), and `bench jones_signature_throughput` is on its own line. `t27c gen` now emits `fn bench_jones_signature_throughput`.
- `cosine_similarity_bounds` binds `s0 = standard_structure()`, `s1 = invert_structure(s0)` and asserts `sim >= -1.0 and sim <= 1.0` on `cosine_similarity(s0, s1)`: the same structures and the same assertion, with no `NOT CHECKED` marker.
- The two `measure:` lines are joined into one line each, text unchanged.
- The four seals were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH). Both `parse-no-discard` entries left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN.

## Not verified

- `t27c test-report` is BLOCKED on both specs on master and here alike (undeclared `vsa` in decision_gate, a parse error at generated line 211 in filter). `zig ast-check` errors are the same set before and after (6 and 1). Bench bodies are prose and lower to `NOT LOWERED` comments, as every other bench in these files.

Closes #5733
