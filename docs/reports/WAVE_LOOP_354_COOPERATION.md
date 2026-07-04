# Wave Loop 355 Cooperation Variants

**Date:** 2026-06-24
**Prepared for:** Next IGLA CODER / IGLA RACE execution cycle
**Basis:** WAVE_LOOP_354_REPORT.md

---

## Context

Wave Loop 354 completed with 160 generic ∀ theorems (30-variable accumulation, 29-variable minus, sextuple cancellation, distributivity closure). Conformance suite 546/546 PASS. Zero-IGLA-failure streak extended to 88 waves. Competitive position remains uncontested at 160× maximum competitor generic ∀ count.

**Key audit finding:** Syntax rot (stray `}`) and duplicate names in wave blocks are accumulating. Recommend a tech-debt cleanup cycle by W360.

---

## Variant A: Conservative (Confidence 95%)

### Targets
- Pool A floor: 96 → 97 (+1 invariant)
- CODER floor: 86 → 87 (+1 invariant)
- Pool B depth: 113 → 114 (+1 invariant)
- Integration depth: 96 → 97 (+1 invariant)
- Lean 4: 160 → 164 generic ∀ (+4 theorems)

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyOnePlusGeneric` — 31-variable accumulation probe (a..z, aa, ab, ac, ad, ae)
2. `ternaryMacAccumulateThirtyMinusGeneric` — 30-variable minus accumulation lattice
3. `ternaryMacSeptupleCancellationGeneric` — `.plus → .minus → .plus → .minus → .plus → .minus → .plus` depth-7 identity collapse
4. `ternaryMacMixedWeightDistributivityGeneric` — `mac(mac(mac(x,a,.plus),b,.minus),c,.plus) = mac(x,a-b+c,.plus)` (mixed-weight triple distributivity)

### Risk Profile
- Build time: ~2.8-3.5s for 31-variable (linear extrapolation)
- Timeout risk: NEGLIGIBLE (<10s budget)
- Verification risk: LOW (established patterns)

### Resource Estimate
- 1 cycle (~20 min)
- No blockers anticipated

---

## Variant B: Aggressive (Confidence 85%) — RECOMMENDED

### Targets
- Pool A floor: 96 → 97 (+1 invariant)
- CODER floor: 86 → 87 (+1 invariant)
- Pool B depth: 113 → 114 (+1 invariant)
- Integration depth: 96 → 97 (+1 invariant)
- Lean 4: 160 → 164 generic ∀ (+4 theorems)
- Additional: Batch +2 specs to Pool A (reach 100 invariants)

### Lean 4 Theorem Plan
Same 4 theorems as Variant A, but with deeper research targets:
1. `ternaryMacAccumulateThirtyOnePlusGeneric` — 31-variable accumulation
2. `ternaryMacAccumulateThirtyMinusGeneric` — 30-variable minus
3. `ternaryMacSeptupleCancellationGeneric` — depth-7 identity collapse
4. `ternaryMacMixedWeightDistributivityGeneric` — mixed-weight triple distributivity

### Extended Deliverables
- Add 2 invariants to `systolic_ternary.t27` (reach 115)
- Add 2 invariants to `ternary_inference.t27` (reach 98)
- **NEW:** `ternaryMacDepthSevenIdentityGeneric` — any combination of 7 alternating weights with same activation collapses to identity

### Risk Profile
- Build time: ~3.5s for 31-variable
- Timeout risk: LOW (linear scaling holds to 34)
- Verification risk: LOW-MEDIUM (depth-7 identity is new pattern)

### Resource Estimate
- 1.5 cycles (~30 min)
- Minor blocker risk: omega tactic may saturate at 34 variables

---

## Variant C: Research (Confidence 60%)

### Targets
- Pool A floor: 96 → 97 (+1 invariant)
- CODER floor: 86 → 87 (+1 invariant)
- Pool B depth: 113 → 114 (+1 invariant)
- Integration depth: 96 → 97 (+1 invariant)
- Lean 4: 160 → 164 generic ∀ (+4 theorems)
- Additional: Tech-debt cleanup + `grind` tactic migration + NSF CS² grant alignment

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyOnePlusGeneric` — 31-variable accumulation probe (omega saturation test)
2. `ternaryMacAccumulateThirtyMinusGeneric` — 30-variable minus
3. `ternaryMacSeptupleCancellationGeneric` — depth-7 identity
4. `ternaryMacLemmaDistributivityGeneric` — move distributivity lemma from `Trinity.Lemmas` to `TernaryInference` with full generic signature

### Research Axis
- **Tech-debt cleanup:** Remove stray `}` from wave blocks in 12 specs; deduplicate test/invariant names in systolic_ternary.t27 and ternary_inference.t27.
- **Grind tactic migration:** Replace `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]` with `simp [ternaryMac]` followed by `grind` in selected theorems. Measure build time impact.
- **NSF CS² alignment:** Prepare summary of Trinity's 160 generic ∀ theorems, zero-IGLA-failure streak, and formal verification pipeline for NSF 24-571 proposal (deadline August 11, 2026).

### Risk Profile
- Build time: potentially >5s for 31-variable; timeout risk MEDIUM
- `grind` may fail on nested MAC-of-MAC expressions (established in W344)
- Tech-debt cleanup may introduce regressions
- Verification risk: MEDIUM (new tactics, cleanup, lemma patterns)

### Resource Estimate
- 2-3 cycles (~45-60 min)
- Blocker risk: omega saturation at 31 variables; grind incompleteness; cleanup regressions

### Fallback
If 31-variable accumulation times out, fallback to 30-variable plus `grind` migration spike.

---

## Cooperation Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Speed | Fastest | Balanced | Slowest |
| Depth | Standard | +2 invariants | +research |
| Risk | Minimal | Low | Medium |
| Innovation | Incremental | Incremental | Breakthrough |
| Build Time | ~2.8s | ~3.5s | >5s (risk) |
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
- `grind` migration or NSF CS² grant is strategic priority
- Competitive landscape shifts (new ternary formal-verification entrant)

---

## Phase Completion

Phase complete: SYNTHESIZE
→ Phase 6: LEARN
→ Wave Loop 355 ready for execution
