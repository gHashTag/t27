# Wave Loop 342 -- Cooperation Variants for W343

**Date:** 2026-06-23
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #342`
**Basis:** W342 Report (`docs/reports/WAVE_LOOP_342_REPORT.md`)

---

## Strategic Goal for W343

Extend the **109 generic ∀ milestone** to **112 generic ∀** (3 new theorems).

Target depths:
- Pool A: 84→85
- CODER: 74→75
- Pool B (systolic_ternary): 101→102
- Integration (ternary_inference): 84→85
- Lean 4: 109→112 generic ∀

---

## Variant A -- Conservative (19-variable probe + minus completion)

### Lean 4 Theorems (3 new → 112 total generic ∀)

1. **`ternaryMacAccumulateNineteenPlusGeneric`**  
   `mac^19(0, [a..s], .plus) = a+b+...+s`  
   **19-variable omega boundary probe.** Extends deepest accumulation depth by 1. Expected build time 1.7s. If timeout, document and fall back to 18-var minus.

2. **`ternaryMacAccumulateEighteenMinusGeneric`**  
   `mac^18(0, [a..r], .minus) = -(a+b+...+r)`  
   Completes 18-variable accumulation lattice in minus weight. Symmetric to AccumulateEighteenPlusGeneric (W342). Foundation for 18-operand systolic tiles with minus weight.

3. **`ternaryMacPsumScalingPlusGeneric`** (REPLACES Variant C)  
   `mac(mac(0, a, .plus), k*b, .plus) = mac(0, a + k*b, .plus)`  
   **Psum scalar scaling for plus weights** -- extends scalar scaling from `psum=0` base to arbitrary accumulator. This generalizes the W340-W342 scalar scaling family to psum contexts, unlocking arbitrary-depth systolic tile quantization.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥85, CODER ≥75, Pool B ≥102, Integration ≥85

### Risk
- LOW: 19-variable may be near omega timeout threshold. If `simp+omega` fails, replace with `AccumulateEighteenMinusGeneric` + 2 smaller theorems.

---

## Variant B -- Recommended (19-variable + psum scaling + minus completion)

*(Recommended by Trinity Agent based on W342 learnings: 3-weight scaling lattice complete, next frontier is psum generalization.)*

### Lean 4 Theorems (3 new → 112 total generic ∀)

1. **`ternaryMacAccumulateNineteenPlusGeneric`**  
   `mac^19(0, [a..s], .plus) = a+b+...+s`  
   **19-variable omega boundary probe.** Expected 1.7s build. If timeout, replace with `AccumulateEighteenMinusGeneric`.

2. **`ternaryMacPsumScalingPlusGeneric`**  
   `mac(mac(0, a, .plus), k*b, .plus) = mac(0, a + k*b, .plus)`  
   **Psum scalar scaling generalization** -- arbitrary accumulator plus-weight scaling. This is the highest-value theorem because it generalizes the entire scalar scaling lattice (W340-W342) to real systolic psum contexts where `psum ≠ 0`. Enables quantization-aware tiling proofs.

3. **`ternaryMacPsumScalingMinusGeneric`**  
   `mac(mac(0, a, .minus), k*b, .minus) = mac(0, a + k*b, .minus)`  
   **Psum scalar scaling for minus weights** -- complete the psum scaling lattice for both plus and minus weights. Zero-weight psum scaling is trivial (zero multiplication), so this makes the psum scaling family essentially complete.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥85, CODER ≥75, Pool B ≥102, Integration ≥85

### Advantage
- Psum scaling opens a new algebraic dimension: scaling invariance with arbitrary accumulator.
- Completes minus-weight psum scaling, giving near-complete coverage across plus/minus/zero weights.
- 19-variable is the natural next accumulation step; if omega saturates, fallback documented.

### Risk
- MEDIUM: `simp+omega` may timeout at 19 variables. If so, replace AccumulateNineteenPlusGeneric with AccumulateEighteenMinusGeneric and keep psum scaling pair.

---

## Variant C -- Ambitious (19-variable + grind migration + mixed psum scaling)

### Lean 4 Theorems (3 new → 112 total generic ∀)

1. **`ternaryMacAccumulateNineteenPlusGeneric`**  
   **19-variable omega boundary probe.**

2. **`ternaryMacAccumulateNineteenMinusGeneric`**  
   **19-variable minus accumulation** -- `-(a+b+...+s)` -- doubles the omega boundary probe across both weights. High risk if 19-variable plus already times out.

3. **`ternaryMacPsumMixedScalingGeneric`**  
   `mac(mac(0, a, .plus), k*b, .minus) = mac(0, a - k*b, .plus)`  
   **Mixed-weight psum scaling** -- extends scalar scaling to mixed weights with arbitrary psum. Most algebraically complex theorem in the psum scaling family. Proves that systolic tile quantization is invariant under mixed weight transitions.

### Additional Work
- **Grind tactic migration spike:** Convert one existing accumulation theorem from `simp+omega` to `grind` (Lean 4 v4.31+). Benchmark build time. If `grind` is faster or scales further, document and recommend for W344.
- **19-variable timeout contingency:** If both 19-var theorems fail, fallback to 18-var minus + PsumScalingPlusGeneric + PsumScalingMinusGeneric (equivalent to Variant B).

### Risk
- HIGH: Two 19-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: Mixed-weight psum scaling may fail to simplify with `simp+omega`. May need `grind`.
- HIGH: Grind migration is exploratory; may not complete within wave scope.

---

## Comparison Matrix

| Dimension | Variant A (Conservative) | Variant B (Recommended) | Variant C (Ambitious) |
|-----------|-------------------------|------------------------|----------------------|
| Accumulation depth | 19 (+1 from W342) | 19 (+1 from W342) | 19 (+1 from W342) |
| Psum scaling | Partial (plus only) | Complete (plus+minus) | Mixed weight |
| Grind migration | No | No | Yes (spike) |
| Timeout risk | LOW | MEDIUM | HIGH |
| Theorem novelty | Medium | High | Very High |
| Build time est. | 1.7s | 1.8s | 2.0s+ |
| Competitive impact | 112 generic ∀ | 112 generic ∀ | 112 generic ∀ |

---

## Recommended: Variant B

**Variant B** is recommended because:
1. **Psum scaling generalization** is the highest-value next step. Scalar scaling has been fully proven for `psum=0` (W340-W342). Real hardware uses `psum ≠ 0`, so psum scaling unlocks quantization proofs for actual systolic tiles.
2. **Plus+minus pair** completes the psum scaling lattice in a single wave, giving maximum coverage.
3. **19-variable probe** is the natural continuation; if omega times out, the fallback preserves the psum scaling gains.
4. **Risk is manageable:** If 19-variable fails, fallback to 18-variable minus (still valuable) + psum scaling pair.

---

## Rollout Plan

1. **W343 issue creation:** File `#343` with `Closes #343` referencing this doc.
2. **Variant B selection** (or fallback to A if timeout detected).
3. **Batch append** +2 tests, +1 invariant per spec.
4. **Lean 4 theorems** (3 new).
5. **Build** (`lake build Trinity.TernaryInference`).
6. **Seal regenerate** (27 IGLA seals).
7. **Suite run** (`t27c suite --repo-root .`).
8. **Commit** with `Closes #343`.
9. **Report + cooperation variants** for W344.
10. **Memory + skill update**.

---

**φ² + 1/φ² = 3 | TRINITY**
