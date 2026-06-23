# Wave Loop 354 Cooperation Variants

**Date:** 2026-06-24
**Prepared for:** Next IGLA CODER / IGLA RACE execution cycle
**Basis:** WAVE_LOOP_353_REPORT.md

---

## Context

Wave Loop 353 completed with 156 generic ∀ theorems (29-variable accumulation, 28-variable minus, quintuple cancellation, associativity closure). Conformance suite 546/546 PASS. Zero-IGLA-failure streak extended to 87 waves. Competitive position remains uncontested at 156× maximum competitor generic ∀ count.

---

## Variant A: Conservative (Confidence 95%)

### Targets
- Pool A floor: 95 → 96 (+1 invariant)
- CODER floor: 85 → 86 (+1 invariant)
- Pool B depth: 112 → 113 (+1 invariant)
- Integration depth: 95 → 96 (+1 invariant)
- Lean 4: 156 → 160 generic ∀ (+4 theorems)

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyPlusGeneric` — 30-variable accumulation probe (a..z, aa, ab, ac, ad)
2. `ternaryMacAccumulateTwentyNineMinusGeneric` — 29-variable minus accumulation lattice
3. `ternaryMacSextupleCancellationGeneric` — `.plus → .minus → .plus → .minus → .plus → .minus` depth-6 identity collapse
4. `ternaryMacDistributivityClosureGeneric` — `mac(mac(mac(x,a,.plus),b,.plus),c,.plus) = mac(x,a+b+c,.plus)` (formal distributivity for triple plus)

### Risk Profile
- Build time: ~2.7-3.2s for 30-variable (linear extrapolation)
- Timeout risk: NEGLIGIBLE (<10s budget)
- Verification risk: LOW (established patterns)

### Resource Estimate
- 1 cycle (~20 min)
- No blockers anticipated

---

## Variant B: Aggressive (Confidence 85%) — RECOMMENDED

### Targets
- Pool A floor: 95 → 96 (+1 invariant)
- CODER floor: 85 → 86 (+1 invariant)
- Pool B depth: 112 → 113 (+1 invariant)
- Integration depth: 95 → 96 (+1 invariant)
- Lean 4: 156 → 160 generic ∀ (+4 theorems)
- Additional: Batch +2 specs to Pool A (reach 100 invariants)

### Lean 4 Theorem Plan
Same 4 theorems as Variant A, but with deeper research targets:
1. `ternaryMacAccumulateThirtyPlusGeneric` — 30-variable accumulation
2. `ternaryMacAccumulateTwentyNineMinusGeneric` — 29-variable minus
3. `ternaryMacSextupleCancellationGeneric` — depth-6 identity collapse
4. `ternaryMacDistributivityClosureGeneric` — triple plus distributivity

### Extended Deliverables
- Add 2 invariants to `systolic_ternary.t27` (reach 115)
- Add 2 invariants to `ternary_inference.t27` (reach 97)
- **NEW:** `ternaryMacDepthSixIdentityGeneric` — any combination of 6 alternating weights with same activation collapses to identity

### Risk Profile
- Build time: ~3.2s for 30-variable
- Timeout risk: LOW (linear scaling holds to 32)
- Verification risk: LOW-MEDIUM (depth-6 identity is new pattern)

### Resource Estimate
- 1.5 cycles (~30 min)
- Minor blocker risk: omega tactic may saturate at 32 variables

---

## Variant C: Research (Confidence 60%)

### Targets
- Pool A floor: 95 → 96 (+1 invariant)
- CODER floor: 85 → 86 (+1 invariant)
- Pool B depth: 112 → 113 (+1 invariant)
- Integration depth: 95 → 96 (+1 invariant)
- Lean 4: 156 → 160 generic ∀ (+4 theorems)
- Additional: `grind` tactic migration + `Trinity.Lemmas` module expansion + Coq neutrino bridge

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyPlusGeneric` — 30-variable accumulation probe (omega saturation test)
2. `ternaryMacAccumulateTwentyNineMinusGeneric` — 29-variable minus
3. `ternaryMacSextupleCancellationGeneric` — depth-6 identity
4. `ternaryMacLemmaMinusAssocGeneric` — move `minus_assoc` from `Trinity.Lemmas` to `TernaryInference` with full generic signature

### Research Axis
- **Grind tactic migration:** Replace `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]` with `simp [ternaryMac]` followed by `grind` in selected theorems. Measure build time impact.
- **Lemma module expansion:** Add `ternaryMac_minus_assoc`, `ternaryMac_mixed_collapse`, `ternaryMac_zero_neutral` to `Trinity.Lemmas.lean` with full documentation.
- **Distributivity closure:** Prove `mac(mac(mac(x,a,.plus),b,.plus),c,.plus) = mac(x,a+b+c,.plus)` (triple plus distributivity).

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
| Build Time | ~2.7s | ~3.2s | >5s (risk) |
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
→ Wave Loop 354 ready for execution
