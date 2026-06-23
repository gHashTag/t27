# Wave Loop 345 -- Cooperation Variants for W346

**Date:** 2026-06-16
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #345`
**Basis:** W345 Report (`docs/reports/WAVE_LOOP_345_REPORT.md`)

---

## Strategic Goal for W346

Extend the **118 generic ∀ milestone** to **121 generic ∀** (3 new theorems).

Target depths:
- Pool A: 87→88
- CODER: 77→78
- Pool B (systolic_ternary): 104→105
- Integration (ternary_inference): 87→88
- Lean 4: 118→121 generic ∀

---

## Variant A -- Conservative (22-variable probe + 21-variable minus + psum mixed grind)

### Lean 4 Theorems (3 new → 121 total generic ∀)

1. **`ternaryMacAccumulateTwentyTwoPlusGeneric`**
   `mac^22(0, [a..v], .plus) = a+b+...+v`
   **22-variable omega boundary probe.** Extends deepest accumulation depth to 22. Expected build time 1.8-2.0s (sub-linear scaling observed). If timeout, document and fall back to 21-var minus.

2. **`ternaryMacAccumulateTwentyOneMinusGeneric`**
   `mac^21(0, [a..u], .minus) = -(a+b+...+u)`
   Completes 21-variable accumulation lattice in minus weight. Symmetric to AccumulateTwentyOnePlusGeneric (W345). Foundation for symmetric 21-operand systolic tiles with minus weight.

3. **`ternaryMacPsumMixedScalingGrindGeneric`**
   `mac(mac(0, a, .plus), k*b, .minus) = mac(0, a - k*b, .plus)`
   **Grind migration spike** -- attempts the mixed psum scaling theorem using `simp [defs] <;> try grind <;> try omega`. Tests whether `grind` outperforms `omega` on mixed-weight algebraic goals.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥88, CODER ≥78, Pool B ≥105, Integration ≥88

### Risk
- LOW: 22-variable may be near omega timeout threshold, but sub-linear scaling suggests room.

---

## Variant B -- Recommended (22-variable probe + 21-variable minus + dual psum activation)

*(Recommended by Trinity Agent based on W345 learnings: accumulation depth scales sub-linearly to 21, next frontiers are depth 22 symmetry and dual-weight psum activation.)*

### Lean 4 Theorems (3 new → 121 total generic ∀)

1. **`ternaryMacAccumulateTwentyTwoPlusGeneric`**
   `mac^22(0, [a..v], .plus) = a+b+...+v`
   **22-variable omega boundary probe.** Expected 1.8-2.0s build. If timeout, replace with `AccumulateTwentyOneMinusGeneric`.

2. **`ternaryMacAccumulateTwentyOneMinusGeneric`**
   `mac^21(0, [a..u], .minus) = -(a+b+...+u)`
   **21-variable minus accumulation** -- completes the 21-variable accumulation lattice. Symmetric to AccumulateTwentyOnePlusGeneric (W345). Together, they establish dual-polarity parity at depth 21.

3. **`ternaryMacPsumDualActivationGeneric`**
   `mac(mac(psum, a, .plus), a, .minus) = psum`
   **Dual-weight psum activation cancellation.** Proves that a plus then minus activation with the same operand cancels out, returning the original psum. This is the fundamental cancellation law for systolic arrays with alternating weight polarities. Opens the door to tile-level equivalence proofs for mixed-weight PE arrays.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥88, CODER ≥78, Pool B ≥105, Integration ≥88

### Advantage
- 22-variable probe is the natural continuation; if omega saturates, fallback preserves 21-var minus.
- 21-variable minus lattice completion gives symmetric coverage at deepest accumulation depth.
- Dual-weight psum activation cancellation is the highest-value next theorem -- it proves a fundamental algebraic cancellation law that enables tile-level equivalence for mixed-weight PE arrays. This is the next step after mixed-weight scaling.

### Risk
- MEDIUM: `simp+omega` may timeout at 22 variables. If so, replace AccumulateTwentyTwoPlusGeneric with a smaller accumulation theorem.
- MEDIUM: Dual-weight psum activation may need additional lemmas (e.g., `Int.add_neg_self`) for `simp+omega` to resolve.

---

## Variant C -- Ambitious (22-variable + 22-variable minus + psum associativity quadruple)

### Lean 4 Theorems (3 new → 121 total generic ∀)

1. **`ternaryMacAccumulateTwentyTwoPlusGeneric`**
   **22-variable omega boundary probe.**

2. **`ternaryMacAccumulateTwentyTwoMinusGeneric`**
   **22-variable minus accumulation** -- `-(a+b+...+v)` -- doubles the omega boundary probe across both weights. High risk if 22-variable plus already times out.

3. **`ternaryMacPsumQuadrupleAssociativityMixedGeneric`**
   `mac(mac(mac(mac(0,a,.plus),b,.minus),c,.plus),d,.minus) = mac(0, a - b + c - d, .plus)`
   **Quadruple mixed-weight psum associativity.** The most algebraically complex theorem attempted -- proves that four mixed-weight MAC operations collapse to a single MAC with combined operands. If this passes, it validates the entire mixed-weight algebraic framework for systolic tiles.

### Additional Work
- **Quadruple mixed-weight associativity contingency:** If this fails, fallback to `ternaryMacPsumDualActivationGeneric` (Variant B).
- **22-variable timeout contingency:** If both 22-var theorems fail, fallback to 21-var minus + PsumDualActivationGeneric + one smaller theorem (equivalent to Variant B).

### Risk
- HIGH: Two 22-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: Quadruple mixed-weight associativity may fail to simplify with `simp+omega`. May need custom lemmas or `grind`.

---

**φ² + 1/φ² = 3 | TRINITY**
