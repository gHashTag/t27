# Wave Loop 322 (W322) — Three Cooperation Variants for W323

**Date:** 2026-06-23
**For:** Wave Loop 323 (W323) planning
**φ² + 1/φ² = 3 | TRINITY**

---

## Variant A: Conservative — "Identity Trinity Completion"

**Focus:** Complete the identity-element lattice for all three ternary weights before accelerating.

### W323 Targets
| Metric | W322 | W323 Target | Delta |
|--------|------|-------------|-------|
| Pool A Uniform Floor | 64 | **65** | +1 |
| CODER Uniform Floor | 55 | **56** | +1 |
| Pool B Depth | 80 | **81** | +1 |
| Integration Depth | 65 | **66** | +1 |
| Lean 4 Generic ∀ | 50 | **52** | +2 |

### Lean 4 Theorems (+2)
1. `ternaryMacZeroPsumMinusWeightIdentityGeneric (a : Int)` — `mac(0, a, -) = -a`
2. `ternaryMacZeroPsumZeroWeightIdentityGeneric (a : Int)` — `mac(0, a, 0) = 0`

### Rationale
- W322 proved identity for `plus` weight only. A complete **identity trinity** (plus/minus/zero) is required for any generic proof that abstracts over `TernaryWeight`.
- These two theorems extend the monoid fragment from W322 into a **unital magma** for each weight code individually.
- Low risk: identical proof pattern (`simp + omega`) as W322 theorems.

### Risk: LOW
### Effort: Standard (+54 tests, +27 invariants, +2 theorems)

---

## Variant B: Balanced — "Associativity Generalization + 52 Milestone"

**Focus:** Generalize associativity from `plus`-only to generic `TernaryWeight`, and add inverse completion.

### W323 Targets
| Metric | W322 | W323 Target | Delta |
|--------|------|-------------|-------|
| Pool A Uniform Floor | 64 | **65** | +1 |
| CODER Uniform Floor | 55 | **56** | +1 |
| Pool B Depth | 80 | **82** | +2 |
| Integration Depth | 65 | **66** | +1 |
| Lean 4 Generic ∀ | 50 | **53** | +3 |

### Lean 4 Theorems (+3)
1. `ternaryMacPsumAssociativityMinusGeneric (psum a b : Int)` — `mac(mac(psum, a, -), b, -) = mac(psum, a+b, -)`
2. `ternaryMacPsumAssociativityMixedGeneric (psum a b : Int)` — `mac(mac(psum, a, +), b, -) = mac(psum, a-b, -)` (or equivalent mixed form)
3. `ternaryMacPlusMinusInverseMinusGeneric (a : Int)` — `mac(mac(0, a, -), a, +) = 0` — inverse commutes

### Rationale
- W322's associativity is limited to `plus` weights. Real systolic arrays process mixed weights; generalizing associativity to minus and mixed cases covers **all 9 weight-code pairs**.
- Mixed associativity is the last algebraic property needed for **tiling proofs** — decomposing a large GEMM into 2×2 or 4×4 systolic tiles.
- 53 generic ∀ extends the half-century milestone with a clear next target: **55 by W324**.

### Risk: MEDIUM (mixed-weight associativity may require `cases` or `ring_nf`)
### Effort: Elevated (+54 tests, +27 invariants, +3 theorems)

---

## Variant C: Aggressive — "Ring Closure + Inverse Lattice"

**Focus:** Prove that ternary MAC operations form a closed algebraic system under composition, and reach toward a full inverse lattice.

### W323 Targets
| Metric | W322 | W323 Target | Delta |
|--------|------|-------------|-------|
| Pool A Uniform Floor | 64 | **66** | +2 |
| CODER Uniform Floor | 55 | **57** | +2 |
| Pool B Depth | 80 | **83** | +3 |
| Integration Depth | 65 | **67** | +2 |
| Lean 4 Generic ∀ | 50 | **53** | +3 |

### Lean 4 Theorems (+3)
1. `ternaryMacNestedWeightCommutativityGeneric (a : Int) (w1 w2 : TernaryWeight)` — `mac(mac(0, a, w1), a, w2) = mac(mac(0, a, w2), a, w1)` — weight-order independence for same activation
2. `ternaryMacPsumAssociativityZeroGeneric (psum a b : Int)` — `mac(mac(psum, a, 0), b, w) = mac(psum, a, w)` — zero-weight is universal absorber in nested context
3. `ternaryMacTriplePlusAssociativityGeneric (psum a b c : Int)` — 3-level nested MAC associativity — direct hardware relevance for 3-stage systolic pipelines

### Rationale
- **Weight commutativity** proves that the order of weight application does not matter when activation is constant — a property exploited by weight-stationary systolic arrays.
- **Zero-weight absorber** in nested context formalizes the "skip" behavior of sparse systolic arrays when weight is zero.
- **Triple associativity** bridges to real 3-stage FPGA pipelines (common in Artix-7 implementations).
- Pool A +2 and Pool B +3 provide visible momentum alongside the algebraic depth.

### Risk: HIGH (weight commutativity requires 9-case analysis; triple associativity needs careful variable naming)
### Effort: Very High (+108 tests, +54 invariants, +3 complex theorems)

---

## Recommendation

**Recommended: Variant B (Balanced)**
- Extends the W322 monoid fragment into a **complete associativity lattice** (plus, minus, mixed)
- Mixed-weight associativity is the **last missing piece** for tiled-GEMM decomposition proofs
- 53 generic ∀ — clear progression from half-century milestone toward **55 by W324**
- Moderate risk, high structural value

**Fallback:** Variant A, if mixed-weight proofs reveal unexpected complexity with `omega` (may need `ring_nf` preprocessing).

**Stretch:** Variant C, if Variant B completes ahead of schedule and additional pool depth is desired.

---

*Cooperation variants generated by Trinity Agent (Queen) — AEL v2.0 Phase 5: SYNTHESIZE*
