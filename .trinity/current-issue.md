# Wave Loop 518 — Cooperation variants

**Issue:** #1487 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-518` (to create from `wave-loop-517`)  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and execute one cooperation variant from
`docs/reports/FPGA_LOOP_COOPERATION_W518_2026-07-07.md`:

- **Variant A (recommended):** clear the remaining W508
  `break`/`continue` yosys and Icarus smoke baselines.
- **Variant B:** add packed scalar struct equality / inequality operators in
  the Icarus-lowerable subset.
- **Variant C:** extend W517 to deeper / multi-dimensional packed AOS
  parameters with array-typed fields.

---

## Residual boundaries from W517

- W508 `break`/`continue` early-exit yosys/Icarus baselines remain (2 yosys,
  3 Icarus).
- Packed scalar struct equality / comparison operators are not yet supported in
  the Icarus-lowerable Verilog path.
- Nested AOS parameters with array-typed fields deeper than one struct level
  have no dedicated witness coverage yet.

---

## Reference

- W517 closeout: `docs/reports/WAVE_LOOP_517_CLOSEOUT.md`
- W518 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W518_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*
