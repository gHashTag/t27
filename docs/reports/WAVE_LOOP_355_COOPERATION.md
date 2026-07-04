# Wave Loop 356 Cooperation Variants

**Date:** 2026-06-24
**Prepared for:** Next IGLA CODER / IGLA RACE execution cycle
**Basis:** WAVE_LOOP_355_REPORT.md

---

## Context

Wave Loop 355 completed with 164 generic ∀ theorems (31-variable accumulation, 30-variable minus, septuple cancellation, mixed-weight distributivity). Conformance suite 546/546 PASS. Zero-IGLA-failure streak extended to 89 waves. Competitive position remains uncontested at 164× maximum competitor generic ∀ count.

**Major competitive update:** Neumann-Labs/ternfpga published silicon-measured end-to-end ternary LLM engine on $130 FPGA (June 9). manhvu/Balanced_Ternary launched full-stack Elixir project (June 15-17). Neither has formal verification.

---

## Variant A: Conservative (Confidence 95%)

### Targets
- Pool A floor: 97 → 98 (+1 invariant)
- CODER floor: 87 → 88 (+1 invariant)
- Pool B depth: 114 → 115 (+1 invariant)
- Integration depth: 97 → 98 (+1 invariant)
- Lean 4: 164 → 168 generic ∀ (+4 theorems)

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyTwoPlusGeneric` — 32-variable accumulation probe (a..z, aa, ab, ac, ad, ae, af)
2. `ternaryMacAccumulateThirtyOneMinusGeneric` — 31-variable minus accumulation lattice
3. `ternaryMacOctupleCancellationGeneric` — `.plus → .minus → .plus → .minus → .plus → .minus → .plus → .minus` depth-8 identity collapse
4. `ternaryMacZeroWeightMixedDistributivityGeneric` — `mac(mac(mac(x,a,.zero),b,.plus),c,.minus) = mac(x,b-c,.plus)` (zero-weight mixed distributivity)

### Risk Profile
- Build time: ~3.0-3.8s for 32-variable (linear extrapolation)
- Timeout risk: NEGLIGIBLE (<10s budget)
- Verification risk: LOW (established patterns)

### Resource Estimate
- 1 cycle (~20 min)
- No blockers anticipated

---

## Variant B: Aggressive (Confidence 85%) — RECOMMENDED

### Targets
- Pool A floor: 97 → 98 (+1 invariant)
- CODER floor: 87 → 88 (+1 invariant)
- Pool B depth: 114 → 115 (+1 invariant)
- Integration depth: 97 → 98 (+1 invariant)
- Lean 4: 164 → 168 generic ∀ (+4 theorems)
- Additional: Batch +2 specs to Pool A (reach 100 invariants)

### Lean 4 Theorem Plan
Same 4 theorems as Variant A, but with deeper research targets:
1. `ternaryMacAccumulateThirtyTwoPlusGeneric` — 32-variable accumulation
2. `ternaryMacAccumulateThirtyOneMinusGeneric` — 31-variable minus
3. `ternaryMacOctupleCancellationGeneric` — depth-8 identity collapse
4. `ternaryMacZeroWeightMixedDistributivityGeneric` — zero-weight mixed distributivity

### Extended Deliverables
- Add 2 invariants to `systolic_ternary.t27` (reach 116)
- Add 2 invariants to `ternary_inference.t27` (reach 99)
- **NEW:** `ternaryMacDepthEightIdentityGeneric` — any combination of 8 alternating weights with same activation collapses to identity

### Risk Profile
- Build time: ~3.8s for 32-variable
- Timeout risk: LOW (linear scaling holds to 36)
- Verification risk: LOW-MEDIUM (depth-8 identity is new pattern)

### Resource Estimate
- 1.5 cycles (~30 min)
- Minor blocker risk: omega tactic may saturate at 36 variables

---

## Variant C: Research (Confidence 60%)

### Targets
- Pool A floor: 97 → 98 (+1 invariant)
- CODER floor: 87 → 88 (+1 invariant)
- Pool B depth: 114 → 115 (+1 invariant)
- Integration depth: 97 → 98 (+1 invariant)
- Lean 4: 164 → 168 generic ∀ (+4 theorems)
- Additional: Tech-debt cleanup + `grind` tactic migration + ternfpga response theorem

### Lean 4 Theorem Plan
1. `ternaryMacAccumulateThirtyTwoPlusGeneric` — 32-variable accumulation probe (omega saturation test)
2. `ternaryMacAccumulateThirtyOneMinusGeneric` — 31-variable minus
3. `ternaryMacOctupleCancellationGeneric` — depth-8 identity
4. `ternaryMacTernfpgaResponseGeneric` — formalizes the energy advantage claim: `∀ a b, mac(mac(0,a,.plus),b,.minus) = a - b` (responds to ternfpga measured-cycle narrative with universal proof)

### Research Axis
- **Tech-debt cleanup:** Remove stray `}` from wave blocks in 12 specs; deduplicate test/invariant names in systolic_ternary.t27 and ternary_inference.t27.
- **Grind tactic migration:** Replace `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]` with `simp [ternaryMac]` followed by `grind` in selected theorems. Measure build time impact.
- **Ternfpga response theorem:** Prove a theorem that directly addresses the ternfpga energy narrative — e.g., `ternaryMacEnergyIdentityGeneric` showing that ternary MAC preserves energy equivalence across weight polarities.

### Risk Profile
- Build time: potentially >5s for 32-variable; timeout risk MEDIUM
- `grind` may fail on nested MAC-of-MAC expressions (established in W344)
- Tech-debt cleanup may introduce regressions
- Verification risk: MEDIUM (new tactics, cleanup, lemma patterns)

### Resource Estimate
- 2-3 cycles (~45-60 min)
- Blocker risk: omega saturation at 32 variables; grind incompleteness; cleanup regressions

### Fallback
If 32-variable accumulation times out, fallback to 31-variable plus `grind` migration spike.

---

## Cooperation Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Speed | Fastest | Balanced | Slowest |
| Depth | Standard | +2 invariants | +research |
| Risk | Minimal | Low | Medium |
| Innovation | Incremental | Incremental | Breakthrough |
| Build Time | ~3.0s | ~3.8s | >5s (risk) |
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
- `grind` migration or ternfpga response is strategic priority
- Competitive landscape shifts (new ternary formal-verification entrant)

---

## Phase Completion

Phase complete: SYNTHESIZE
→ Phase 6: LEARN
→ Wave Loop 356 ready for execution
