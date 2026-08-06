# Wave Loop 347 -- Cooperation Variants for W348

**Date:** 2026-06-23
**Branch:** `trinity-rust-rings`
**Issue Gate:** `Closes #347`
**Basis:** W347 Report (`docs/reports/WAVE_LOOP_347_REPORT.md`)

---

## Strategic Goal for W348

Extend the **125 generic ∀ milestone** to **128 generic ∀** (3 new theorems).

Target depths:
- Pool A: 89→90
- CODER: 79→80
- Pool B (systolic_ternary): 106→107
- Integration (ternary_inference): 89→90
- Lean 4: 125→128 generic ∀

---

## Variant A -- Conservative (24-variable probe + 23-variable minus + psum quadruple mixed associativity)

### Lean 4 Theorems (3 new → 128 total generic ∀)

1. **`ternaryMacAccumulateTwentyFourPlusGeneric`**
   `mac^24(0, [a..x], .plus) = a+b+...+x`
   **24-variable omega boundary probe.** Extends deepest accumulation depth to 24. Expected build time 1.5-2.0s. If timeout, document and fall back to 23-var minus.

2. **`ternaryMacAccumulateTwentyThreeMinusGeneric`**
   `mac^23(0, [a..w], .minus) = -(a+b+...+w)`
   Completes 23-variable accumulation lattice in minus weight. Symmetric to AccumulateTwentyThreePlusGeneric (W347). Foundation for symmetric 23-operand systolic tiles with minus weight.

3. **`ternaryMacPsumQuadrupleMixedAssociativityGeneric`**
   `mac(mac(mac(mac(psum, a, .plus), b, .minus), c, .plus), d, .minus) = mac(psum, a - b + c - d, .plus)`
   **Quadruple mixed-weight psum associativity.** Proves that four mixed-weight MAC operations collapse to a single MAC with combined operands. Validates that arbitrary-length mixed-weight chains can be algebraically collapsed. If this fails, fallback to `ternaryMacPsumTripleMixedAssociativityGeneric` (already proven in W347).

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥90, CODER ≥80, Pool B ≥107, Integration ≥90

### Risk
- LOW: 24-variable may be near omega timeout threshold, but 23-variable passed without timeout suggesting room.
- MEDIUM: Quadruple mixed-weight associativity may need additional lemmas for `simp+omega`.

---

## Variant B -- Recommended (24-variable probe + 23-variable minus + custom lemma library spike)

*(Recommended by Trinity Agent based on W347 learnings: accumulation depth solver is below compilation baseline at 23 variables, but the next frontier requires reducing `simp` expansion overhead through a custom lemma library.)*

### Lean 4 Theorems (3 new → 128 total generic ∀)

1. **`ternaryMacAccumulateTwentyFourPlusGeneric`**
   `mac^24(0, [a..x], .plus) = a+b+...+x`
   **24-variable omega boundary probe.** Expected 1.5-2.0s build. If timeout, replace with `AccumulateTwentyThreeMinusGeneric`.

2. **`ternaryMacAccumulateTwentyThreeMinusGeneric`**
   `mac^23(0, [a..w], .minus) = -(a+b+...+w)`
   **23-variable minus accumulation** -- completes the 23-variable accumulation lattice. Symmetric to AccumulateTwentyThreePlusGeneric (W347). Together, they establish dual-polarity parity at depth 23.

3. **`ternaryMacLemmaLibrarySpike`** *(meta-theorem)*
   Create a `TernaryMac.Lemmas` module with pre-proven helper lemmas:
   - `ternaryMac_plus_assoc` -- `mac(mac(acc, a, .plus), b, .plus) = mac(acc, a+b, .plus)`
   - `ternaryMac_minus_assoc` -- `mac(mac(acc, a, .minus), b, .minus) = mac(acc, a+b, .minus)`
   - `ternaryMac_mixed_collapse` -- `mac(mac(acc, a, .plus), b, .minus) = mac(acc, a-b, .plus)`
   **Purpose:** Reduce `simp` expansion overhead by providing pre-proven compositional lemmas. This is the structural investment needed to scale beyond 25 variables.

### Spec Batch Append
- +2 tests, +1 invariant per spec (27 specs)
- Depth: Pool A ≥90, CODER ≥80, Pool B ≥107, Integration ≥90

### Advantage
- 24-variable probe is the natural continuation; if omega saturates, fallback preserves 23-var minus.
- 23-variable minus lattice completion gives symmetric coverage at deepest accumulation depth.
- Custom lemma library is the **highest-value structural investment** -- it reduces proof automation time and enables scaling to 30+ variables. Without it, every theorem re-expands `ternaryMac_eq_acc_plus_mul` from scratch.

### Risk
- MEDIUM: `simp+omega` may timeout at 24 variables. Fallback to 23-var minus preserves progress.
- MEDIUM: Lemma library creation requires architectural decision (new file `TernaryMac/Lemmas.lean` or inline module).

---

## Variant C -- Ambitious (24-variable + 24-variable minus + grind tactic migration)

### Lean 4 Theorems (3 new → 128 total generic ∀)

1. **`ternaryMacAccumulateTwentyFourPlusGeneric`**
   **24-variable omega boundary probe.**

2. **`ternaryMacAccumulateTwentyFourMinusGeneric`**
   `mac^24(0, [a..x], .minus) = -(a+b+...+x)`
   **24-variable minus accumulation** -- doubles the omega boundary probe across both weights. High risk if 24-variable plus already times out.

3. **`ternaryMacGrindBenchmarkGeneric`**
   `mac(mac(0, a, .plus), b, .plus) = a+b`
   **Grind tactic migration spike.** Proves the base case accumulation identity using Lean 4's `grind` tactic (commutative ring solver) instead of `simp+omega`. If successful, replaces the entire `simp+omega` pipeline for accumulation theorems with `grind`, which scales to much larger expressions.

### Additional Work
- **Grind tactic validation:** If `grind` solves the base case, migrate all accumulation theorems to `grind` for W349+.
- **24-variable timeout contingency:** If both 24-var theorems fail, fallback to 23-var minus + LemmaLibrarySpike + one smaller theorem (equivalent to Variant B).

### Risk
- HIGH: Two 24-variable theorems may exceed omega timeout. Requires fallback plan.
- HIGH: `grind` tactic may not solve ternary MAC identities. Requires experimental validation.

---

## Recommended Path: Variant B

**Justification:** The 23-variable accumulation boundary is confirmed. The next bottleneck is not omega solver capacity, but `simp` expansion overhead. A custom lemma library (`TernaryMac.Lemmas`) is the structural investment that enables scaling to 30+ variables and reduces build times across all future waves. This is the engineering priority that compounds over time.

---

**φ² + 1/φ² = 3 | TRINITY**
