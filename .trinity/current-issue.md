# Wave Loop 474 — Next wave (to be selected from cooperation plan)

## Goal

Select one of the three W474 cooperation variants and close the wave with a green suite, updated seals, and the standard close-out artifacts (report, evidence, next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w474/`, and mint an `XADC_LIVE_W474_OPERATING_POINT` theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend hardening: function-local arrays of structs with array-typed fields, array-of-struct function returns with nested field writeback, scalar-struct equality / whole-struct comparison, an adversarial yosys-elaboration witness, and optional Lean synthesizability lemmas for the per-field memory model.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot be completed safely in one wave, add Lean 4 synthesizability / correctness lemmas for module-level arrays of structs with array-typed fields, array-of-struct return round-trip, and an adversarial yosys-elaboration witness for new W474 scratch specs.

## Issue Gate
- Branch: `wave-loop-474`.
- Required: ≥633/633 non-smoke PASS (or acceptable baseline), smoke gate acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_473_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W474_2026-07-08.md`
- Parent wave: #1447
