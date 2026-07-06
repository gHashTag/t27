# Wave Loop 458 — Issue #1429

## Goal
Select one of three W458 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w458/`, and mint an
  `XADC_LIVE_W458_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: module-level array parameter passing and yosys warning hygiene,
  with regression specs and unit tests.
- **Variant C (fallback):** if Variant B is blocked by a parser/AST refactor that
  cannot be completed safely in one wave, extend the board-less Lean 4
  boot-evidence lattice with synthesizability theorems, adversarial ±2 ns jitter
  envelope lemmas, and compiler-correctness bridge statements.

## Issue Gate
- Closes #1429 on land.
- Branch: `wave-loop-458`.
- Required: 579/579 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_457_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W457_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W458_2026-07-01.md`
