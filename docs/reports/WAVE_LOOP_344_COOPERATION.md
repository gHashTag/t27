# Wave Loop 344 -- Cooperation Variants for W345

**Date:** 2026-06-23
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #344`
**Basis:** W344 Report (`docs/reports/WAVE_LOOP_344_REPORT.md`)

---

## Strategic Goal for W345

Extend the **115 generic ∀ milestone** to **118 generic ∀** (3 new theorems).

Target depths:
- Pool A: 86→87
- CODER: 76→77
- Pool B (systolic_ternary): 103→104
- Integration (ternary_inference): 86→87
- Lean 4: 115→118 generic ∀

---

## Variant A -- Conservative (21-variable probe + 20-variable minus + grind-only)

### Lean 4 Theorems (3 new → 118 total generic ∀)

1. **`ternaryMacAccumulateTwentyOnePlusGeneric`**
   `mac^21(0, [a..u], .plus) = a+b+...+u`
   **21-variable omega boundary probe.** Extends deepest accumulation depth to 21. Expected build time 1.9s. If timeout, document and fall back to 20-var minus.

2. **`ternaryMacAccumulateTwentyMinusGeneric`**
   `mac^20(0, [a..t], .minus) = -(a+b+...+t)`
   Completes 20-variable accumulation lattice in minus weight. Symmetric to AccumulateTwentyPlusGeneric (W344). Foundation for symmetric 20-operand systolic tiles with minus weight.

3. **`ternaryMacGrindOnlyGeneric`**
   `mac(mac(0, a, .plus), b, .plus) = a + b`
   **Grind-only benchmark** -- attempts to prove a simple identity using `grind` with definition hints (`grind [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]`) instead of `simp+grind+omega`. Tests whether grind can be the sole automation tactic.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥87, CODER ≥77, Pool B ≥104, Integration ≥87

### Risk
- LOW: 21-variable may be near omega timeout threshold.

---

## Variant B -- Recommended (21-variable probe + 20-variable minus + mixed psum scaling)

*(Recommended by Trinity Agent based on W344 learnings: accumulation lattice complete at 19, next frontiers are depth 20 symmetry and mixed-weight algebra.)*

### Lean 4 Theorems (3 new → 118 total generic ∀)

1. **`ternaryMacAccumulateTwentyOnePlusGeneric`**
   `mac^21(0, [a..u], .plus) = a+b+...+u`
   **21-variable omega boundary probe.** Expected 1.9s build. If timeout, replace with `AccumulateTwentyMinusGeneric`.

2. **`ternaryMacAccumulateTwentyMinusGeneric`**
   `mac^20(0, [a..t], .minus) = -(a+b+...+t)`
   **20-variable minus accumulation** -- completes the 20-variable accumulation lattice. Symmetric to AccumulateTwentyPlusGeneric (W344). Together, they establish dual-polarity parity at depth 20.

3. **`ternaryMacPsumMixedScalingGeneric`**
   `mac(mac(0, a, .plus), k*b, .minus) = mac(0, a - k*b, .plus)`
   **Mixed-weight psum scaling** -- extends scalar scaling to mixed weights with arbitrary accumulator. Most algebraically complex theorem in the psum scaling family. Proves that systolic tile quantization is invariant under mixed weight transitions. This opens a new algebraic dimension beyond same-weight psum scaling.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥87, CODER ≥77, Pool B ≥104, Integration ≥87

### Advantage
- 21-variable probe is the natural continuation; if omega saturates, fallback preserves 20-var minus.
- 20-variable minus lattice completion gives symmetric coverage at deepest accumulation depth.
- Mixed-weight psum scaling is the highest-value next theorem -- it generalizes the entire psum scaling family to cross-weight transitions, enabling proofs for systolic arrays with alternating weight polarities.

### Risk
- MEDIUM: `simp+omega` may timeout at 21 variables. If so, replace AccumulateTwentyOnePlusGeneric with a smaller accumulation theorem.
- MEDIUM: Mixed-weight psum scaling may fail to simplify with `simp+omega`. May need `simp+grind`.

---

## Variant C -- Ambitious (21-variable + 21-variable minus + grind deep accumulation)

### Lean 4 Theorems (3 new → 118 total generic ∀)

1. **`ternaryMacAccumulateTwentyOnePlusGeneric`**
   **21-variable omega boundary probe.**

2. **`ternaryMacAccumulateTwentyOneMinusGeneric`**
   **21-variable minus accumulation** -- `-(a+b+...+u)` -- doubles the omega boundary probe across both weights. High risk if 21-variable plus already times out.

3. **`ternaryMacGrindAccumulateBenchmarkGeneric`**
   `mac^10(0, [a..j], .plus) = a+b+...+j` -- proven with `simp+grind` instead of `simp+omega`. Benchmarks whether `grind` outperforms `omega` on medium-depth accumulation goals. If grind is faster or more robust, recommends full grind migration for accumulation theorems.

### Additional Work
- **Grind deep accumulation benchmark:** Convert `ternaryMacAccumulateTenPlusGeneric` from `simp+omega` to `simp+grind`. Benchmark build time.
- **21-variable timeout contingency:** If both 21-var theorems fail, fallback to 20-var minus + PsumMixedScalingGeneric + one smaller theorem (equivalent to Variant B).

### Risk
- HIGH: Two 21-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: Grind may not scale to 10-variable accumulation as well as omega.
- HIGH: Mixed-weight psum scaling is skipped in favor of grind benchmarking, leaving an algebraic gap.

---

## Comparison Matrix

| Dimension | Variant A (Conservative) | Variant B (Recommended) | Variant C (Ambitious) |
|-----------|-------------------------|------------------------|----------------------|
| Accumulation depth | 21 (+1 from W344) | 21 (+1 from W344) | 21 (+1 from W344) |
| Minus lattice | 20-var completion | 20-var completion | 21-var (risky) |
| Mixed psum | No | Yes | No |
| Grind migration | Yes (simple only) | No | Yes (deep accumulation) |
| Timeout risk | LOW | MEDIUM | HIGH |
| Theorem novelty | Medium | High | High |
| Build time est. | 1.9s | 1.9s | 2.0s+ |
| Competitive impact | 118 generic ∀ | 118 generic ∀ | 118 generic ∀ |

---

## Recommended: Variant B

**Variant B** is recommended because:
1. **21-variable probe** is the natural next step in the accumulation depth progression. Build time has been linear (1.5s→1.6s→1.7s→1.8s), so 1.9s is a reasonable estimate.
2. **20-variable minus lattice completion** gives symmetric coverage at the deepest accumulation depth. This is more valuable than pushing minus to 21 variables (which risks timeout).
3. **Mixed-weight psum scaling** is the highest-value next theorem. It generalizes the psum scaling family to cross-weight transitions, unlocking proofs for systolic arrays with alternating weight polarities -- a realistic hardware scenario.
4. **Risk is manageable:** If 21-variable fails, fallback to 20-variable minus preserves lattice completion. If mixed-weight psum scaling fails, replace with grind benchmark or smaller theorem.

---

## Rollout Plan

1. **W345 issue creation:** File `#345` with `Closes #345` referencing this doc.
2. **Variant B selection** (or fallback to A if timeout detected).
3. **Batch append** +2 tests, +1 invariant per spec.
4. **Lean 4 theorems** (3 new).
5. **Build** (`lake build Trinity.TernaryInference`).
6. **Seal regenerate** (27 IGLA seals).
7. **Suite run** (`t27c suite --repo-root .`).
8. **Commit** with `Closes #345`.
9. **Report + cooperation variants** for W346.
10. **Memory + skill update**.

---

**φ² + 1/φ² = 3 | TRINITY**
