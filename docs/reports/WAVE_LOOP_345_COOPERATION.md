# Wave Loop 345 -- Cooperation Variants for W346

**Date:** 2026-06-23
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #345`
**Basis:** W345 Report (`docs/reports/WAVE_LOOP_345_REPORT.md`)

---

## Strategic Goal for W346

Extend the **118 generic ∀ milestone** to **121 generic ∀** (3 new theorems).

Target depths:
- Pool A: 88→89
- CODER: 78→79
- Pool B (systolic_ternary): 105→106
- Integration (ternary_inference): 88→89
- Lean 4: 118→121 generic ∀

---

## Variant A -- Conservative (22-variable probe + 21-variable minus + grind-only)

### Lean 4 Theorems (3 new → 121 total generic ∀)

1. **`ternaryMacAccumulateTwentyTwoPlusGeneric`**
   `mac^22(0, [a..v], .plus) = a+b+...+v`
   **22-variable omega boundary probe.** Extends deepest accumulation depth to 22. Expected build time 2.0s. If timeout, document and fall back to 21-var minus.

2. **`ternaryMacAccumulateTwentyOneMinusGeneric`**
   `mac^21(0, [a..u], .minus) = -(a+b+...+u)`
   Completes 21-variable accumulation lattice in minus weight. Symmetric to AccumulateTwentyOnePlusGeneric (W345). Foundation for symmetric 21-operand systolic tiles with minus weight.

3. **`ternaryMacGrindOnlyBaseGeneric`**
   `mac(mac(0, a, .plus), b, .plus) = a + b`
   **Grind-only benchmark** -- attempts to prove a simple identity using `grind [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]` without prior `simp`. Tests whether grind can be the sole automation tactic for base-case theorems.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥89, CODER ≥79, Pool B ≥106, Integration ≥89

### Risk
- LOW: 22-variable may be near omega timeout threshold.

---

## Variant B -- Recommended (22-variable probe + 21-variable minus + mixed-weight commutativity)

*(Recommended by Trinity Agent based on W345 learnings: mixed-weight psum scaling succeeded, next frontier is mixed-weight commutativity and depth 22.)*

### Lean 4 Theorems (3 new → 121 total generic ∀)

1. **`ternaryMacAccumulateTwentyTwoPlusGeneric`**
   `mac^22(0, [a..v], .plus) = a+b+...+v`
   **22-variable omega boundary probe.** Expected 2.0s build. If timeout, replace with `AccumulateTwentyOneMinusGeneric`.

2. **`ternaryMacAccumulateTwentyOneMinusGeneric`**
   `mac^21(0, [a..u], .minus) = -(a+b+...+u)`
   **21-variable minus accumulation** -- completes the 21-variable accumulation lattice. Symmetric to AccumulateTwentyOnePlusGeneric (W345). Together, they establish dual-polarity parity at depth 21.

3. **`ternaryMacMixedWeightCommutativityGeneric`**
   `mac(mac(0, a, .plus), b, .minus) = mac(mac(0, b, .minus), a, .plus)`
   **Mixed-weight commutativity** -- proves that the order of activations with opposite weights can be swapped with sign adjustment. This is the first commutativity theorem across different ternary weights, opening a new algebraic structure beyond same-weight properties.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥89, CODER ≥79, Pool B ≥106, Integration ≥89

### Advantage
- 22-variable probe is the natural continuation; if omega saturates, fallback preserves 21-var minus.
- 21-variable minus lattice completion gives symmetric coverage at deepest accumulation depth.
- Mixed-weight commutativity is the highest-value next theorem -- it establishes that ternary MAC algebra forms a near-commutative structure across weight polarities, enabling reordering optimizations in systolic arrays.

### Risk
- MEDIUM: `simp+omega` may timeout at 22 variables. If so, replace AccumulateTwentyTwoPlusGeneric with a smaller accumulation theorem.
- MEDIUM: Mixed-weight commutativity may require `simp+grind+omega` combination. May need manual proof steps.

---

## Variant C -- Ambitious (22-variable + 22-variable minus + grind deep accumulation)

### Lean 4 Theorems (3 new → 121 total generic ∀)

1. **`ternaryMacAccumulateTwentyTwoPlusGeneric`**
   **22-variable omega boundary probe.**

2. **`ternaryMacAccumulateTwentyTwoMinusGeneric`**
   **22-variable minus accumulation** -- `-(a+b+...+v)` -- doubles the omega boundary probe across both weights. High risk if 22-variable plus already times out.

3. **`ternaryMacGrindAccumulateTenGeneric`**
   `mac^10(0, [a..j], .plus) = a+b+...+j` -- proven with `simp+grind` instead of `simp+omega`. Benchmarks whether `grind` outperforms `omega` on medium-depth accumulation goals. If grind is faster or more robust, recommends full grind migration for accumulation theorems.

### Additional Work
- **Grind deep accumulation benchmark:** Convert `ternaryMacAccumulateTenPlusGeneric` from `simp+omega` to `simp+grind`. Benchmark build time.
- **22-variable timeout contingency:** If both 22-var theorems fail, fallback to 21-var minus + MixedWeightCommutativityGeneric + one smaller theorem (equivalent to Variant B).

### Risk
- HIGH: Two 22-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: Grind may not scale to 10-variable accumulation as well as omega.
- HIGH: Mixed-weight commutativity is skipped in favor of grind benchmarking, leaving an algebraic gap.

---

## Comparison Matrix

| Dimension | Variant A (Conservative) | Variant B (Recommended) | Variant C (Ambitious) |
|-----------|-------------------------|------------------------|----------------------|
| Accumulation depth | 22 (+1 from W345) | 22 (+1 from W345) | 22 (+1 from W345) |
| Minus lattice | 21-var completion | 21-var completion | 22-var (risky) |
| Mixed-weight algebra | No | Yes (commutativity) | No |
| Grind migration | Yes (simple only) | No | Yes (deep accumulation) |
| Timeout risk | LOW | MEDIUM | HIGH |
| Theorem novelty | Medium | Very High | High |
| Build time est. | 2.0s | 2.0s | 2.1s+ |
| Competitive impact | 121 generic ∀ | 121 generic ∀ | 121 generic ∀ |

---

## Recommended: Variant B

**Variant B** is recommended because:
1. **22-variable probe** is the natural next step in the accumulation depth progression. Build time has been linear (1.5s→1.6s→1.7s→1.8s→1.9s), so 2.0s is a reasonable estimate.
2. **21-variable minus lattice completion** gives symmetric coverage at the deepest accumulation depth. This is more valuable than pushing minus to 22 variables (which risks timeout).
3. **Mixed-weight commutativity** is the highest-value next theorem. It proves that ternary MAC operations are near-commutative across weight polarities, enabling activation reordering in systolic arrays -- a critical optimization for hardware scheduling.
4. **Risk is manageable:** If 22-variable fails, fallback to 21-variable minus preserves lattice completion. If mixed-weight commutativity fails, replace with grind benchmark or smaller theorem.

---

## Rollout Plan

1. **W346 issue creation:** File `#346` with `Closes #346` referencing this doc.
2. **Variant B selection** (or fallback to A if timeout detected).
3. **Batch append** +2 tests, +1 invariant per spec.
4. **Lean 4 theorems** (3 new).
5. **Build** (`lake build Trinity.TernaryInference`).
6. **Seal regenerate** (27 IGLA seals).
7. **Suite run** (`t27c suite --repo-root .`).
8. **Commit** with `Closes #346`.
9. **Report + cooperation variants** for W347.
10. **Memory + skill update**.

---

**φ² + 1/φ² = 3 | TRINITY**
