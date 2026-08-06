# Wave Loop 343 -- Cooperation Variants for W344

**Date:** 2026-06-23
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #343`
**Basis:** W343 Report (`docs/reports/WAVE_LOOP_343_REPORT.md`)

---

## Strategic Goal for W344

Extend the **112 generic ∀ milestone** to **115 generic ∀** (3 new theorems).

Target depths:
- Pool A: 86→87
- CODER: 76→77
- Pool B (systolic_ternary): 103→104
- Integration (ternary_inference): 86→87
- Lean 4: 112→115 generic ∀

---

## Variant A -- Conservative (20-variable probe + 19-variable minus + mixed psum)

### Lean 4 Theorems (3 new → 115 total generic ∀)

1. **`ternaryMacAccumulateTwentyPlusGeneric`**  
   `mac^20(0, [a..t], .plus) = a+b+...+t`  
   **20-variable omega boundary probe.** Extends deepest accumulation depth to 20. Expected build time 1.8s. If timeout, document and fall back to 19-var minus.

2. **`ternaryMacAccumulateNineteenMinusGeneric`**  
   `mac^19(0, [a..s], .minus) = -(a+b+...+s)`  
   Completes 19-variable accumulation lattice in minus weight. Symmetric to AccumulateNineteenPlusGeneric (W343). Foundation for symmetric 19-operand systolic tiles with dual-polarity accumulation.

3. **`ternaryMacPsumMixedScalingGeneric`**  
   `mac(mac(0, a, .plus), k*b, .minus) = mac(0, a - k*b, .plus)`  
   **Mixed-weight psum scaling** -- extends scalar scaling to mixed weights with arbitrary accumulator. Most algebraically complex theorem in the psum scaling family. Proves that systolic tile quantization is invariant under mixed weight transitions.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥87, CODER ≥77, Pool B ≥104, Integration ≥87

### Risk
- LOW: 20-variable may be near omega timeout threshold. If `simp+omega` fails, replace with smaller theorem.

---

## Variant B -- Recommended (20-variable probe + 19-variable minus + grind migration spike)

*(Recommended by Trinity Agent based on W343 learnings: psum scaling lattice complete, next frontiers are accumulation depth extension and tactic modernization.)*

### Lean 4 Theorems (3 new → 115 total generic ∀)

1. **`ternaryMacAccumulateTwentyPlusGeneric`**  
   `mac^20(0, [a..t], .plus) = a+b+...+t`  
   **20-variable omega boundary probe.** Expected 1.8s build. If timeout, replace with `AccumulateNineteenMinusGeneric`.

2. **`ternaryMacAccumulateNineteenMinusGeneric`**  
   `mac^19(0, [a..s], .minus) = -(a+b+...+s)`  
   **19-variable minus accumulation** -- completes the 19-variable accumulation lattice. Symmetric to AccumulateNineteenPlusGeneric (W343). Together, they establish dual-polarity parity at depth 19.

3. **`ternaryMacGrindBenchmarkGeneric`**  
   `mac(mac(0, a, .plus), b, .plus) = mac(0, a+b, .plus)` (identity+associativity base case) proven with `grind` instead of `simp+omega`.  
   **Grind tactic migration spike** -- converts one existing simple theorem to use Lean 4 v4.31+ built-in commutative ring solver. Benchmarks build time vs `simp+omega`. If `grind` is faster or produces smaller proof terms, document and recommend for W345+ full migration.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥87, CODER ≥77, Pool B ≥104, Integration ≥87

### Advantage
- 20-variable probe is the natural continuation; if omega saturates, fallback preserves 19-var minus.
- 19-variable minus lattice completion gives symmetric coverage at deepest accumulation depth.
- Grind migration is a strategic investment -- if it outperforms `simp+omega`, future accumulation theorems could scale beyond 20 variables.

### Risk
- MEDIUM: `simp+omega` may timeout at 20 variables. If so, replace AccumulateTwentyPlusGeneric with a smaller accumulation theorem.
- MEDIUM: `grind` may not prove the base case as cleanly as `simp+omega`. Fallback: keep existing `simp+omega` proof and try `grind` on a different theorem.

---

## Variant C -- Ambitious (20-variable + 20-variable minus + mixed psum scaling)

### Lean 4 Theorems (3 new → 115 total generic ∀)

1. **`ternaryMacAccumulateTwentyPlusGeneric`**  
   **20-variable omega boundary probe.**

2. **`ternaryMacAccumulateTwentyMinusGeneric`**  
   **20-variable minus accumulation** -- `-(a+b+...+t)` -- doubles the omega boundary probe across both weights. High risk if 20-variable plus already times out.

3. **`ternaryMacPsumMixedScalingGeneric`**  
   `mac(mac(0, a, .plus), k*b, .minus) = mac(0, a - k*b, .plus)`  
   **Mixed-weight psum scaling** -- extends scalar scaling to mixed weights with arbitrary psum. Most algebraically complex theorem in the psum scaling family.

### Additional Work
- **Grind tactic migration spike:** Convert `ternaryMacPsumScalingPlusGeneric` (W343) from `simp+omega` to `grind`. Benchmark build time.
- **20-variable timeout contingency:** If both 20-var theorems fail, fallback to 19-var minus + PsumMixedScalingGeneric + grind spike (equivalent to Variant B with mixed psum instead of grind benchmark).

### Risk
- HIGH: Two 20-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: Mixed-weight psum scaling may fail to simplify with `simp+omega`.
- HIGH: Grind migration is exploratory; may not complete within wave scope.

---

## Comparison Matrix

| Dimension | Variant A (Conservative) | Variant B (Recommended) | Variant C (Ambitious) |
|-----------|-------------------------|------------------------|----------------------|
| Accumulation depth | 20 (+1 from W343) | 20 (+1 from W343) | 20 (+1 from W343) |
| Minus lattice | 19-var completion | 19-var completion | 20-var (risky) |
| Mixed psum | Yes | No | Yes |
| Grind migration | No | Yes (spike) | Yes (on existing thm) |
| Timeout risk | LOW | MEDIUM | HIGH |
| Theorem novelty | High | Medium-High | Very High |
| Build time est. | 1.8s | 1.8s | 2.0s+ |
| Competitive impact | 115 generic ∀ | 115 generic ∀ | 115 generic ∀ |

---

## Recommended: Variant B

**Variant B** is recommended because:
1. **20-variable probe** is the natural next step in the accumulation depth progression. Build time has been linear (1.5s→1.6s→1.7s), so 1.8s is a reasonable estimate.
2. **19-variable minus lattice completion** gives symmetric coverage at the deepest accumulation depth. This is more valuable than pushing minus to 20 variables (which risks timeout).
3. **Grind migration spike** is a strategic investment. If Lean 4's built-in `grind` tactic outperforms `simp+omega`, future waves could scale beyond 20 variables with cleaner proofs. Even if `grind` doesn't win, the benchmark data is valuable for long-term tactic selection.
4. **Risk is manageable:** If 20-variable fails, fallback to 19-variable minus preserves lattice completion. If `grind` fails, fallback to `simp+omega` preserves the theorem.

---

## Rollout Plan

1. **W344 issue creation:** File `#344` with `Closes #344` referencing this doc.
2. **Variant B selection** (or fallback to A if timeout detected).
3. **Batch append** +2 tests, +1 invariant per spec.
4. **Lean 4 theorems** (3 new).
5. **Build** (`lake build Trinity.TernaryInference`).
6. **Seal regenerate** (27 IGLA seals).
7. **Suite run** (`t27c suite --repo-root .`).
8. **Commit** with `Closes #344`.
9. **Report + cooperation variants** for W345.
10. **Memory + skill update**.

---

**φ² + 1/φ² = 3 | TRINITY**
