# Wave Loop 521 — Cooperation variants

**Issue:** #1490 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-521` (to create from `wave-loop-520`)  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and execute one cooperation variant from
`docs/reports/FPGA_LOOP_COOPERATION_W521_2026-07-07.md`:

- **Variant A (recommended):** add Lean 4 proof witnesses for the W520 multi-
  dimensional array-of-structs parameter paths (register-mode and packed-element
  AOS).
- **Variant B:** extend multi-dimensional AOS lowering to function returns and
  module-level whole-array assignment from calls.
- **Variant C:** harden the Icarus-lowerable classifier for AOS parameter shapes
  and add adversarial negative witnesses.

---

## Residual boundaries from W520

- Module-level and local 2-D/3-D AOS parameters now lower correctly for
  register-mode, packed-element, and memory-mode element structs.
- Formal Lean 4 proof witnesses for the new multi-dimensional AOS parameter
  cases are not yet written.
- Multi-dimensional AOS **return values** and module-level assignment from
  function calls remain unimplemented.
- The Icarus-lowerable classifier does not explicitly reject non-lowerable AOS
  parameter shapes (e.g. structs containing string/enum/f32 fields).

---

## Reference

- W520 closeout: `docs/reports/WAVE_LOOP_520_CLOSEOUT.md`
- W521 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W521_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*
