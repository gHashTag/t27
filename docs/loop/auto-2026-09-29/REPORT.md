# Loop auto-2026-09-29 — final report

One pass, eleven iterations, every number below measured on this tree
(`/tmp/t27_carry`, branch `loop/auto-2026-09-29`, PR #5084, Closes #5083).

## What was wrong (measured, at baseline)

| family | baseline | now |
|---|---|---|
| silently-discarded parse tokens (corpus sum) | 27,562 | **5,695 (−79.3%)** |
| specs with whole-block fallback (forall family) | 29 | **0** |
| `for` census row (quantifier tails + closure loops) | 49 events / 12 specs | **GONE** (iter 6 preserved tails, iter 7 rewrote loops) |
| `given` not-lowerable census row | 19 events / 8 specs | **5 / 2** (iter 8: 6 specs green; reader+sgd = sketch modules, left documented) |
| `clause value over-consumed` rows (assert/given/then) | 16 events / 6 specs | **GONE** (iter 9: mechanic pinned by probe — an unmodelled clause value swallows its whole block) |
| `then` not-lowerable census row | 15 events / 9 specs | **11 / 8** (iter 10: 3 rewrites incl. the family's biggest spec; remainder = documented leaves) |
| `and`/`when` not-lowerable rows | 9 events / 5 specs | **6 / 3** (iter 11: lotus + sac_critic green — a stray-`then` typo and a literal-ellipsis sketch placeholder) |
| discarding specs | 107 | **88** |
| corpus reds (ratchet ledger) | 148 | **131** (#5079 closed; iters 7-11: 17 specs fully green) |
| module-var arrays typed const ROM (typechecker) | 13 sites | **0** (iter 7 part 2) |
| truncated spec (#5079) | 1 | **0** |
| tri test suite | 824 pass / **3 fail** | **827/0 measured** (iter 5) |
| master ledger drift (stale/untracked entries) | 52 stale / 47 untracked | **0** (re-blessed) |

## The iterations

1. **Charter + baseline + competitor scan** — reframed the inventory from
   "how many reds" to "how many tokens does the parser silently throw away";
   claim taken (`tri loop claim`), branch + PR + state protocol files.
2. **forall preservation (site 1)** — `parse_invariant_clause` arm +
   `capture_to_next_top_level` walker. −13,198 discarded tokens in one
   commit; emitted bytes and every committed seal unchanged (#2774 keeps
   ownership of what forall MEANS). Found and fixed master's ledger drift
   while re-blessing.
3. **#5079 verilog_bench_harness.t27** — truncated spec closed by
   DELIBERATE rewrite (format!→concat/constants, T?→?T), not by resealing
   garbage. AST verified whole-file. Corpus 148→147.
4. **forall preservation (site 2)** — `parse_bdd_clauses` LED shape; the
   256-event census row eliminated. −5,805 more tokens (14,364→8,559).
   Plus `tri loop state` (new CLI subcommand): reads/validates the state
   file every cron firing reads first — caught a `.gitattributes`
   side-write recurrence and an unset upstream on its first live run.
5. **The August 824/3 trio** — one defect, four layers: W472 pseudo-Lean
   (#4765), merge-dropped `validate_lean_standalone` phase, ungated master
   lean CI, and d51db4ac1's partial restoration masking a dropped
   `cclk_sweep` parameter. All restored; the W472 Lean block rewritten in
   real Lean 4 (0 warnings, 0 sorry; two FALSE lemmas corrected with
   comments, not papered over). Details: iterations/05-fpga-lean-restoration.md.
6. **assert quantifier-tail preservation (site 3)** — the census top row
   (`for` mid-clause, 49 events / 12 specs) was two defects: 45 English
   quantifier tails on asserts, 3 closure loops. Tails now preserve
   verbatim under a partial mark (site-1 discipline — the first version
   lowered them and would have emitted checks on the quantifier's free
   variables; caught before commit). One clean check resurrected from a
   mixed block (constants.t27 `pow(0.0, 0.0) == 1.0`). Tokens
   8,559→7,914 (−71.3% cumulative); 5 specs discard zero now. New CLI:
   `parse-complete --fallbacks --specs`. Details:
   iterations/06-quantifier-tail-preservation.md.
7. **closure-loop rewrite + the mutability flag nobody read** — the `for`
   row's 3-event remainder was Zig dialect in the specs, not a parser gap;
   rewritten to native given/and/assert clauses, resurrecting **10 runtime
   checks that had never emitted**. The rewrite unmasked a typechecker bug
   hidden for months behind the earlier phase: `parse_var_decl` reuses the
   ConstDecl node kind with var-ness in `extra_mutable`; the generator and
   the fn-local typecheck arm read the flag, the module symbol-collection
   arm did not — every module-level `var` array typed as const ROM, W456
   hard-erroring element assignment (13 corpus sites). One-arm fix. Gate:
   0 unexpected failures / 5 unexpected passes — corpus 147→**142**.
   Details: iterations/07-closure-loop-rewrite.md.
8. **the given not-lowerable family** — decomposition first (the state.md
   instruction): 19 events / 8 specs = **5 root constructs**, 644/986
   tokens of it one thing — Zig-spelled struct values in clauses, where
   the language already owns the native spelling. Six specs rewritten
   (attention now emits **48 real checks**; mime's type unquoted and the
   recipient data survives in emitted Zig; html/xml use `undefined` for
   the map field — the corpus's own idiom, since no native map type
   exists). Two specs left DOCUMENTED as whole-module sketches (reader:
   generics + string fn-types; sgd: runtime heap allocs) — their fallback
   measures the corpus containing sketches, not a parser gap. Gate: 0
   unexpected failures / 6 passes; re-blessed 136/136 RATCHET CLEAN.
   Tokens → **7,021 (−74.5%)**; corpus 142→**136**. Details:
   iterations/08-not-lowerable-family.md.
9. **the over-consumed family, eliminated** — a probe pinned the census
   name's mechanic: a clause whose value the walker cannot model swallows
   the REST OF THE BLOCK (a trailing `then y = expr` drops its own
   given). Three constructs fixed — then-bindings → when (logging 86→0),
   python len-prefix pseudo-literals → native (gelu 112→0), Zig
   `&[_]T{...}` → native arrays (config) — plus abs-bar asserts → abs()
   (**the four L5 identity checks now execute**, incl.
   |φ² + 1/φ² − 3| < 1e-12), two finite-domain prose quantifiers
   unrolled (iteration-7 discipline), and `E_OPTIMAL = 1/e` → a
   digit-check against 19-digit e. All three census rows GONE. Left
   documented: @FieldType comptime ×5, infinite/analytic prose ×14.
   Diagnostic lesson: the census names the death site, not the cause —
   logging's `LogLevel.debug` given was innocent (probe-proven); the
   poison sat two clauses later. Tokens → **6,621 (−76.0%)**; corpus
   136→**134**. Details: iterations/09-over-consumed-family.md.
10. **the not-lowerable then-row** — decomposition found 829/986 family
   tokens in THREE specs of fixable dialect, and two undeclared-helper
   discoveries: positional_enc's `then not all_equal(...)` called a
   function the module never declares (grep: single occurrence), and
   multi_head_attn's `sum(result)` nested inside `approximately_equal`.
   Rewrites: **attention_mechanism 729→0** (repeated given heads →
   and-chains, python `[0.0] * 12` → `[0.0; 12]`, `then result == void`
   → bare `when f(...)`), positional_enc 109→34 (element-witness `!=`
   pair), multi_head_attn 70→45 (two-sided bands keep the original
   tolerances; **2 checks resurrected** that never emitted). Leaves
   documented: circular_buffer/kd_tree/segment_tree — their module APIs
   are Zig-allocator-shaped (`std.mem.Allocator` params, `*T` receivers)
   and cannot be CALLED from t27 clauses without redesigning the
   modules; sdk (C-for clause), sigmoid (comptime); #2774
   quantifier-tail invariants. Tokens → **5,792 (−79.0%)**; corpus
   134→**133**. Details: iterations/10-then-row.md.
11. **the remaining clause rows** — a sequencing discovery first: the
   helper-declaration candidate (approximately_equal ×14 undeclared
   calls) is INVISIBLE until #2774 — those specs stay pinned at
   parse-no-discard by their quantifier tails, so typecheck fixes
   cannot move the corpus count. Parse rows went first. lotus 48→0
   (a stray `then` after `and` — a TYPO — and `CycleResult{...}` with
   a literal ellipsis, replaced by undefined-field construction, the
   corpus idiom); sac_critic 49→0 (multi-line Zig-style clause bodies
   and a clause-level const → native and-chains; `then true` kept
   verbatim — the original claim was execution-survival). Leaves
   confirmed by printing blocks, not guessing: async_stream 213
   (allocator+closure+generator), io 112 (arrow-lambdas), ternary_add
   96 (#2774), filesystem 38 (allocator), sgd+reader (sketches).
   Tokens → **5,695 (−79.3%)**; corpus 133→**131**. Details:
   iterations/11-remaining-clause-rows.md.

## Falsified premises (recorded, not "fixed")

- Plan item 6 (`t27c gen --out/` swallows bad state): the directory no
  longer exists; t27c rejects unknown flags rc=2. My first test measured
  head's exit code, not t27c's (W846 hazard, caught).
- Plan item 7 (19 `.tri` files parse to silent garbage): all 19 fail LOUDLY
  today; the header describing silence is in git history, not the tree.
- Two W472 "lemmas" were false statements, not hard proofs.

## Self-critique (the honest list)

- −79.3% is TOKENS, not specs: corpus red count moved 148→131 (17 specs
  fully green across iterations 7-11; the 5 iteration-6 specs retired on
  re-bless at no-vacuous-invariant, honest until #2774 lowers forall).
- Parse-level fixable dialect is nearly EXHAUSTED by measurement: the
  remaining clause-row specs are leaves (allocator/closure/#2774/
  sketches). Further corpus-greening from spec rewrites has hit
  diminishing returns; the next gains are in #2774 lowering or in
  deciding the sketch modules' fate.
- reader.t27 and sgd.t27 (~195 tokens) are sketch modules left red on
  purpose; iteration 10 added circular_buffer/kd_tree/segment_tree
  (~197) to that family — Zig-allocator-shaped APIs that cannot be
  called natively. The family is now 5 specs / ~392 tokens of permanent
  residents unless the next loop redesigns those modules' APIs or
  deletes them. Recorded, not resolved.
- Iteration 10's band rewrites preserve the ORIGINAL tolerances (1e-4,
  1e-6) on gf16 values whose relative precision is ~1e-3 — if the
  resurrected checks fail at runtime, that is the specs' own claims
  failing honestly; the gate reported no such failure, but execution
  coverage of these two checks was not separately verified.
- Iteration 7's typechecker half was found by the gate, not by foresight —
  the first gate's "UNEXPECTED FAILURES: 1" was nearly blessed away as a
  phase move; the error's line number (code the iteration never touched)
  proved a real defect. 9 of the 13 mutability-error sites still sit
  behind earlier-phase failures — fixed in the compiler, red in the ledger
  until their phase clears.
- Iteration 6's first implementation would have shipped a false emission
  (checks on free variables); the site-1 discipline was already written in
  the file's comments. Reading precedents before writing the new site is
  now recorded as a skill step.
- Boot path still cannot take a synthetic operating point (only sweeps can)
  — deliberate scope cut, no test pins it.
- `H4Lagrangian.lean` remains red (failing analytic inequality named in
  iterations/05); it blocks nothing in tri but blocks a full `lake build`.
- The 508-seal September drift is DECOMPOSED (seal-drift-2026-09-29.md) but
  NOT resealed — mass reseal is forbidden until the branch-fate decision.
- Two user-blocked items stand: PR merges (#5078/#5081) and the lean yml
  gate (#5082, needs `gh auth refresh -h github.com -s workflows`).

## What the next loop inherits (measured map)

1. **"stopped mid-clause" rows** — vcd_conformance_compare 6 events
   (stopped on `.`) and a 6-event spec stopped on `true` — the biggest
   unexamined mass left.
2. **Undeclared test helpers** — approximately_equal ×14 (9 more blocks
   in positional_enc parse clean but fail TYPECHECK on the undeclared
   symbol; 2 call sites use a two-arg no-tolerance form), sum,
   random_input, any_mha_config, positive_u32, all_equal. PARKED with a
   measured reason: those specs are pinned at parse-no-discard by #2774
   tails, so the fix is invisible in the corpus count until #2774.
3. Sketch-module fate: reader.t27 + sgd.t27 (~195 tokens) + the
   allocator family circular_buffer/kd_tree/segment_tree (~197) +
   closure/lambda specs async_stream (213) + io (112) — rewrite,
   redesign their APIs, or delete; do not let them become permanent
   ledger residents.
4. H4Lagrangian failing lemma (named in iterations/05).
5. Boot-path synthetic operating point (restore ResolvedPvtContext or
   document the cut as permanent).
6. 9 of the 13 module-var mutability sites still red behind earlier phases
   — the compiler fix landed (iteration 7); their ledger entries clear as
   their phase clears.
7. The 682 seal reds, decomposed into: ~508 Sep-20 codegen drift,
   13 never-saved gft_* seals, 17 yosys smoke, 1 FPGA smoke, 17+135 legacy.
   Iterations 6-7 add: spec_hash is a PARSE-level hash — it drifts with
   any parser change even when all four gen hashes match (ternary_add
   measured: gen MATCH ×4, spec_hash MISMATCH).
