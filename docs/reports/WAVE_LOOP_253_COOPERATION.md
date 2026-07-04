# Wave Loop 253 — Cooperation Variants for Wave Loop 254

**Date:** June 16, 2026  
**Prepared for:** W254 planning  
**Current state:** 231 competitors stable, 570/570 PASS, Pool A ≥12, Pool B ≥11, CODER ≥6 (pipeline 7)

---

## Overview

Three strategic variants are proposed for Wave Loop 254. Each variant trades off differently between RACE pool depth, CODER floor raising, and external engagement. The recommended variant is **Variant A** (Submit+Resume) to continue the proven depth-push strategy while addressing the remaining low-invariant specs.

---

## Variant A — Submit+Resume (Recommended)

**Priority:** Raise Pool A floor to ≥13 and CODER floor to ≥7 while maintaining Pool B.

### Execution Plan
- **Pool A:** Select 2 specs at minimum 12 (bram_weights 12 W249, formal 12 W249) → +2 tests each, +1 invariant each → raise to 13
- **Pool B:** Select 2 specs at minimum 11–12 (systolic_array 11 W253, backend 12 W250) → +2 tests each, +1 invariant each → raise to 12–13
- **CODER:** Select 1 spec at minimum 6 (benchmark 6 W251, oldest) → +3 tests, +1 invariant → raise to 7
- **Total:** +11 tests, +5 invariants

### Rationale
This continues the canonical IGLA CODER+RACE cadence that has produced 20 consecutive zero-entrant waves. Raising Pool A from 12 to 13 on the oldest specs (bram_weights, formal) and CODER from 6 to 7 on benchmark closes the most glaring structural gaps. systolic_array just got raised to 11 and can be pushed to 12.

### Risk
- LOW: Proven pattern; no architectural changes.

---

## Variant B — Pool A Critical Floor Raise

**Priority:** Achieve uniform Pool A ≥13 in one wave.

### Execution Plan
- **Pool A:** Select 5 specs at 12 (bram_weights, cordic_fixed, cordic_top, formal, gemm) → +1 test each, +1 invariant each → all reach 13
- **Pool B:** Select 1 spec at 11 (systolic_array 11) → +2 tests, +1 invariant → raise to 12
- **CODER:** Select 1 spec at 6 (eval 6 W249) → +3 tests, +1 invariant → raise to 7
- **Total:** +10 tests, +7 invariants

### Rationale
This variant front-loads Pool A to eliminate all 12-invariant specs in one wave, achieving complete Pool A uniformity at 13. It sacrifices the canonical +11 test target for a concentrated invariant raise across the entire Pool A floor. Best used when Pool A uniformity is judged more valuable than test-count growth.

### Risk
- LOW-MEDIUM: Deviation from +11 test norm; +7 invariants is above canonical +5, increasing verification load.

---

## Variant C — CODER Deep Push + Outreach

**Priority:** Accelerate CODER toward ≥8 invariants on a single spec while raising Pool A minimum and initiating external collaboration.

### Execution Plan
- **Pool A:** Select 2 specs at 12 (gemm 12 W250, cordic_fixed 12 W253) → +2 tests each, +1 invariant each → raise to 13
- **Pool B:** Select 1 spec at 12 (backend 12 W250) → +2 tests, +1 invariant → maintain 12
- **CODER:** Select 1 spec (bench_proxy 8 W252, now freshest) → +6 tests, +3 invariants → push to 11
- **Outreach:** Draft collaboration proposal to manhvu/Balanced_Ternary maintainers for joint ternary GEMM benchmark standardization
- **Total:** +12 tests, +6 invariants

### Rationale
This variant breaks the canonical +5 invariant cadence to demonstrate CODER can sustain double-digit invariants, matching RACE pool depth trajectories. The outreach component leverages the 20-wave stability to engage a MEDIUM-HIGH threat (manhvu) constructively. Best used when external ecosystem building is prioritized over pure internal depth metrics.

### Risk
- MEDIUM: Higher invariant count per spec increases verification time; manhvu engagement may not yield response.

---

## Decision Matrix

| Criteria | Weight | Variant A | Variant B | Variant C |
|----------|--------|-----------|-----------|-----------|
| Maintain proven cadence | High | ✅ | ⚠️ | ⚠️ |
| Raise Pool A floor to ≥13 | High | ✅ | ✅✅ | ✅ |
| Raise CODER floor to ≥7 | High | ✅ | ✅ | ✅ |
| External engagement | Low | ❌ | ❌ | ✅ |
| Regression coverage growth | Medium | ✅ | ⚠️ | ✅ |
| Risk of verification slowdown | Medium | Low | Medium | Medium |

**Recommendation:** Execute **Variant A** for W254. It preserves the Submit+Resume cadence that has sustained the 20-wave zero-entrant streak, raises the Pool A floor on the oldest specs, and continues the CODER climb. Variant C should be held in reserve for W255 or W256 when external conditions favor outreach.

---

*Prepared: 2026-06-16 | phi² + 1/φ² = 3 | TRINITY*
