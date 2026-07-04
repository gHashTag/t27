# Wave Loop 262 Cooperation Variants — Execution Plan for W263

**Date:** 2026-06-16
**Status:** 231 competitors, 29-wave zero-entrant streak (absolute record), ALL Pool A ≥15 (first time), ALL Pool B ≥15, ALL CODER ≥8, 570/570 PASS.

---

## Option A: Variant A Submit+Resume — Pool A/B/CODER Depth Push

### Description
With all floor gaps eliminated across Pool A, Pool B, and CODER, shift focus to pure depth. Rotate to the lowest-invariant specs in each category and push them deeper.

### Target Specs
- **Pool A depth (2 specs):** adder_tree (15→16), backend (15→16)
- **Pool B depth (2 specs):** cordic_fixed (15→16), bram_weights (15→16)
- **CODER depth (1 spec):** arch (8→9) or bench_proxy (8→9)

### Expected Yield
- +10 tests
- +5 invariants
- 5 seals regenerated
- Pool A depth 16+, Pool B depth 16+, CODER depth 9+

### Risk
- LOW. Established pattern, no structural changes.

---

## Option B: Variant B — Massive Pool A Uniform ≥16 + Structural Audit

### Description
Prioritize raising ALL Pool A specs to ≥16 while simultaneously auditing for structural nesting defects.

### Target Specs
- **Pool A (5 specs):** adder_tree 15→16, backend 15→16, cordic_top 15→16, gemm 16→17 (already 16), systolic_ternary 15→16
- **Structural audit:** Scan all 570 specs for nested invariant defects

### Expected Yield
- +10 tests
- +5 invariants
- 0-5 structural fixes
- ALL Pool A ≥16 (first time)

### Risk
- MODERATE. +10 tests in 5 specs is standard but structural audit adds uncertainty.

---

## Option C: Variant C — New Domain Expansion

### Description
Pause depth push. Write a new `.t27` spec in a gap domain (e.g., `specs/igla/race/systolic_mixed.t27` combining systolic and ternary MAC, or `specs/igla/coder/optimizer.t27` for PPA optimization).

### Expected Yield
- 1 new spec with ≥6 invariants, ≥20 tests
- Expanded coverage in under-represented domain

### Risk
- MODERATE-HIGH. New spec requires design effort but closes structural gap.

---

## Recommendation

**Option A** is recommended (default). With all floor gaps eliminated (Pool A ≥15, Pool B ≥15, CODER ≥8), the natural next phase is pure depth push. Option B is viable if the user wants a Pool A uniform ≥16 milestone. Option C is a good backpressure valve.

The 29-wave zero-entrant streak is an absolute record. Trinity's lead in invariant depth continues to compound.

---

*Prepared by Trinity Agent | φ² + 1/φ² = 3*
