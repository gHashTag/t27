# Wave Loop 353 Cooperation Variants

**Date:** 2026-06-24
**Prepared for:** Next IGLA CODER / IGLA RACE execution cycle
**Basis:** WAVE_LOOP_352_REPORT.md

---

## Context

Wave Loop 352 completed with 152 generic ∀ theorems (28-variable accumulation, 27-variable minus, quadruple cancellation, generalized commutativity). Conformance suite 546/546 PASS. Zero-IGLA-failure streak extended to 86 waves. Competitive position remains uncontested at 152× maximum competitor generic ∀ count.

---

## Variant A: Conservative (Confidence 95%)

### Targets
- Pool A floor: 94 → 95 (+1 invariant)
- CODER floor: 84 → 85 (+1 invariant)
- Pool B depth: 111 → 112 (+1 invariant)
- Integration depth: 94 → 95 (+1 invariant)
- Lean 4: 152 → 156 generic ∀ (+4 theorems)

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateTwentyNinePlusGeneric` — 29-variable accumulation probe (a..z, aa, ab, ac)
2. `ternaryMacAccumulateTwentyEightMinusGeneric` — 28-variable minus accumulation lattice
3. `ternaryMacQuintupleCancellationGeneric` — `.plus → .minus → .plus → .minus → .plus` depth-5 identity collapse
4. `ternaryMacAssociativityClosureGeneric` — `mac(mac(mac(0,a,.plus),b,.plus),c,.plus) = mac(0,a+b+c,.plus)` (formal associativity for triple plus)

### Risk Profile
- Build time: ~2.5-3.0s for 29-variable (linear extrapolation)
- Timeout risk: NEGLIGIBLE (<10s budget)
- Verification risk: LOW (established patterns)

### Resource Estimate
- 1 cycle (~20 min)
- No blockers anticipated

---

## Variant B: Aggressive (Confidence 85%) — RECOMMENDED

### Targets
- Pool A floor: 94 → 95 (+1 invariant)
- CODER floor: 84 → 85 (+1 invariant)
- Pool B depth: 111 → 112 (+1 invariant)
- Integration depth: 94 → 95 (+1 invariant)
- Lean 4: 152 → 156 generic ∀ (+4 theorems)
- Additional: Batch +2 specs to Pool A (reach 97 invariants)

### Lean 4 Theorem Plan
Same 4 theorems as Variant A, but with deeper research targets:
1. `ternaryMacAccumulateTwentyNinePlusGeneric` — 29-variable accumulation
2. `ternaryMacAccumulateTwentyEightMinusGeneric` — 28-variable minus
3. `ternaryMacQuintupleCancellationGeneric` — depth-5 identity collapse
4. `ternaryMacAssociativityClosureGeneric` — triple plus associativity

### Extended Deliverables
- Add 2 invariants to `systolic_ternary.t27` (reach 113)
- Add 2 invariants to `ternary_inference.t27` (reach 96)
- **NEW:** `ternaryMacDepthFiveIdentityGeneric` — any combination of 5 alternating weights with same activation collapses to identity

### Risk Profile
- Build time: ~3.0s for 29-variable
- Timeout risk: LOW (linear scaling holds to 30)
- Verification risk: LOW-MEDIUM (depth-5 identity is new pattern)

### Resource Estimate
- 1.5 cycles (~30 min)
- Minor blocker risk: omega tactic may saturate at 30 variables

---

## Variant C: Research (Confidence 60%)

### Targets
- Pool A floor: 94 → 95 (+1 invariant)
- CODER floor: 84 → 85 (+1 invariant)
- Pool B depth: 111 → 112 (+1 invariant)
- Integration depth: 94 → 95 (+1 invariant)
- Lean 4: 152 → 156 generic ∀ (+4 theorems)
- Additional: `grind` tactic migration + `Trinity.Lemmas` module expansion

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyPlusGeneric` — 30-variable accumulation probe (omega saturation test)
2. `ternaryMacAccumulateTwentyNineMinusGeneric` — 29-variable minus
3. `ternaryMacQuintupleCancellationGeneric` — depth-5 identity
4. `ternaryMacLemmaPlusAssocGeneric` — move `plus_assoc` from `Trinity.Lemmas` to `TernaryInference` with full generic signature

### Research Axis
- **Grind tactic migration:** Replace `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]` with `simp [ternaryMac]` followed by `grind` in selected theorems. Measure build time impact.
- **Lemma module expansion:** Add `ternaryMac_minus_assoc`, `ternaryMac_mixed_collapse`, `ternaryMac_zero_neutral` to `Trinity.Lemmas.lean` with full documentation.
- **Composition closure:** Prove `mac(mac(mac(0,a,.plus),b,.minus),c,.plus) = mac(0,a-b+c,.plus)` (mixed-weight triple associativity).

### Risk Profile
- Build time: potentially >5s for 30-variable; timeout risk MEDIUM
- `grind` may fail on nested MAC-of-MAC expressions (established in W344)
- Verification risk: MEDIUM (new tactics, new lemma patterns)

### Resource Estimate
- 2-3 cycles (~45-60 min)
- Blocker risk: omega saturation at 30 variables; grind incompleteness

### Fallback
If 30-variable accumulation times out, fallback to 29-variable plus `grind` migration spike.

---

## Cooperation Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Speed | Fastest | Balanced | Slowest |
| Depth | Standard | +2 invariants | +research |
| Risk | Minimal | Low | Medium |
| Innovation | Incremental | Incremental | Breakthrough |
| Build Time | ~2.5s | ~3.0s | >5s (risk) |
| Generic ∀ Δ | +4 | +4 | +4 |
| Recommended | No | **YES** | Experimental |

---

## Decision Criteria

Choose **Variant B** if:
- Zero-IGLA-failure streak must be maintained (>95% confidence)
- Need balanced progress on depth and breadth
- No appetite for timeout/debug risk

Choose **Variant C** if:
- Willing to accept 1 timeout/debug cycle for breakthrough capability
- `grind` migration is strategic priority
- Competitive landscape shifts (new ternary formal-verification entrant)

---

## Phase Completion

Phase complete: SYNTHESIZE
→ Phase 6: LEARN
→ Wave Loop 353 ready for execution
