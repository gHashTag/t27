# state.md — live loop state (the file every firing reads FIRST)

```
loop:            auto-2026-09-29
branch:          loop/auto-2026-09-29 (pushed; PR #5084, issue #5083)
worktree:        /tmp/t27_carry   (warm cargo+mathlib; tri/t27c built)
claim:           tri loop claim auto-2026-09-29   [HELD]
iteration:       8 (given not-lowerable family — six rewrites, two leaves)
current-task:    iteration 8 done in-tree. The census given-family (19 events
                 / 8 specs) decomposed into 5 root constructs; 6 specs
                 rewritten natively (attention 417→0, hslm 153→0, packed_trit
                 74→0, mime 44→0 + type unquoted, html 37→0, xml 43→0 —
                 `undefined` for the unconstructible map field per the
                 corpus's own mime idiom), 2 left deliberately (reader, sgd —
                 whole-module sketches: generics / string fn-types / runtime
                 heap allocs; next loop decides their fate explicitly).
                 attention now emits 48 real checks. Gate: 0 unexpected
                 failures, 6 unexpected passes, re-blessed 136/136, RATCHET
                 CLEAN.
next-up:         iteration 9 candidates: (a) "clause value over-consumed"
                 family — assert 6/3, given 6/2 (gelu_approx, config,
                 constants, radix_economy, jones_polynomial), then 4/1
                 (logging); (b) not-lowerable then-row 15/9; (c) H4Lagrangian
                 failing analytic inequality (iterations/05); (d) boot-path
                 synthetic operating point (ResolvedPvtContext).
baseline-reds:   corpus 148 → 136 (iter 8: 6 specs fully green) | tri tests
                 827/0 MEASURED (iter 5) | tokens 27,562 → 7,021 (−74.5%)
pushed:          yes — iterations 01-06 committed; PR #5084 open
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

### iteration 8 — DONE 2026-09-29 (report: iterations/08-not-lowerable-family.md)
- Census given-family decomposed FIRST (per state.md instruction): 5 root
  constructs behind 19 events / 8 specs. 644 of 986 family tokens = one
  construct: Zig-spelled struct values in clauses. The language already
  owns the native spelling → spec-side rewrites (iteration-3 class), no
  compiler change.
- 6 specs green: attention (48 checks now emit), hslm, packed_trit,
  mime (type `"[]const []const u8"` unquoted → `[][]const u8`; recipient
  data survives in emitted Zig), html, xml (`undefined` for the map field
  — corpus's own idiom; no native map type exists, msgpack carries the
  same quoted field and stays green).
- 2 left documented: reader (generics, string fn-types, closures), sgd
  (runtime heap allocs, undeclared locals) — whole-module sketches; their
  fallback measures the corpus containing sketches, not a parser gap.
- Tokens 7,789 → 7,021 (−74.5% cumulative); events 108 → 92; specs 99 →
  93; given-row 19/8 → 5/2 (the two leaves). Corpus 142 → 136. Gate: 0
  unexpected failures / 6 passes → re-bless 136/136 RATCHET CLEAN.

### iteration 7 — DONE 2026-09-29 (report: iterations/07-closure-loop-rewrite.md)
- Part 1: 3 closure-loop events = Zig dialect in the specs (`for (const i)
  |reg| in [...] {` — W699 rung 9 needs a preceding clause; parse_for_stmt
  rejects the hybrid). Rewritten natively (given/and/assert unrolls) —
  10 runtime checks resurrected; census `for` row GONE; tokens 7,914→7,789
  (−71.7% cumulative); specs 102→99.
- Part 2 (unmasked by part 1): module symbol-collection ignored
  `extra_mutable` → every module-level var array typed const ROM → W456
  hard-error on element assignment (13 corpus sites). One-arm fix;
  bridge 6→0, matmul_serial_hw, disjoint_set 2→0 errors. FROZEN_HASH →
  586cb23881194ea4.
- Gate: 0 unexpected failures / 5 unexpected passes (3 rewrite specs +
  bridge + matmul_serial_hw — ALL fully green; corpus 147→142). Re-blessed.

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

Iteration 8: the decomposition-first instruction paid off — "clause not
lowerable" was FIVE root constructs, and patching the walker for the row's
most common shape would have extended the parser toward Zig struct-literal
syntax (the opposite of right: the language already owns the spelling).
Weaknesses: `undefined` as a field value keeps the map field
unconstructible (a real native map type would be better; noted in the
specs); reader/sgd stay red at ~195 tokens combined because nobody has
decided whether sketch modules belong in the corpus — next loop must
decide explicitly, not let them become permanent residents; and the mime
type-unquote touched a DECLARED field type for the first time this loop
(justified — the quoted form never checked anything; `use`-scan verified
no importers).
