# Wave Loop 260 Cooperation Variants — Execution Plan for W261

**Date:** 2026-06-16  
**Status:** 231 competitors, 27-wave zero-entrant streak (absolute record), ALL Pool A ≥14 (first time), 570/570 PASS.

---

## Option A: Variant A Submit+Resume — Pool A/B Depth + CODER Floor Raise

### Description
Continue the invariant-first depth march. Pool A floor is **ELIMINATED** (ALL ≥14). Pool B has 6 specs at 14. CODER has 4 specs at 7. Rotate to CODER floor raise (raise one of benchmark/pipeline/prm/training from 7→8) + 2 Pool A depth specs + 2 Pool B depth specs.

### Target Specs
- **Pool A depth (2 specs):** cordic_fixed (14→15), rtl (14→15)
- **Pool B depth (2 specs):** cordic (14→15), systolic_array (14→15)
- **CODER floor raise (1 spec):** benchmark (7→8) or training (7→8) — oldest untouched CODER specs

### Expected Yield
- +11 tests
- +5 invariants
- 5 seals regenerated
- ALL CODER ≥8 (first time in history)
- Pool A depth 15+, Pool B depth 15+

### Risk
- LOW. Established pattern, no structural changes.

---

## Option B: Variant B — Massive CODER Floor Raise + Pool B 15 Uniform

### Description
Prioritize CODER floor raise to ALL ≥8 while simultaneously pushing Pool B specs at 14→15.

### Target Specs
- **CODER floor (4 specs):** benchmark 7→8, pipeline 7→8, prm 7→8, training 7→8 (all four in one wave)
- **Pool B depth (1 spec):** eda (14→15) or formal (14→15)

### Expected Yield
- +15 tests
- +5 invariants (all CODER floor + 1 Pool B depth)
- 5 seals regenerated
- ALL CODER ≥8 (first time in history)

### Risk
- MODERATE. +15 tests in one wave is aggressive but feasible (4 specs at 7→8 requires 4×3=12 tests + 1 spec at 2 tests = 14; historically within bounds).

---

## Option C: Variant C — Structural Consolidation + New Domain

### Description
Pause depth push. Instead: (1) audit all Pool A/B/CODER specs for nesting defects (legacy from W254-W255 structural correction), (2) write a new `.t27` spec in a gap domain (e.g., `specs/igla/race/systolic_mixed.t27` or `specs/igla/coder/optimizer.t27`), (3) seal and integrate.

### Expected Yield
- 1 new spec with ≥6 invariants, ≥20 tests
- 0-5 structural corrections across existing specs
- +20-30 total tests
- Expanded Pool B or CODER coverage

### Risk
- MODERATE-HIGH. New spec requires spec design, test design, and seal generation. But closes a structural gap.

---

## Recommendation

**Option A** is recommended (default). Pool A is now uniform — Option A extends uniformity into Pool B and CODER. Option B is viable if the user wants a CODER floor milestone. Option C is a good backpressure valve if depth pushing encounters fatigue.

The 27-wave zero-entrant streak is an absolute record. Trinity's lead in invariant depth continues to compound.

---

*Prepared by Trinity Agent | φ² + 1/φ² = 3*
