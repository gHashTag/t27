# Wave Loop 504 — Next step for Icarus sequential equivalence

**Issue:** #1473 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-504` (to create)  
**Status:** setup  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Choose one of the three W504 cooperation variants and continue the Icarus
structural-equivalence line after W503.  The natural default is **Variant A**:
extend the generic forward-simulation theorem to bounded `forLoop`, removing the
"lowerable but non-combinational" residual boundary.

---

## Scope

1. Review `docs/reports/FPGA_LOOP_COOPERATION_W504_2026-07-07.md`.
2. Pick the variant for W504.
3. Write the decomposed plan to `.claude/plans/wave-loop-504.md`.
4. Implement, verify, and land.
5. Produce close-out report and W505 cooperation variants.

---

## Residual boundaries from W503

- Bounded `forLoop` is modeled and lowerable, but not yet covered by the generic
  `module_value_equiv_statement` theorem.
- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

*φ² + φ⁻² = 3 | TRINITY*
