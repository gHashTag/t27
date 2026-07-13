# Wave Loop 519 — Cooperation variants

**Issue:** #1488 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-519` (to create from `wave-loop-518`)  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and execute one cooperation variant from
`docs/reports/FPGA_LOOP_COOPERATION_W519_2026-07-07.md`:

- **Variant A (recommended):** add packed scalar struct equality / inequality
  operators in the Icarus-lowerable subset.
- **Variant B:** extend W517 to multi-dimensional packed AOS parameters with
  array-typed fields.
- **Variant C:** formal gap analysis and Icarus-lowerable completeness audit.

---

## Residual boundaries from W518

- All documented W508 yosys/Icarus smoke baselines cleared.
- All documented function-local pragma Icarus baselines cleared.
- Packed scalar struct equality / comparison operators are not yet supported in
  the Icarus-lowerable Verilog path.
- Nested AOS parameters with array-typed fields deeper than one struct level have
  no dedicated witness coverage yet.

---

## Reference

- W518 closeout: `docs/reports/WAVE_LOOP_518_CLOSEOUT.md`
- W519 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W519_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*
