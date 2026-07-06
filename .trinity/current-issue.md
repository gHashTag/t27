# Wave Loop 460 — Issue #1433

## Goal
Select one of three W460 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w460/`, and mint an
  `XADC_LIVE_W460_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: generalize array-parameter support to multiple/different call sites,
  lower bench-block local variables to declared registers, and clear the three
  pre-existing `let_binding` cargo-test failures.
- **Variant C (fallback):** if Variant B is blocked by a parser/AST scope refactor
  that cannot be completed safely in one wave, extend the board-less Lean 4
  boot-evidence lattice with synthesizability theorems, adversarial ±2 ns jitter
  envelope lemmas, and compiler-correctness bridge statements.

## Issue Gate
- Closes #1433 on land.
- Branch: `wave-loop-460`.
- Required: 583/583 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_459_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W459_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W460_2026-07-01.md`
