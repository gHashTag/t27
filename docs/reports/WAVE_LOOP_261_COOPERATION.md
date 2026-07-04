# Wave Loop 261 Cooperation Variants — Execution Plan for W262

**Date:** 2026-06-16  
**Status:** 231 competitors, 28-wave zero-entrant streak (absolute record), ALL CODER ≥8 (first time), ALL Pool B ≥15 (first time), 570/570 PASS.

---

## Option A: Variant A Submit+Resume — Pool A Floor Elimination + Depth

### Description
Focus on Pool A — the last remaining frontier. Pool A has 3 specs at 14 (eda, formal, systolic_array). Raise all three to 15 while pushing 2 Pool A depth specs.

### Target Specs
- **Pool A floor elimination (3 specs):** eda (14→15), formal (14→15), systolic_array (14→15)
- **Pool A depth (2 specs):** rtl (15→16), gemm (15→16)

### Expected Yield
- +13 tests
- +5 invariants
- 5 seals regenerated
- ALL Pool A ≥15 (first time in history)
- Pool A depth 16+

### Risk
- MODERATE. +13 tests is aggressive but achievable. All Pool A specs are well-understood.

---

## Option B: Variant B — Massive Pool A Floor Elimination + Structural Audit

### Description
Prioritize eliminating ALL Pool A floor specs (3 specs at 14→15) and simultaneously audit all specs for structural nesting defects (legacy from W254-W255).

### Target Specs
- **Pool A floor (3 specs):** eda 14→15, formal 14→15, systolic_array 14→15
- **Structural audit:** Scan all 570 specs for nested invariant defects, fix 0-5 critical cases
- **Pool B depth (1 spec):** adder_tree (15→16) or opcodes (15→16)

### Expected Yield
- +8 tests (3 Pool A + 1 Pool B)
- +4 invariants
- 0-5 structural fixes
- ALL Pool A ≥15 (first time)

### Risk
- MODERATE-HIGH. Structural audit adds uncertainty but closes a long-standing vulnerability.

---

## Option C: Variant C — New Domain Expansion + Pool A Floor

### Description
Pause pure depth push. Write a new `.t27` spec in a gap domain (e.g., `specs/igla/race/systolic_mixed.t27` or `specs/igla/coder/optimizer.t27`) + raise 1 Pool A floor spec.

### Expected Yield
- 1 new spec with ≥6 invariants, ≥20 tests
- 1 Pool A spec raised 14→15 (+2 tests, +1 invariant)
- Expanded coverage in under-represented domain

### Risk
- MODERATE-HIGH. New spec requires design effort but closes structural gap.

---

## Recommendation

**Option A** is recommended (default). Pool A is the last remaining frontier — eliminating the 14-invariant floor across ALL Pool A specs would be a historic milestone. Option B is viable if the user wants structural hygiene alongside depth. Option C is a good backpressure valve if depth pushing encounters fatigue.

The 28-wave zero-entrant streak is an absolute record. Trinity's lead in invariant depth continues to compound. Pool A uniform ≥15 is the next historic target.

---

*Prepared by Trinity Agent | φ² + 1/φ² = 3*
