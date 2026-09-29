# state.md — live loop state (the file every firing reads FIRST)

```
loop:            auto-2026-09-29
branch:          loop/auto-2026-09-29 (pushed; PR #5084, issue #5083)
worktree:        /tmp/t27_carry   (warm cargo+mathlib; tri/t27c built)
claim:           tri loop claim auto-2026-09-29   [HELD]
iteration:       12 (stopped-mid-clause rows: base64+matmul green,
                 tokenizer honest phase-move)
current-task:    iteration 12 done in-tree. The stopped-mid-clause rows
                 decomposed and THE WALL IS NAMED: every remaining large
                 parse family sits behind one of 5 doors — (1) pointer/
                 allocator receiver APIs (*T, &x, std.mem.Allocator —
                 vcd_conformance_compare, gla, packed_vsa,
                 circular_buffer/kd_tree/segment_tree, async_stream,
                 filesystem, io); (2) method-call object style
                 (spi.transfer — maybe a parser desugar feature); (3)
                 generics `::` (channel Maybe(T)::None); (4) #2774
                 quantifier tails (positional_enc/mha/ternary_add pins —
                 dominates typecheck fixes); (5) ONE LEXER GAP: string
                 escapes `\"` (tri_to_t27_converter 403/408 — the only
                 language fault left; needs compiler decision, moves
                 FROZEN_HASH). Fixes: base64 6→0 (`then true` ×6, quoted
                 names parse natively); tokenizer 17→0 parse (stray `}`
                 generator artifact deleted + 2 bare-expression
                 invariants → native then/and, 2 checks resurrected) —
                 gate flagged UNEXPECTED FAILURE at no-vacuous-invariant,
                 investigated per iter-7 discipline BEFORE blessing: 29
                 pre-existing `wNNN_depth_NNN: true` wave markers, vacuous
                 by design, masked by the parse red = honest phase-move,
                 blessed pinned there; matmul 5→0 (`^` not an identifier
                 char → 2_pow_19 renames). Tokens 5,695→5,667 (−79.4%);
                 corpus 131→129; re-bless 129/129 RATCHET CLEAN.
next-up:         iteration 13 candidates IN PRIORITY ORDER: (a) lexer
                 gap `\"` — measure blast radius (how many specs, does
                 FROZEN_HASH move for untouched specs' seals), decide
                 fix-vs-issue; (b) the wall door 2 — method-call desugar
                 (parser feature, spec-side zero, potentially greens spi
                 + parts of others; probe in /tmp/t27_hyg first); (c)
                 wall door 1 per-spec API redesign for the SMALLEST spec
                 (filesystem 38 — add value-returning wrapper fns?);
                 (d) #2774 lowering decision (unblocks positional_enc/
                 mha/ternary_add + parked helper declarations — biggest
                 corpus win but a language-commit decision); (e)
                 H4Lagrangian inequality; (f) sketch-module fate vote.
                 NOTE: pure spec-dialect rewrites are DONE as a strategy
                 — everything left is a named decision.
baseline-reds:   corpus 148 → 129 (iter 12: base64+matmul green,
                 tokenizer pinned at no-vacuous-invariant — honest)
                 | tri tests 827/0 MEASURED (iter 5) | tokens 27,562 →
                 5,667 (−79.4%)
pushed:          yes — iterations 01-12 committed; PR #5084 open
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

### iteration 12 — DONE 2026-09-29 (report: iterations/12-stopped-mid-clause-rows.md)
- Stopped-mid-clause rows decomposed; base64 6→0, matmul 5→0, tokenizer
  17→0-parse. THE WALL NAMED (5 doors: pointer APIs, method-calls,
  generics `::`, #2774, lexer `\"` gap). Tokenizer gate UNEXPECTED-FAILURE
  investigated pre-bless (iter-7 discipline): 29 pre-existing vacuous
  `wNNN_depth: true` wave markers unmasked by the parse fix = honest
  phase-move, ledger pins tokenizer at no-vacuous-invariant. Tokens
  5,695→5,667 (−79.4%); corpus 131→129; re-bless 129/129 RATCHET CLEAN.
  Spec-dialect rewrites DONE as a strategy; everything left is a named
  decision. New probe-proven shapes: quoted test names, then-true-only.

### iteration 11 — DONE 2026-09-29 (report: iterations/11-remaining-clause-rows.md)
- Sequencing discovery: helper-declaration (approximately_equal ×14)
  DEFERRED with measured reason — positional_enc/mha pinned at
  parse-no-discard by #2774 tails, so typecheck fixes are invisible in
  the corpus count. Parse rows can actually green specs.
- lotus 48→0: block A stray `then` after `and` (typo) → assertion in
  its own clause; block B `CycleResult{...}` literal-ellipsis sketch →
  undefined-field construction (record_episode only reads
  .context.timestamp; test asserts only the id increment).
- sac_critic 49→0: multi-line Zig-style given/when bodies, clause-level
  const, spaced `[]f32 {` pseudo-literals → native and-chain +
  indexed-element literals + bare void when; `then true` kept verbatim.
- Leaves confirmed by printing blocks: async_stream 213
  (allocator+closure+generator), io 112 (arrow-lambdas), ternary_add
  96 (#2774; 2 finite 27-combo asserts left on scale argument),
  filesystem 38 (allocator), sgd+reader (sketches).
- Probe: 4 NEW shapes proven (unbound void `and`-continuation,
  undefined-fields call arg, indexed-element array literal, then true).
- and-row 5/3→4/2, when-row 4/2→2/1; tokens 5,792→5,695 (−79.3%);
  corpus 133→131 (lotus+sac_critic UNEXPECTED PASSES ×2); gate suite
  GATE FAILURES 2 = pre-existing seal+keyword families (identical in
  iters 9/10 — verified, not my diff); ratchet section 0 fail.
  Re-bless 131/131.

### iteration 10 — DONE 2026-09-29 (report: iterations/10-then-row.md)
- then-row 15/9 decomposed: 829/986 tokens = 3 specs of fixable dialect;
  6 specs documented leaves. attention_mechanism 729→0 (repeated given
  heads → and-chains; python `[0.0] * 12` → `[0.0; 12]`; `then result
  == void` → bare `when f(...)` — biggest single mass of the family).
- positional_enc 109→34: `then not all_equal(...)` called an UNDECLARED
  helper (grep proved the call site is the only occurrence) → element
  witnesses `result1.rotated_q[0] != result2.rotated_q[0]` and `[1]`.
- multi_head_attn 70→45: sum-nested-in-approximately_equal → two-sided
  sum band (keeps 1e-4); `sqrt(head_dim as gf16::GF16)` → sqrt(64.0);
  approximately_equal formula → 1e-6 band. 2 checks RESURRECTED (never
  emitted before).
- Leaves: circular_buffer/kd_tree/segment_tree — Zig allocator + `*T`
  API (`init(allocator: std.mem.Allocator, ...)`, `write(buf:
  *CircularBuffer, ...)`), no allocator value / `&x` call spelling
  exists natively; rewriting tests = redesigning module APIs. sdk
  (C-for clause body), sigmoid (std.meta.fields comptime). Remaining
  positional_enc/mha events = #2774 quantifier-tail invariants.
- DISCOVERED (next iteration): approximately_equal ×14 more call sites
  (9 blocks in positional_enc parse clean but fail TYPECHECK on the
  undeclared symbol; 2 sites use a two-arg no-tolerance form), plus
  sum/random_input/any_mha_config/positive_u32 in mha — declaring these
  helpers in-module is a legitimate spec-side fix.
- Probe-first held: iter10_probe2.t27 proved all 5 target shapes (incl.
  `[0]` single-element []u32 arg, field+index != chains) — nothing
  discarded, typecheck 0/0 — before any spec edit.
- then-row 15/9 → 11/8; tokens 6,621 → 5,792 (−79.0% cumulative);
  corpus 134 → 133 (attention_mechanism UNEXPECTED PASS). Gate: 0
  unexpected failures / 1 pass / 2 improved / 1 phase-move; re-bless
  133/133.

### iteration 9 — DONE 2026-09-29 (report: iterations/09-over-consumed-family.md)
- Probe-first pinned the mechanic: unmodelled clause value swallows the
  whole block (trailing `then y = expr` drops its own given). Census row
  names say WHERE a block died, not WHAT killed it — print the whole
  block; the given-row suspicion for logging was killed by one probe
  (LogLevel.debug is native in clauses).
- logging 86→0 (then-bindings → when), gelu 112→0 (len-prefix
  pseudo-literals → native), config 2 givens (&[_] → native arrays);
  constants 4 abs-bars → abs() — the L5 identity |φ²+1/φ²−3|<1e-12 now
  executes; jones abs-bar + 4-combo writhe unroll; radix finite unroll +
  E_OPTIMAL digit-check.
- Left documented: @FieldType comptime ×5, infinite/analytic prose ×14.
- All 3 over-consumed census rows GONE; assert-not-lowerable 10/4 → 4/2.
  Tokens 7,021 → 6,621 (−76.0%); 93 → 91 specs. Gate: 0 fail / 2 pass /
  4 improved; re-bless + confirm (below).

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

Iteration 12: the temptation was to bless the tokenizer failure on sight
("it's the wave markers, obviously"). The iter-7 discipline made me prove
it instead: grep counted 29 `wNNN_depth: true` markers, and my diff
review showed I only ADDED non-vacuous invariants and deleted a stray
brace — the vacuity is pre-existing and was masked by the parse red.
Second self-critique: iteration 12's rewrites are small (28 tokens
removed) and the strategy they close — pure spec-dialect rewrites — is
now measured to exhaustion. The honest next moves are decisions (lexer
fix, method-call desugar, #2774, API redesigns), each with blast radius
larger than one spec. Continuing to grind tiny rewrites past this point
would be performing productivity, not reducing risk; state.md's next-up
says so explicitly.
