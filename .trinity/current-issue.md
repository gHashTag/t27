# Wave Loop 517 — Cooperation variants

**Issue:** #1486 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-517` (to create from `wave-loop-516`)  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and execute one cooperation variant from
`docs/reports/FPGA_LOOP_COOPERATION_W517_2026-07-07.md`:

- **Variant A (recommended):** enable whole-array-field reads from packed
  array-of-structs **parameters** and return them from functions.
- **Variant B:** clear the remaining W508 `break`/`continue` yosys and Icarus
  smoke baselines.
- **Variant C:** add packed scalar struct equality / inequality operators in the
  Icarus-lowerable subset.

---

## Residual boundaries from W516

- Packed AOS **parameter** whole-array-field reads are not yet lowered.
- The remaining W508 `break`/`continue` yosys/Icarus smoke baselines are still
  documented.
- Packed scalar struct equality / comparison operators are not yet supported in
  the Icarus-lowerable Verilog path.

---

## Reference

- W516 closeout: `docs/reports/WAVE_LOOP_516_CLOSEOUT.md`
- W517 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W517_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*
