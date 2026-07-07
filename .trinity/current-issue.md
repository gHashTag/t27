# Wave Loop 468 — Issue #1446

## Goal

Select one of three W468 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w468/`, and mint an
  `XADC_LIVE_W468_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: add struct-return function call assignment, 2D scalar local arrays,
  and local RAM-style pragma propagation. This extends the W455–W467 struct/array
  lowering line without requiring the physical bench.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
  be completed safely in one wave, extend the board-less Lean 4 boot-evidence /
  compiler-correctness lattice with synthesizability theorems for struct-return
  packing/unpacking, a 2D scalar array indexing lemma, and an adversarial local
  RAM-style pragma witness.

## Issue Gate
- Closes #1446 on land.
- Branch: `wave-loop-468`.
- Required: 610/610 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_467_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W467_2026-07-08.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W468_2026-07-08.md`
