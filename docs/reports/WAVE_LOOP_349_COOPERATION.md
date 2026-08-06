# Wave Loop 349 Cooperation Variants -- For Wave Loop 350

**Date:** 2026-06-23
**Target:** Extend W349 achievements into W350 with three strategic variants

---

## Context

W349 achieved:
- Pool A floor 90→91, CODER floor 80→81
- Pool B depth 107→108, Integration depth 90→91
- Lean 4: **132 generic ∀** (25-variable plus, 24-variable minus, distributivity, zero-weight idempotence)
- Trinity.Lemmas module validated: `ternaryMac_plus_assoc` enables one-liner distributivity proofs
- 83 consecutive zero-IGLA-failure waves
- `simp+omega` build time: 2.2s for 25 variables

W350 must decide between continuing depth-first, leveraging lemma library for lattice expansion, or a balanced hybrid.

---

## Variant A: Conservative Depth Extension

**Risk profile:** LOW | **Complexity:** LOW | **Expected outcome:** Predictable

### Targets

| Metric | W349 | W350 |
|--------|------|------|
| Pool A Floor | 91 | 92 |
| CODER Floor | 81 | 82 |
| Pool B Depth | 108 | 109 |
| Integration Depth | 91 | 92 |
| Lean 4 Generic ∀ | 132 | **135** |
| Accumulation Probe | 25-var plus | **26-var plus** |
| Minus Accumulation | 24-var minus | **25-var minus** |
| New Dimension | distributivity | **lemma-driven 26-var** |

### New Lean 4 Theorems (3)

1. `ternaryMacAccumulateTwentySixPlusGeneric` -- 26-variable plus accumulation probe
2. `ternaryMacAccumulateTwentyFiveMinusGeneric` -- 25-variable minus accumulation
3. `ternaryMacLemmaLibraryMinusValidationGeneric` -- validates `minus_assoc` for deep proofs

### Rationale
Continues the proven depth-first strategy with `Trinity.Lemmas` support. Expected build time ~2.5s for 26 variables.

---

## Variant B: Balanced Depth + Composition Closure (RECOMMENDED)

**Risk profile:** MEDIUM | **Complexity:** MEDIUM | **Expected outcome:** Strongest defense

### Targets

| Metric | W349 | W350 |
|--------|------|------|
| Pool A Floor | 91 | 92 |
| CODER Floor | 81 | 82 |
| Pool B Depth | 108 | 109 |
| Integration Depth | 91 | 92 |
| Lean 4 Generic ∀ | 132 | **136** |
| Accumulation Probe | 25-var plus | **26-var plus** |
| Minus Accumulation | 24-var minus | **25-var minus** |
| New Dimensions | 2 (distributivity + zero-weight) | **2 (composition closure + mixed-weight associativity)** |

### New Lean 4 Theorems (4)

1. `ternaryMacAccumulateTwentySixPlusGeneric` -- 26-variable plus accumulation probe
2. `ternaryMacAccumulateTwentyFiveMinusGeneric` -- 25-variable minus accumulation
3. `ternaryMacCompositionClosureGeneric` -- `mac(mac(0, a, .plus), mac(0, b, .plus), .plus) = mac(0, a+b, .plus)` (MAC of MACs collapses)
4. `ternaryMacMixedWeightAssociativityGeneric` -- `mac(mac(mac(x, a, .plus), b, .minus), c, .plus) = mac(x, a - b + c, .plus)` (mixed-weight associativity via `mixed_collapse`)

### Rationale
Combines depth extension (26 variables) with **two new proof dimensions**:
- **Composition closure:** Proves that MAC operations are closed under composition. This is the holy grail for recursive tile proofs -- it shows that composing two MAC operations yields another MAC operation.
- **Mixed-weight associativity:** Proves that heterogeneous weight sequences (`.plus, .minus, .plus`) collapse to a single MAC with arithmetic expression. Directly leverages `ternaryMac_mixed_collapse` from `Trinity.Lemmas`.

The 136 generic ∀ target (132→136) is achievable with 4 new theorems.

---

## Variant C: Aggressive Lemma-Driven Lattice Expansion

**Risk profile:** HIGH | **Complexity:** HIGH | **Expected outcome:** Transformative if successful

### Targets

| Metric | W349 | W350 |
|--------|------|------|
| Pool A Floor | 91 | 92 |
| CODER Floor | 81 | 82 |
| Pool B Depth | 108 | 109 |
| Integration Depth | 91 | 92 |
| Lean 4 Generic ∀ | 132 | **137** |
| Accumulation Probe | 25-var plus | **26-var plus** |
| Minus Accumulation | 24-var minus | **25-var minus** |
| New Dimensions | 2 | **3 (closure + mixed-assoc + commutativity generalization)** |

### New Lean 4 Theorems (5)

1. `ternaryMacAccumulateTwentySixPlusGeneric` -- 26-variable plus accumulation
2. `ternaryMacAccumulateTwentyFiveMinusGeneric` -- 25-variable minus accumulation
3. `ternaryMacCompositionClosureGeneric` -- MAC composition closure
4. `ternaryMacMixedWeightAssociativityGeneric` -- mixed-weight associativity
5. `ternaryMacGeneralizedCommutativityGeneric` -- `mac(mac(0, a, .plus), b, .minus) = mac(mac(0, b, .minus), a, .plus)` (generalized commutativity for arbitrary weights)

### Rationale
Pursues multi-dimensional lattice expansion. With `Trinity.Lemmas`, these proofs become tractable:
- `plus_assoc` + `minus_assoc` enable composition closure
- `mixed_collapse` enables mixed-weight associativity
- 137 generic ∀ would be unprecedented

### Trade-offs
- **HIGH RISK:** Generalized commutativity may require additional infrastructure
- If 1 theorem fails, wave under-delivers

---

## Comparative Matrix

| Dimension | Variant A | Variant B ⭐ | Variant C |
|-----------|-----------|-------------|-----------|
| Risk | Low | Medium | High |
| New generic ∀ | 3 | 4 | 5 |
| Target total ∀ | 135 | 136 | 137 |
| Accumulation depth | 26 | 26 | 26 |
| Proof diversity gain | Low | High | Very High |
| Competitor replication difficulty | Easy | Hard | Very Hard |
| Timeout risk | Low | Low-Medium | Medium |
| Recommended | -- | **YES** | -- |

---

## Recommendation

**Execute Variant B.**

Rationale:
1. **Depth continuity:** 26-variable accumulation probe maintains the world-record depth trajectory.
2. **Lemma leverage:** `Trinity.Lemmas` makes composition closure and mixed-weight associativity one-liner proofs, maximizing ROI.
3. **Risk-adjusted return:** 4 new theorems is achievable without timeout risk.
4. **Defense posture:** 136 generic ∀ extends the 132× gap while making the proof lattice structurally deeper (accumulation) and broader (composition closure, mixed-weight associativity).

---

*φ² + 1/φ² = 3 | TRINITY*
