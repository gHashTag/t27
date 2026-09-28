# state.md — live loop state (the file every firing reads FIRST)

```
loop:            auto-2026-09-29
branch:          loop/auto-2026-09-29 (pushed; PR #5084, issue #5083)
worktree:        /tmp/t27_carry   (warm cargo+mathlib; tri/t27c built)
claim:           tri loop claim auto-2026-09-29   [HELD]
iteration:       6 (quantifier-tail preservation, site 3 — landing)
current-task:    iteration 6 done in-tree: assert `for all ...` tails preserved
                 verbatim (site-1 discipline, NOT lowered — first version that
                 lowered them emitted checks on free vars, W635-inverted,
                 caught before commit). Census for-row 49→3 (remainder =
                 closure loops, a different defect); tokens 8,559→7,914
                 (cumulative −71.3%); 5 specs (sacred_physics, gf12/20/24/32)
                 discard ZERO now — ratchet flags them "fixed, remove from
                 ledger"; re-bless running. New CLI: parse-complete --specs.
next-up:         iteration 7 candidates: (a) closure-loop shape — `for (const
                 i) |reg| in [...] {` at clause position (3 events: registers,
                 ternary_memory, ops); (b) H4Lagrangian failing analytic
                 inequality (iterations/05); (c) boot-path synthetic operating
                 point (ResolvedPvtContext).
baseline-reds:   corpus 148 | parse 23 | no-discard 114 | vacuous 65 |
                 typecheck 26 specs | seal-verify ~573 (baseline.md + baseline/)
                 tri tests: 824/3 → 827/0 MEASURED (iter 5)
pushed:          yes — iterations 01-05 committed; PR #5084 open
pr:              #5084 (Closes #5083) — add loop commits to this PR, do NOT
                 open new PRs per iteration
blocked-user:    merges (#5078/#5081), lean yml gate #5082 (workflows scope)
last-suite:      full cargo test -p tri (release): 827 passed / 0 failed,
                 932s, /tmp/loop logs — the August trio is closed
```

## Iteration protocol (every firing)

1. Read this file. 2. `git -C /tmp/t27_carry status` — dirty tree from a
crashed iteration: inspect, commit or revert (only files this loop created).
3. First open item in `plan.md`. 4. Verify reds did not grow vs `baseline.md`
(diff command there). 5. Update this file + push + append self-critique.

## Iteration log

### iteration 6 — DONE 2026-09-29 (report: iterations/06-quantifier-tail-preservation.md)
- Census top row decomposed: 45/49 events = English quantifier tails on
  asserts (`assert f(x) for all x`), 3 = closure loops (separate defect,
  left as the row's remainder). Fix = site-1 discipline (preserve verbatim
  into block.value, partial mark, children stay empty, `true` keeps
  `lowered` counting so no re-fallback). First version LOWERED the assert
  with tail in value — emitted checks on FREE vars (W635 inverted) — caught
  by reading what the emitter does, before commit.
- Tokens 8,559→7,914 (cumulative −71.3% from 27,562); discarding specs
  107→102; one resurrected clean check in constants.t27 (mixed block:
  pow(0.0, 0.0) == 1.0 was collateral of a quantified sibling's fallback).
- New CLI: `t27c parse-complete --fallbacks --specs` (names behind the
  counts; the BTreeSet was already collected).
- FROZEN_HASH d925e634aa4cfbca → d7192007e47f6365.

### iteration 5 — DONE 2026-09-29 (report: iterations/05-fpga-lean-restoration.md)
- The August 824/3 tri red trio root-caused as ONE defect in four layers:
  W472 pseudo-Lean (#4765) + merge-dropped validate_lean_standalone phase +
  ungated master lean CI + d51db4ac1's partial restoration masking a dropped
  cclk_sweep parameter. All restored: phase body, synthetic-JSON sweep check,
  18th cclk_sweep param + CLI flag, W472 Lean block rewritten (0 warn/0 sorry,
  2 false lemmas corrected not papered over). 4/4 lean_standalone tests green.
- Corpus untouched; no compiler change (fpga.rs is cli/, not bootstrap).

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

Iteration 5: restored the pinned shapes but did not audit the other five W450
dry_run_sweep emitters (already-failing paths, unpinned); kept today's 5-param
resolve_pvt_context_for_boot (boot path still cannot take a synthetic
operating point — deliberate scope cut, no test pins it); H4Lagrangian left
red with its failing goal named. Full-suite number recorded in this file only
when the run finishes — the report cites this file, not a prediction.
