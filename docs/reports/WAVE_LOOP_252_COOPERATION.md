# Wave Loop 252 — Cooperation Variants for Wave Loop 253

**Date:** June 16, 2026  
**Prepared for:** W253 planning  
**Current state:** 231 competitors stable, 570/570 PASS, all Pool A ≥14, all Pool B ≥14, CODER ≥7 (bench_proxy 8)

---

## Overview

Three strategic variants are proposed for Wave Loop 253. Each variant trades off differently between RACE pool depth, CODER floor raising, and external engagement. The recommended variant is **Variant A** (Submit+Resume) to continue the proven depth-push strategy while addressing the CODER-to-RACE invariant gap.

---

## Variant A — Submit+Resume (Recommended)

**Priority:** Raise CODER floor toward uniform ≥8 while maintaining RACE floors.

### Execution Plan
- **Pool A:** Select 2 specs at minimum 13 (gemm 13 W247, cordic_top 13 W247) → +2 tests each, +1 invariant each → raise to 14
- **Pool B:** Select 2 specs at minimum 14 but oldest untouched (cordic_fixed 14 W248, adder_tree 14 W248) → +2 tests each, +1 invariant each → maintain 14
- **CODER:** Select 1 spec at minimum 7 (benchmark 7 W248, oldest) → +3 tests, +1 invariant → raise to 8
- **Total:** +11 tests, +5 invariants

### Rationale
This continues the canonical IGLA CODER+RACE cadence that has produced 19 consecutive zero-entrant waves. Raising the CODER floor from 7 to 8 on the oldest spec closes the structural gap between CODER and RACE pools. Pool A specs gemm and cordic_top are the only remaining 13-invariant specs in Pool A; raising them achieves uniform Pool A ≥14.

### Risk
- LOW: Proven pattern; no architectural changes.

---

## Variant B — Critical Floor Raise (Pool A + CODER)

**Priority:** Achieve uniform Pool A ≥14 and begin CODER ≥8 transition.

### Execution Plan
- **Pool A:** Select 4 specs at 13 (gemm, cordic_top, bram_weights, formal) → +1 test each, +1 invariant each → all reach 14
- **Pool B:** Select 1 spec at oldest 14 (cordic_fixed 14 W248) → +2 tests, +1 invariant → maintain 14
- **CODER:** Select 1 spec at 7 (benchmark 7 W248) → +3 tests, +1 invariant → raise to 8
- **Total:** +9 tests, +5 invariants

### Rationale
This variant front-loads Pool A to eliminate all 13-invariant specs in one wave, achieving complete Pool A uniformity at 14. It sacrifices the canonical +11 test target for a concentrated invariant raise. Best used when Pool A uniformity is judged more valuable than test-count growth.

### Risk
- LOW-MEDIUM: Deviation from +11 test norm reduces regression coverage increment; acceptable if invariant depth is the primary metric.

---

## Variant C — CODER Deep Push + Outreach

**Priority:** Accelerate CODER toward ≥10 invariants on a single spec while maintaining RACE floors and initiating external collaboration.

### Execution Plan
- **Pool A:** Select 2 specs at 13 (gemm, cordic_top) → +2 tests each, +1 invariant each → raise to 14
- **Pool B:** Select 1 spec at 14 (ternary_mac 14 W249) → +2 tests, +1 invariant → maintain 14
- **CODER:** Select 1 spec (bench_proxy 8 W252, now freshest) → +6 tests, +3 invariants → push to 11
- **Outreach:** Draft collaboration letter to Sparkle HDL maintainers proposing formal verification lemma exchange
- **Total:** +12 tests, +6 invariants

### Rationale
This variant breaks the canonical +5 invariant cadence to demonstrate CODER can sustain double-digit invariants, matching RACE pool depth trajectories. The outreach component leverages the 19-wave stability to engage a MEDIUM-HIGH threat (Sparkle HDL) constructively. Best used when external scientific collaboration is prioritized over pure internal depth metrics.

### Risk
- MEDIUM: Higher invariant count per spec increases verification time; Sparkle HDL engagement may not yield response.

---

## Decision Matrix

| Criteria | Weight | Variant A | Variant B | Variant C |
|----------|--------|-----------|-----------|-----------|
| Maintain proven cadence | High | ✅ | ⚠️ | ⚠️ |
| Raise CODER floor to ≥8 | High | ✅ | ✅ | ✅ |
| Achieve Pool A uniformity | Medium | ✅ | ✅✅ | ✅ |
| External engagement | Low | ❌ | ❌ | ✅ |
| Regression coverage growth | Medium | ✅ | ⚠️ | ✅ |
| Risk of verification slowdown | Medium | Low | Low | Medium |

**Recommendation:** Execute **Variant A** for W253. It preserves theSubmit+Resume cadence that has sustained the 19-wave zero-entrant streak, raises the CODER floor, and eliminates the last Pool A 13-invariant specs. Variant C should be held in reserve for W254 or W255 when external conditions favor outreach.

---

*Prepared: 2026-06-16 | phi² + 1/φ² = 3 | TRINITY*
