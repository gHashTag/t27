# Wave Loop 470 — Issue #1448

## Goal

Select one of three W470 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w470/`, and mint an
  `XADC_LIVE_W470_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: add arrays of structs returned from functions, module-level
  `var mem : [N]Pt` RAM-style read/write, 2D scalar array parameter literals,
  and nested struct literal packing in expression contexts. This extends the
  W455–W469 struct/array lowering line without requiring the physical bench.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
  be completed safely in one wave, add Lean 4 synthesizability / correctness
  lemmas for scalar struct parameter flattening and whole-struct comparison,
  plus an adversarial yosys-elaboration witness for new struct-array specs.

## Issue Gate
- Closes #1448 on land.
- Branch: `wave-loop-470`.
- Required: ≥620/620 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_469_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W470_2026-07-08.md`
