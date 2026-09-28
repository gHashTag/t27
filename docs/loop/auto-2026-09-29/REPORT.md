# Loop auto-2026-09-29 — final report

One pass, five iterations, every number below measured on this tree
(`/tmp/t27_carry`, branch `loop/auto-2026-09-29`, PR #5084, Closes #5083).

## What was wrong (measured, at baseline)

| family | baseline | now |
|---|---|---|
| silently-discarded parse tokens (corpus sum) | 27,562 | **8,559 (−69%)** |
| specs with whole-block fallback (forall family) | 29 | **0** |
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

## Falsified premises (recorded, not "fixed")

- Plan item 6 (`t27c gen --out/` swallows bad state): the directory no
  longer exists; t27c rejects unknown flags rc=2. My first test measured
  head's exit code, not t27c's (W846 hazard, caught).
- Plan item 7 (19 `.tri` files parse to silent garbage): all 19 fail LOUDLY
  today; the header describing silence is in git history, not the tree.
- Two W472 "lemmas" were false statements, not hard proofs.

## Self-critique (the honest list)

- −69% is TOKENS, not specs: corpus red count moved 148→147 (one), because
  preservation stops the discarding without changing what lowers.
- Mid-block forall shape (lowered clauses, then forall, then more clauses)
  was not separately re-measured after site 2; its 49-event/12-spec census
  row is next loop's first candidate.
- Boot path still cannot take a synthetic operating point (only sweeps can)
  — deliberate scope cut, no test pins it.
- `H4Lagrangian.lean` remains red (failing analytic inequality named in
  iterations/05); it blocks nothing in tri but blocks a full `lake build`.
- The 508-seal September drift is DECOMPOSED (seal-drift-2026-09-29.md) but
  NOT resealed — mass reseal is forbidden until the branch-fate decision.
- Two user-blocked items stand: PR merges (#5078/#5081) and the lean yml
  gate (#5082, needs `gh auth refresh -h github.com -s workflows`).

## What the next loop inherits (measured map)

1. "for mid-clause" 49 events / 12 specs — the largest remaining discard row.
2. H4Lagrangian failing lemma (named in iterations/05).
3. Boot-path synthetic operating point (restore ResolvedPvtContext or
   document the cut as permanent).
4. The 681 seal reds, now decomposed into: 508 Sep-20 codegen drift,
   13 never-saved gft_* seals, 17 yosys smoke, 1 FPGA smoke, 17+135 legacy.
