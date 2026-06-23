# Wave Loop 346 -- Cooperation Variants for W347

**Date:** 2026-06-16
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #346`
**Basis:** W346 Report (`docs/reports/WAVE_LOOP_346_REPORT.md`)

---

## Strategic Goal for W347

Extend the **122 generic ∀ milestone** to **125 generic ∀** (3 new theorems).

Target depths:
- Pool A: 88→89
- CODER: 78→79
- Pool B (systolic_ternary): 105→106
- Integration (ternary_inference): 88→89
- Lean 4: 122→125 generic ∀

---

## Variant A -- Conservative (23-variable probe + 22-variable minus + psum dual commutativity)

### Lean 4 Theorems (3 new → 125 total generic ∀)

1. **`ternaryMacAccumulateTwentyThreePlusGeneric`**
   `mac^23(0, [a..w], .plus) = a+b+...+w`
   **23-variable omega boundary probe.** Extends deepest accumulation depth to 23. Expected build time 1.0-1.2s (solver below baseline). If timeout, document and fall back to 22-var minus.

2. **`ternaryMacAccumulateTwentyTwoMinusGeneric`**
   `mac^22(0, [a..v], .minus) = -(a+b+...+v)`
   Completes 22-variable accumulation lattice in minus weight. Symmetric to AccumulateTwentyTwoPlusGeneric (W346). Foundation for symmetric 22-operand systolic tiles with minus weight.

3. **`ternaryMacPsumDualCommutativityGeneric`**
   `mac(mac(psum, a, .plus), b, .minus) = mac(mac(psum, b, .minus), a, .plus)`
   **Dual-weight psum commutativity.** Generalizes mixed-weight commutativity to arbitrary accumulators. Proves that cross-weight activation order can be swapped with sign adjustment for any live accumulator.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥89, CODER ≥79, Pool B ≥106, Integration ≥89

### Risk
- LOW: 23-variable may be near omega timeout threshold, but 1.0s at 22 variables suggests room.

---

## Variant B -- Recommended (23-variable probe + 22-variable minus + psum triple mixed associativity)

*(Recommended by Trinity Agent based on W346 learnings: accumulation depth solver is below compilation baseline at 22 variables, next frontiers are depth 23 symmetry and triple mixed-weight associativity.)*

### Lean 4 Theorems (3 new → 125 total generic ∀)

1. **`ternaryMacAccumulateTwentyThreePlusGeneric`**
   `mac^23(0, [a..w], .plus) = a+b+...+w`
   **23-variable omega boundary probe.** Expected 1.0-1.2s build. If timeout, replace with `AccumulateTwentyTwoMinusGeneric`.

2. **`ternaryMacAccumulateTwentyTwoMinusGeneric`**
   `mac^22(0, [a..v], .minus) = -(a+b+...+v)`
   **22-variable minus accumulation** -- completes the 22-variable accumulation lattice. Symmetric to AccumulateTwentyTwoPlusGeneric (W346). Together, they establish dual-polarity parity at depth 22.

3. **`ternaryMacPsumTripleMixedAssociativityGeneric`**
   `mac(mac(mac(psum, a, .plus), b, .minus), c, .plus) = mac(psum, a - b + c, .plus)`
   **Triple mixed-weight psum associativity.** Proves that three mixed-weight MAC operations collapse to a single MAC with combined operands. This is the next step after dual-weight cancellation -- it validates that arbitrary-length mixed-weight chains can be algebraically collapsed. Enables proofs for deep systolic arrays with alternating polarities.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥89, CODER ≥79, Pool B ≥106, Integration ≥89

### Advantage
- 23-variable probe is the natural continuation; if omega saturates, fallback preserves 22-var minus.
- 22-variable minus lattice completion gives symmetric coverage at deepest accumulation depth.
- Triple mixed-weight psum associativity is the highest-value next theorem -- it proves that arbitrary-length mixed-weight chains collapse algebraically, enabling proofs for deep systolic arrays with alternating polarities. This is the next step after dual-weight cancellation.

### Risk
- MEDIUM: `simp+omega` may timeout at 23 variables. If so, replace AccumulateTwentyThreePlusGeneric with a smaller accumulation theorem.
- MEDIUM: Triple mixed-weight associativity may need additional lemmas for `simp+omega` to resolve.

---

## Variant C -- Ambitious (23-variable + 23-variable minus + psum quadruple mixed associativity)

### Lean 4 Theorems (3 new → 125 total generic ∀)

1. **`ternaryMacAccumulateTwentyThreePlusGeneric`**
   **23-variable omega boundary probe.**

2. **`ternaryMacAccumulateTwentyThreeMinusGeneric`**
   **23-variable minus accumulation** -- `-(a+b+...+w)` -- doubles the omega boundary probe across both weights. High risk if 23-variable plus already times out.

3. **`ternaryMacPsumQuadrupleMixedAssociativityGeneric`**
   `mac(mac(mac(mac(psum, a, .plus), b, .minus), c, .plus), d, .minus) = mac(psum, a - b + c - d, .plus)`
   **Quadruple mixed-weight psum associativity.** The most algebraically complex theorem attempted -- proves that four mixed-weight MAC operations collapse to a single MAC with combined operands. If this passes, it validates the entire mixed-weight algebraic framework for deep systolic tiles.

### Additional Work
- **Quadruple mixed-weight associativity contingency:** If this fails, fallback to `ternaryMacPsumTripleMixedAssociativityGeneric` (Variant B).
- **23-variable timeout contingency:** If both 23-var theorems fail, fallback to 22-var minus + TripleMixedAssociativity + one smaller theorem (equivalent to Variant B).

### Risk
- HIGH: Two 23-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: Quadruple mixed-weight associativity may fail to simplify with `simp+omega`. May need custom lemmas or `grind`.

---

**φ² + 1/φ² = 3 | TRINITY**
