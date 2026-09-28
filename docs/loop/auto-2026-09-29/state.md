# state.md — live loop state (the file every firing reads FIRST)

```
loop:            auto-2026-09-29
branch:          loop/auto-2026-09-29 (pushed; PR #5084, issue #5083)
worktree:        /tmp/t27_carry   (warm cargo+mathlib; tri/t27c built)
claim:           tri loop claim auto-2026-09-29   [HELD]
iteration:       3 (in flight)
current-task:    L2 #5079 verilog_bench_harness.t27 deliberate rewrite
next-up:         second forall site (parse_bdd_clauses, 256 events/29 specs);
                 then tri CLI additions (task #8)
baseline-reds:   corpus 148 | parse 23 | no-discard 114 | vacuous 65 |
                 typecheck 26 specs | seal-verify ~573 (baseline.md + baseline/)
pushed:          yes — iteration 01 committed + PR #5084 open
pr:              #5084 (Closes #5083) — add loop commits to this PR, do NOT
                 open new PRs per iteration
blocked-user:    merges (#5078/#5081), lean yml gate #5082 (workflows scope)
last-suite:      /tmp/loop_suite_i1.{log,json} (baseline run)
```

## Iteration protocol (every firing)

1. Read this file. 2. `git -C /tmp/t27_carry status` — dirty tree from a
crashed iteration: inspect, commit or revert (only files this loop created).
3. First open item in `plan.md`. 4. Verify reds did not grow vs `baseline.md`
(diff command there). 5. Update this file + push + append self-critique.

## Iteration log

### iteration 3 — started 2026-09-29 ~01:50
- Target: #5079 truncated `specs/test_framework/verilog_bench_harness.t27`
  (Rust-ism cascade: format! ×8, `!` prefix, if-expression fields, `...`
  spread, orphaned tail lines 405–460 OUTSIDE the module close at 404).
  Protocol update after iteration 2: parse-check the touched spec
  IMMEDIATELY after each parser/spec edit (`t27c parse <file>`), full suite
  only at iteration end.

### iteration 2 — DONE 2026-09-29 ~01:50 (report: iterations/02-forall-preservation.md)
- forall preservation arm in parse_invariant_clause + capture_to_next_top_level
  (boundary BEFORE depth adjust — v1 had it after and broke ternary_inference).
- −13,198 silently-discarded tokens corpus-wide (27,562→14,364); ternary_inference
  1813→87; corpus reds 148→148 (volume win, not count win); zero new reds.
- Ledger re-blessed: discard sum pinned 23,831→14,364, max_entries 152→148,
  RATCHET CLEAN. ANOMALY resolved: master's ledger had drifted (52 stale
  entries removed, 47 never-tracked failing specs added — all verified
  already-failing at baseline; none caused by this loop).

### iteration 1 — DONE 2026-09-29 ~01:10 (report: iterations/01-*)
- Charter+claim+branch; baseline measured (the silent-discard reframe);
  competitor scan; docs/README.md map line; issue #5083; PR #5084; pushed.

## Self-critique of the latest completed iteration

Iteration 1: baseline ran once (determinism asserted, not yet proven);
competitor scan 3 queries deep; suite ran in a warm worktree (rerun on a
clean clone would prove independence). All named in iterations/01 report.
