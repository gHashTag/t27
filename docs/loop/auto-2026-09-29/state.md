# state.md — live loop state (the file every firing reads FIRST)

```
loop:            auto-2026-09-29
branch:          loop/auto-2026-09-29 (pushed; PR #5084, issue #5083)
worktree:        /tmp/t27_carry   (warm cargo+mathlib; tri/t27c built)
claim:           tri loop claim auto-2026-09-29   [HELD]
iteration:       5 (open)
current-task:    close-out (task #9): commit site 2 + seal-drift doc, push,
                 verify tri full test, then report + 3 variants + skills/memory
next-up:         plan items left: none open except close-out; new census top
                 row (for mid-clause, 49/12) is next loop's first candidate
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

### iteration 4 — DONE 2026-09-29 (report: iterations/04-forall-site-2.md)
- `tri loop state` (88e3d4e05, pushed earlier this iteration) + forall site 2
  in parse_bdd_clauses: LED-shape forall preserved via same walker as site 1.
  Census row 256 events/29 specs ELIMINATED; discarded 14,364→8,559 (cumulative
  −69% from 27,562); corpus 147 (volume win); gate rc=0; ledger re-blessed
  147/147, no self-inflicted adds. New census top row: for mid-clause 49/12.
- Plan items 6/7 premises measured FALSE (recorded, no fake fixes).

### iteration 3 — DONE 2026-09-29 ~02:20 (report: iterations/03-verilog-bench-5079.md)
- #5079 CLOSED by deliberate rewrite: format! ×8 (3→concat, 3→constants with
  comments), T? ×2 → ?T. Everything else already parsed. AST verified whole.
  Corpus 148→147; ledger retired the entry, 147/147 cap, RATCHET CLEAN.

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

Iteration 4: mid-block forall shape not re-measured after the fix (claim
inherited from site-1 analysis); −69% is tokens, not specs; `tri loop
state` detached-HEAD path untested. All named in iterations/04 report.
