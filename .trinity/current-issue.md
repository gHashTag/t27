# Wave Loop 528 — 2-D AOS cross-boundary lowering / soundness / tooling

**Issue:** #1499 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-528`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Pick up from the W527 packed-vector 2-D array-of-scalar-struct lowering and
advance one of the three cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W528_2026-07-14.md`.

1. **Variant A (recommended):**
   - Extend 2-D scalar-struct lowering to module-level packed parameters.
   - Add 2-D AOS function-parameter passing and return values.
   - Add scratch witnesses for module param, function param, function return,
     and whole-array copy.
   - Update `detect_unsupported_verilog_locals` to allow the new shapes.
   - Reseal affected specs and keep `./scripts/tri test` failures at the 16
     pre-existing smoke baselines.

2. **Variant B:**
   - Formalize the 2-D packed-vector AoS layout in `Trinity.IcarusLowerable`.
   - Prove value preservation for the W527 witness (or a cleaned module-level
     variant) via `module_value_equiv_proved_sequential`.
   - Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

3. **Variant C:**
   - Add an Icarus simulation gate to `tri test` for the lowerable subset.
   - Add a seal-drift CI job that fails on `t27c seal --verify` mismatches.
   - Audit and document the 16 pre-existing yosys smoke failures.
   - Write an ADR summarizing the W469–W527 codegen delta.

---

## Residual boundaries from W527

- Function-local 2-D arrays of scalar structs lower correctly.
- Module-level 2-D AOS parameters and cross-function 2-D AOS values are still
  rejected or not yet emitted.
- The IcarusLowerable Lean 4 stack has not yet been extended to model the new
  packed-vector layout.
- `./scripts/tri test` carries 16 pre-existing yosys smoke failures.

---

*φ² + φ⁻² = 3 | TRINITY*
