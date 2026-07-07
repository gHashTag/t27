# Wave Loop 472 — Issue #1450

## Goal

Select one of three W472 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w472/`, and mint an
  `XADC_LIVE_W472_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: deeply nested array-of-struct literals, module-level writable struct
  arrays with array fields (`var mem : [N]Shape`), direct deeply nested
  returned-array field access (`make_shape(0)[i].pts[j].x`), and formal
  synthesizability lemmas for the per-field memory model.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
  be completed safely in one wave, add Lean 4 synthesizability / correctness lemmas
  for module-level writable arrays of scalar structs, array-of-struct return
  round-trip, and an adversarial yosys-elaboration witness for new W472 scratch
  specs.

## Issue Gate
- Closes #1450 on land.
- Branch: `wave-loop-472`.
- Required: ≥626/626 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_471_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W472_2026-07-08.md`
