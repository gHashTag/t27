# Loop auto-2026-09-29 — final report

One pass, five iterations, every number below measured on this tree
(`/tmp/t27_carry`, branch `loop/auto-2026-09-29`, PR #5084, Closes #5083).

## What was wrong (measured, at baseline)

| family | baseline | now |
|---|---|---|
| silently-discarded parse tokens (corpus sum) | 27,562 | **7,914 (−71.3%)** |
| specs with whole-block fallback (forall family) | 29 | **0** |
| `for` census row (quantifier tails) | 49 events / 12 specs | **3 events** (closure loops, a different defect) |
| discarding specs | 107 | **102** (sacred_physics, gf12/20/24/32 discard zero) |
| corpus reds (ratchet ledger) | 148 | **147** (#5079 closed) |
| truncated spec (#5079) | 1 | **0** |
| tri test suite | 824 pass / **3 fail** | **827/0 measured** (iter 5) |
| master ledger drift (stale/untracked entries) | 52 stale / 47 untracked | **0** (re-blessed, cap 147) |

## The five iterations

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

## Falsified premises (recorded, not "fixed")

- Plan item 6 (`t27c gen --out/` swallows bad state): the directory no
  longer exists; t27c rejects unknown flags rc=2. My first test measured
  head's exit code, not t27c's (W846 hazard, caught).
- Plan item 7 (19 `.tri` files parse to silent garbage): all 19 fail LOUDLY
  today; the header describing silence is in git history, not the tree.
- Two W472 "lemmas" were false statements, not hard proofs.

## Self-critique (the honest list)

- −71.3% is TOKENS, not specs: corpus red count moved 148→147 by count (plus
  5 specs whose discard reading went to zero in iteration 6, retired on
  re-bless), because preservation stops the discarding without changing
  what lowers.
- The `for` census row's remainder (3 closure-loop events) is a DIFFERENT
  defect deliberately left in place — first candidate for the next
  iteration.
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

1. Closure-loop shape — `for (const i) |reg| in [...] {` at clause position:
   3 events (registers, ternary_memory, ops), the remainder of the old `for`
   row.
2. "clause not lowerable" rows: given 19/8, then 15/9, assert 10/4, and 7/4
   — the next-largest fallback families after the forall work.
3. H4Lagrangian failing lemma (named in iterations/05).
4. Boot-path synthetic operating point (restore ResolvedPvtContext or
   document the cut as permanent).
5. The 681 seal reds, now decomposed into: 508 Sep-20 codegen drift,
   13 never-saved gft_* seals, 17 yosys smoke, 1 FPGA smoke, 17+135 legacy.
   Iteration 6 adds a name: spec_hash is a PARSE-level hash — it drifts with
   any parser change even when all four gen hashes match (ternary_add
   measured: gen MATCH ×4, spec_hash MISMATCH).
