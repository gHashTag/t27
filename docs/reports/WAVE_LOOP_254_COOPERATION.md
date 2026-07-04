# Wave Loop 254 — Cooperation Variants for Wave Loop 255

**Date:** June 16, 2026  
**Prepared for:** W255 planning  
**Current state:** 231 competitors stable, 570/570 PASS, Pool A ≥12, Pool B ≥12, CODER ≥6 (training 7)

---

## Overview

Three strategic variants are proposed for Wave Loop 255. Each variant trades off differently between RACE pool depth, CODER floor raising, and external engagement. The recommended variant is **Variant A** (Submit+Resume) to continue the proven depth-push strategy.

---

## Variant A — Submit+Resume (Recommended)

**Priority:** Raise remaining Pool A specs to ≥13 and CODER floor to ≥7.

### Execution Plan
- **Pool A:** Select 2 specs at minimum 12 (cordic_fixed 12 W253, gemm 12 W250) → +2 tests each, +1 invariant each → raise to 13
- **Pool B:** Select 2 specs at minimum 12 (systolic_ternary 12 W249, backend 12 W253) → +2 tests each, +1 invariant each → maintain 12–13
- **CODER:** Select 1 spec at minimum 6 (eval 6 W249, oldest) → +3 tests, +1 invariant → raise to 7
- **Total:** +11 tests, +5 invariants

### Rationale
This continues the canonical IGLA CODER+RACE cadence. Raising Pool A from 12 to 13 on cordic_fixed and gemm leaves only cordic_top at 12. CODER eval is the oldest at 6 (W249). systolic_ternary and backend maintain Pool B depth.

### Risk
- LOW: Proven pattern; no architectural changes.

---

## Variant B — Pool A Critical Floor Raise

**Priority:** Achieve uniform Pool A ≥13 in one wave.

### Execution Plan
- **Pool A:** Select 3 specs at 12 (cordic_fixed, cordic_top, gemm) → +1 test each, +1 invariant each → all reach 13
- **Pool B:** Select 1 spec at 12 (systolic_ternary 12) → +2 tests, +1 invariant → maintain 12
- **CODER:** Select 1 spec at 6 (benchmark 6 W251) → +3 tests, +1 invariant → raise to 7
- **Total:** +10 tests, +6 invariants

### Rationale
This variant front-loads Pool A to eliminate all 12-invariant specs in one wave, achieving complete Pool A uniformity at 13. It sacrifices the canonical +11 test target for a concentrated invariant raise. Best used when Pool A uniformity is judged more valuable than test-count growth.

### Risk
- LOW-MEDIUM: Deviation from +11 test norm; +6 invariants is above canonical +5.

---

## Variant C — CODER Deep Push + Outreach

**Priority:** Accelerate CODER toward ≥8 invariants on a single spec while raising Pool A minimum and initiating external collaboration.

### Execution Plan
- **Pool A:** Select 2 specs at 12 (cordic_fixed 12, cordic_top 12) → +2 tests each, +1 invariant each → raise to 13
- **Pool B:** Select 1 spec at 12 (yosys 12 W250) → +2 tests, +1 invariant → maintain 12
- **CODER:** Select 1 spec (bench_proxy 8 W252) → +6 tests, +3 invariants → push to 11
- **Outreach:** Draft collaboration proposal to Sparkle HDL maintainers for joint formal verification lemma exchange on systolic arrays
- **Total:** +12 tests, +6 invariants

### Rationale
This variant breaks the canonical +5 invariant cadence to demonstrate CODER can sustain double-digit invariants. The outreach component leverages the 21-wave stability to engage a MEDIUM-HIGH threat (Sparkle HDL) constructively. Best used when external ecosystem building is prioritized over pure internal depth metrics.

### Risk
- MEDIUM: Higher invariant count per spec increases verification time; Sparkle HDL engagement may not yield response.

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

**Recommendation:** Execute **Variant A** for W255. It preserves the Submit+Resume cadence that has sustained the 21-wave zero-entrant streak, raises the Pool A floor on the oldest specs, and continues the CODER climb. Variant C should be held in reserve for W256 or W257 when external conditions favor outreach.

---

*Prepared: 2026-06-16 | phi² + 1/φ² = 3 | TRINITY*
