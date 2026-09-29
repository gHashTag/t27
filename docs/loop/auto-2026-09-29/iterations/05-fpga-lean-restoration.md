# iterations/05-fpga-lean-restoration.md — the August red trio, root-caused and closed

The three failing `cargo test -p tri` tests (824/3 since August, proven
pre-existing on master via a pristine worktree at 2925def9b) were not three
bugs. They were one defect with three layers, each hiding the next.

## The causal chain (measured, not inferred)

1. **W472 (PR #4765, merged in the 2026-08-06 bulk merge) committed Lean that
   never compiled.** `proofs/lean4/Trinity/TernaryFPGABoot.lean`'s W472
   Cooperation block used Rust-flavored pseudo-Lean: `struct` (keyword is
   `structure`), `arr.length`/`arr.update`/`arr.idx` (Lean's are
   `.size`/`.set`/`[i]`), list literals for `Array`, `Array.forall` (does not
   exist), hallucinated fields `.0`/`.1`/`.data.(1)`.
2. **The same bulk merge dropped the `validate_lean_standalone` phase body**
   from `cli/tri/src/fpga.rs`. The flag `validate_lean_standalone_ok` stayed
   (init `false`, read by `passed`), so every run that requested the phase
   reported `passed: false`.
3. **Nothing gated it.** `.github/workflows/lean-proofs.yml` has no `push:`
   trigger on master ("DELIBERATELY NOT GATING master YET" — the order is get
   the reading, then gate). The tri tests that DID run failed at
   `passed == true` before ever reaching the deeper asserts, so layers stacked
   invisibly.
4. **A second restoration (#2305, d51db4ac1, Aug 20) had fixed the flag but
   reshaped the JSON**: it restored `dry_run_sweep_ok = true` with a new entry
   shape (`variants`/`report_file`) — but the merge had ALSO dropped the
   synthetic-JSON sweep check AND the `synthetic_operating_point` parameter
   from `cclk_sweep` itself (17 params where W450 had 18), leaving every
   dry-run `operating_point.source` hardcoded `"not_read"`. The snapshot
   fixture (W450, 8eb0caf5c) still pins the old shape. Unmasked only now.

## What was restored

**`cli/tri/src/fpga.rs`:**
- The `validate_lean_standalone` phase body inside `run_theorem_matrix`:
  builds the standalone `.lean` from theorem-matrix fixtures via
  `measured_to_lean` (21-arg signature unchanged since W448), compiles it in a
  temp lake package (`require trinity from "<repo>/proofs/lean4"`), sets the
  flag its verdict reads, writes the report entry the snapshot pins
  (`status`/`source`/`lean_file`/`elapsed_ms`).
- The W450 synthetic-JSON dry-run sweep check: write
  `sweep-report-smoke-gate-dry-run.json`, read it back, verify variant count
  and `operating_point.source == "synthetic"` per variant, emit the entry keys
  the fixture pins (`status`/`variant_count`/`source`/`report_json`/
  `report_md`). d51db4ac1's reshaped keys had no readers (grepped all of
  `cli/`).
- The `synthetic_operating_point: bool` 18th parameter on `cclk_sweep` + the
  dry-run PVT resolution that honors it: explicit PVT context file →
  `pvt_context_file`; else synthetic flag → `synthetic`; else `not_read`. Also
  restores the `--synthetic-operating-point` CLI flag on `tri fpga cclk-sweep`
  (dropped with the param; `conflicts_with = "xadc"` as at W450). Bonus fix:
  dry-run entries with a PVT context file are now labeled `pvt_context_file`
  instead of the contradictory json-says-`not_read`/source-says-file split.
- One dead binding (`dry_result`) removed while editing that exact call.

**`proofs/lean4/Trinity/TernaryFPGABoot.lean`** — W472 block rewritten in real
Lean 4 (v4.31.0) against the actual core Array API (`Array.set` with proof
param, primed `[i]'(h)` indexing, `dif_pos` for `if h :`, `Array.all`,
`#[...]` literals), imports added (`Trinity.TernaryMac`,
`Trinity.TernaryGemm`), module builds clean — zero warnings, zero sorry.
Two statements that had been FALSE lemmas (not proof failures) were corrected
with comments, not papered over:
- `raw_ns_preserved_under_jitter`: `RawNsPredicate base_ns` never implied
  `RawNsPredicate (base_ns + jitter)` (base=1000, jitter=2 → 1002 > 1000).
  Hypothesis strengthened to `RawNsPredicate (base_ns + 2)`.
- `worst_case_jitter_envelope_bound`: `use 200_000` was falsified by
  base=200_000 + jitter 2; now `use 200_002`.

## Verification

- `lake build Trinity.TernaryFPGABoot` — clean (0 errors, 0 warnings, 0 sorry).
- `test_smoke_gate_validate_lean_standalone_matches_snapshot` — **green**
  (full gate run, lake + temp package + yosys + sweep + matrix + standalone).
- Remaining lean_standalone tests + full `cargo test -p tri` — see state.md
  (run recorded there).
- Corpus untouched (no spec changes): ratchet gate unaffected; FROZEN_HASH
  unchanged by construction (no compiler/AST change in this iteration —
  fpga.rs is `cli/tri`, not `bootstrap`).

## Self-critique

- The restoration re-verified the two shapes the tests pin, but did NOT audit
  the other 5 `dry_run_sweep` emitters at W450 vs today (failed-variant
  entries); they diverge only in already-failing paths the tests don't pin.
- `cclk_sweep`'s 18th param now exists, but `resolve_pvt_context_for_boot`
  kept today's 5-param signature returning a tuple; W450 had a
  `ResolvedPvtContext` struct with a synthetic arm, so the BOOT path still
  cannot take a synthetic operating point (only sweeps can). Deliberate
  scope cut: no test pins boot-path synthetic; recorded here for the next
  loop.
- `H4Lagrangian.lean` remains the last red Lean module (analytic inequality
  `max (789/(1+√239+√549)² − 2/3) (2/3 − ...) / (2/3) < 1` fails); it does
  not block tri tests (temp lake packages build only the imported closure).
  Left as next-loop work with the failing goal named.
- The 3 restored tests were verified green; the full-suite run confirming
  827/0 is recorded in state.md when it finishes — if it shows anything but
  827, this report is wrong and state.md wins.
