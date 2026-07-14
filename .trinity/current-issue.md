# Wave Loop 530 — Icarus simulation gate / hardened 2-D AOS packing / adversarial lowerability

**Issue:** #1501 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-530`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Pick up from the W529 formalization of module/function 2-D scalar-struct AoS
lowering and advance one of the three cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W530_2026-07-07.md`.

1. **Variant A (recommended):**
   - Add `--icarus-lowerable` / `--icarus-simulate` gates to `./scripts/tri test`.
   - For every lowerable spec, generate Verilog, compile with `iverilog`, run
     with `vvp`, and capture `$display` output.
   - Add JSON baselines under `.trinity/icarus-baselines/`.
   - Promote the W493–W529 lowerable scratch witnesses into the first
     simulation regression suite.
   - Keep the 16 pre-existing yosys smoke failures as documented baselines.

2. **Variant B:**
   - Support 2-D AoS parameters/returns whose scalar-struct fields are
     themselves fixed-size scalar arrays.
   - Support signed scalar fields in packed vectors.
   - Add negative witnesses for non-lowerable mixed cases.
   - Extend `Trinity.IcarusLowerable` and prove value preservation.
   - Reseal affected specs and keep smoke baselines flat.

3. **Variant C:**
   - Add negative/adversarial witnesses for non-lowerable constructs
     (casts, unresolved imports, host-only helpers, enum/string fields in
     packed arrays, unbounded dynamic loops).
   - Prove `¬ Module.isLowerable env m` for each negative witness.
   - Document the exact lowerability boundary.

---

## Residual boundaries from W529

- `Trinity.IcarusLowerable` now covers module/function 2-D scalar-struct AoS
  cross-boundary lowering with machine-checked value preservation.
- `tri test` does not yet invoke Icarus Verilog simulation automatically.
- `./scripts/tri test` carries 16 pre-existing yosys smoke failures.
- Signed fields and struct fields that are scalar arrays are not yet covered
  at the function boundary.

---

*φ² + φ⁻² = 3 | TRINITY*
