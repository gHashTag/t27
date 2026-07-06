# Wave Loop 463 — Issue #1439

## Goal
Select one of three W463 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w463/`, and mint an
  `XADC_LIVE_W463_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: propagate array-parameter binding signatures through nested calls,
  allow struct-literal array arguments, and clear one more safe `gen-verilog`
  sub-defect from `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- **Variant C (fallback):** if Variant B is blocked by a call-graph refactor that
  cannot be completed safely in one wave, extend the board-less Lean 4
  boot-evidence lattice with synthesizability theorems, adversarial envelope
  lemmas, and compiler-correctness bridge statements.

## Issue Gate
- Closes #1439 on land.
- Branch: `wave-loop-463`.
- Required: 590/590 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_462_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W462_2026-07-07.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W463_2026-07-07.md`
