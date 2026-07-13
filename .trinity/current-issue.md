# Wave Loop 520 — Cooperation variants

**Issue:** #1489 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-520` (to create from `wave-loop-519`)  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and execute one cooperation variant from
`docs/reports/FPGA_LOOP_COOPERATION_W520_2026-07-07.md`:

- **Variant A (recommended):** extend W517 and W519 to multi-dimensional
  packed arrays-of-structs (AOS) parameters with array-typed fields.
- **Variant B:** formal soundness for scalar struct comparisons (equality and
  ordering) in the Lean 4 IcarusLowerable proof stack.
- **Variant C:** Icarus-lowerable classifier hardening and adversarial negative
  witnesses for struct comparisons.

---

## Residual boundaries from W519

- Scalar struct equality/inequality already lower correctly in the
  Icarus-lowerable Verilog path.
- Ordering comparisons (`<`, `<=`, `>`, `>=`) on local/param/module scalar
  structs now lower to packed-vector comparisons (W519 landed).
- Multi-dimensional AOS parameters with array-typed fields deeper than one
  struct level still have no dedicated witness coverage.
- Lean 4 formal proofs for scalar struct relational operators are not yet
  written.
- The static Icarus classifier does not explicitly reject non-lowerable
  struct comparison shapes.

---

## Reference

- W519 closeout: `docs/reports/WAVE_LOOP_519_CLOSEOUT.md`
- W520 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W520_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*
