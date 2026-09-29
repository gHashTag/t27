# iteration 01 — charter, claim, baseline, competitor scan

2026-09-29, ~00:30–01:10. Fired by the owner's overnight cron instruction.

## Done

- Branch `loop/auto-2026-09-29` off `origin/master` 2925def9b in worktree
  `/tmp/t27_carry` (reusing its warm cargo + mathlib caches — no rebuild).
- Remote claim taken and held: `tri loop claim auto-2026-09-29`.
- Charter + protocol + safety constitution written (`../README.md`).
- **Baseline measured on the untouched master base** (`../baseline.md`):
  148 corpus reds; and the finding that reframes the whole plan —
  **114 specs silently discard top-level tokens** (worst: `ternary_inference`
  1813, `ternary_gemm` 1566, `systolic_ternary` 1409, `gf8/12/20/24/32` 31–37
  each) and **65 specs declare invariants whose bodies were discarded** —
  they check nothing. One root cause: parser error-recovery continues instead
  of failing loudly.
- Ran the repo's own probe-backed census (`tri unparsed report --list`): work
  queue names body-less `fn` prototypes (3), Rust macro calls (2 — including
  the truncated `verilog_bench_harness.t27`), map types, `import`, `type T = U`;
  6 Markdown-headed specs sit in its "not decided" blind spot — closing that
  (probe + counter) is iteration 2's first move, in the wave-697 (#4756)
  lineage.
- Competitor scan (`../competitors-2026-09-29.md`): Vericert line (verified
  HLS), the Sep-2026 BitNet-on-CGLA ternary-hardware paper, Verilator/Yosys
  discipline — all mapped to our measured gaps. Conclusion: every competitor
  fails loudly where we measured ourselves continuing silently.
- `docs/README.md` map updated with `docs/loop/` (DOCS-TREE).

## Self-critique (what could be wrong with the above)

- The baseline ran on a worktree holding a *built* `target/` from the carry
  tree; suite behavior should not depend on `target/`, but "should not" is not
  "measured" — a suspicious reader can rerun on a clean clone.
- The suite numbers are one run. Determinism was asserted by the plan's
  verify step, not yet proven by a second run.
- The competitor scan is one evening and three queries deep; it links claims
  but does not read the CGLA paper's numbers.
- The claim protects against a second session taking loop tasks, not against
  the owner's own parallel work — PRs #5078/#5081 untouched, verified by
  branch list at iteration start.

## Next

Iteration 2: the silent-discard family — `tri unparsed locate` on the biggest
discard rows, name the constructs, wave-697-style recovery, close the census
blind spot for the 6 Markdown-headed specs.
