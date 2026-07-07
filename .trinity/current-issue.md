# Wave Loop 471 — Issue #1449

## Goal

Select one of three W471 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w471/`, and mint an
  `XADC_LIVE_W471_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: add nested struct literal packing in expression contexts, struct
  fields that are arrays (`Outer { pts : [3]Pt }`), direct field access on
  returned arrays of structs (`make_pts(0)[0].x`), and array-of-struct parameter
  literal arguments. This extends the W455–W470 struct/array lowering line
  without requiring the physical bench.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
  be completed safely in one wave, add Lean 4 synthesizability / correctness
  lemmas for module-level writable struct arrays, array-of-struct return
  round-trip, and an adversarial yosys-elaboration witness for new W471
  scratch specs.

## Issue Gate
- Closes #1449 on land.
- Branch: `wave-loop-471`.
- Required: ≥622/622 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_470_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W471_2026-07-08.md`
