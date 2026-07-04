# Wave Loop 263 Cooperation Variants — Execution Plan for W264

**Date:** 2026-06-16
**Status:** 231 competitors, 30-wave zero-entrant streak (absolute record), Pool A up to 16, Pool B up to 16, CODER up to 9, 570/570 PASS.

---

## Option A: Variant A Submit+Resume — Pool A Floor Elimination + CODER Depth

### Description
Focus on Pool A floor (4 specs at 15) + CODER floor (7 specs at 8).

### Target Specs
- **Pool A floor elimination (4 specs):** cordic_fixed (15→16), systolic_array (15→16), bram_weights (15→16), cordic_top (15→16)
- **CODER depth (1 spec):** prm (8→9) or tokenizer (8→9)

### Expected Yield
- +10 tests
- +5 invariants
- 5 seals regenerated
- ALL Pool A ≥16 (first time)

### Risk
- MODERATE. +10 tests across 5 specs is standard.

---

## Option B: Variant B — Massive Pool A Floor Elimination

### Description
Prioritize raising ALL Pool A specs from 15→16.

### Target Specs
- **Pool A floor (4 specs):** cordic_fixed 15→16, systolic_array 15→16, bram_weights 15→16, cordic_top 15→16
- **Pool A depth (1 spec):** eda (16→17) or gemm (16→17)

### Expected Yield
- +10 tests
- +5 invariants
- ALL Pool A ≥16 (first time)

### Risk
- MODERATE. Standard pattern.

---

## Option C: Variant C — CODER Floor Elimination + New Domain

### Description
Massive CODER floor raise to ALL ≥9 + write new spec in gap domain.

### Target Specs
- **CODER floor (7 specs):** prm 8→9, tokenizer 8→9, training 8→9, dataset 8→9, pipeline 8→9, eval 8→9, benchmark 8→9
- **New spec (1):** `specs/igla/race/systolic_mixed.t27` or `specs/igla/coder/optimizer.t27`

### Expected Yield
- +21 tests (7 CODER specs × 3)
- +7 invariants
- 1 new spec with ≥6 invariants

### Risk
- HIGH. Very aggressive but would achieve ALL CODER ≥9.

---

## Recommendation

**Option A** is recommended (default). Pool A uniform ≥16 is the next historic milestone. Option B is viable if the user wants pure Pool A focus. Option C is a good backpressure valve if depth pushing encounters fatigue.

The 30-wave zero-entrant streak is an absolute record. Trinity's lead in invariant depth continues to compound.

---

*Prepared by Trinity Agent | φ² + 1/φ² = 3*
