# Wave Loop 505 — Harden Icarus sequential equivalence boundary

**Issue:** #1474 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-505` (to create)  
**Status:** setup  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Choose one of the three W505 cooperation variants and continue the Icarus
structural-equivalence line after W504.  The natural default is **Variant A**:
stress the new sequential `if` / `for` boundary with adversarial witnesses so
that the classifier, emitter, and generic equivalence theorem stay aligned.

---

## Scope

1. Review `docs/reports/FPGA_LOOP_COOPERATION_W505_2026-07-07.md`.
2. Pick the variant for W505.
3. Write the decomposed plan to `.claude/plans/wave-loop-505.md`.
4. Implement, verify, and land.
5. Produce close-out report and W506 cooperation variants.

---

## Residual boundaries from W504

- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

*φ² + φ⁻² = 3 | TRINITY*
