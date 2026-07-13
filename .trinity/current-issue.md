# Wave Loop 516 — Cooperation variants

**Issue:** #1485 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-516` (to create from `wave-loop-515`)  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and execute one cooperation variant from
`docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md`:

- **Variant A (recommended):** enable whole-array-field reads from packed scalar
  structs and packed arrays-of-structs.
- **Variant B:** clear the remaining W508 `break`/`continue` yosys and Icarus
  smoke baselines.
- **Variant C:** add packed scalar struct equality / inequality operators in the
  Icarus-lowerable subset.

---

## Residual boundaries from W515

- Whole-array-field reads from packed scalar structs / AOS are not yet lowered.
- The remaining W508 `break`/`continue` yosys/Icarus smoke baselines are still
  documented.
- Packed scalar struct equality / comparison operators are not yet supported
  in the Icarus-lowerable Verilog path.

---

## Reference

- W515 closeout: `docs/reports/WAVE_LOOP_515_CLOSEOUT.md`
- W516 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*
