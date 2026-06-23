# Cooperation Variants for Next Wave Loop (W318)

**Date:** 2026-06-23
**Target Wave:** W318
**Current State:** W317 complete (37 generic ∀ theorems, 59/60/75/50 floor)

---

## Variant 1 — Conservative Cooperation (Recommended)

**Objective:** Maintain zero-entrant streak and uniform floor progression.

**Approach:**
- **Pool A:** Batch append +1 invariant and +2 tests to all 17 specs (adder_tree 59→60, others 60→61). **ALL Pool A ≥60** for the first time.
- **Pool B:** Append +1 invariant to systolic_ternary (75→76).
- **CODER:** Batch append +1 invariant to all 10 specs (50→51). **ALL CODER ≥51** for the first time.
- **Integration:** Append +1 invariant (60→61).
- **Lean 4:** Add 2 generic ∀ theorems (37→39). Suggested:
  - `ternaryMacAccumulateTwoMinusGeneric` → `-(a+b)` (2-variable, minus weights)
  - `ternaryMacDistributivityOverActivationSubGeneric` → `mac(psum, a-b, w) = mac(psum, a, w) - mac(0, b, w)`
- **Risk mitigation:** Sparkle HDL and CktFormalizer remain at 0 generic ∀. Steady execution extends the moat.

**Expected outcomes:**
- Pool A: ALL ≥60 (first time)
- Pool B: 76
- CODER: ALL ≥51 (first time)
- Integration: 61
- Lean 4: 71 theorems (39 generic ∀)

**Effort:** ~2.5 hours (batch append, Lean 4 proofs, seal regeneration, conformance, reports)

---

## Variant 2 — Associativity Sprint (Risk-Reward)

**Objective:** Prove MAC associativity for zero-psum cases — the last major algebraic property missing from the ternary MAC theory.

**Approach:**
- **Pool A:** Batch append +1 invariant to all 17 specs (59→60 / 60→61).
- **Pool B:** Append +1 invariant to systolic_ternary (75→76).
- **CODER:** Batch append +1 invariant to all 10 specs (50→51).
- **Integration:** Append +1 invariant (60→61).
- **Lean 4:** Add **3 generic ∀ theorems** (37→40). Target 40 generic ∀ for W318 — crossing into the 40s. Suggested:
  - `ternaryMacAccumulateTwoMinusGeneric` → `-(a+b)`
  - `ternaryMacAssociativityZeroPsumGeneric` → `mac(mac(0,a,w1),b,w2) = mac(0,a,w1) + ternaryMul(b,w2)`
  - `ternaryMacCommutativityPlusWeightGeneric` → `mac(psum, a, .plus) = mac(a, psum, .plus)` (if valid)
- **Risk:** Associativity may require case analysis on weight combinations. Medium proof complexity.

**Expected outcomes:**
- Pool A: ALL ≥60
- Pool B: 76
- CODER: ALL ≥51
- Integration: 61
- Lean 4: 72 theorems (40 generic ∀)

**Effort:** ~3.5 hours (higher proof complexity)

---

## Variant 3 — Structural Induction Leap (Maximum Differentiation)

**Objective:** Generalize the N-scaling pattern to a parametric induction theorem, elevating t27 from "instance proofs" to "general theory."

**Approach:**
- **Pool A:** Batch append +1 invariant to all 17 specs (59→60 / 60→61). Maintain uniform floor.
- **Pool B:** Append +1 invariant (75→76).
- **CODER:** Batch append +1 invariant to all 10 specs (50→51).
- **Integration:** Append +1 invariant (60→61).
- **Lean 4:** Add **2 generic ∀ theorems** (37→39) plus initiate **structural induction background research**:
  - `ternaryMacAccumulateTwoMinusGeneric` → `-(a+b)`
  - `ternaryMacDistributivityOverActivationSubGeneric` → subtraction distributivity
  - **Background:** Define recursive `ternaryMacN` parametrized by depth N. Prove by induction that `∀ N, mac^N(0, a, .plus) = N*a`.
- **Differentiator:** The induction theorem would be the **first formally verified parametric-depth proof** for ternary systolic arrays. It would subsume all 8 N-scaling instance theorems as corollaries.

**Expected outcomes (W318):**
- Pool A: ALL ≥60
- Pool B: 76
- CODER: ALL ≥51
- Integration: 61
- Lean 4: 71 theorems (39 generic ∀)
- **Background deliverable:** Inductive hypothesis statement + base case proof (not counted in 39)

**Risk:** MEDIUM-HIGH. Dependent types for parametric depth may require `Nat` recursion and `simp`/`induction` tactics that don't follow the established `omega` pattern. Estimated 1 proof engineer × 2 weeks background research.

**Effort:** ~4.5 hours (batch + proofs) + ongoing background research

---

## Recommendation

**Choose Variant 1** for W318. It maintains steady +2 generic ∀ execution while pushing uniform floors to ≥60 (Pool A) and ≥51 (CODER). The 39-generic-∀ milestone is within reach without introducing proof-complexity risk.

**Reserve Variant 3 (structural induction)** as a dedicated W319–W320 sprint. Completing the depth-5 family and establishing 2-variable theorems provides sufficient semantic depth before attempting parametric induction.

The conservative path is optimal because:
1. **39 generic ∀** is a strong perception threshold (nearing 40)
2. **ALL Pool A ≥60** and **ALL CODER ≥51** demonstrate structural maturity
3. No competitor is within 30 theorems of t27's position
4. Structural induction requires dedicated research time that shouldn't dilute the weekly wave loop cadence

---

*Generated on 2026-06-23 for Wave Loop 318 planning.*
