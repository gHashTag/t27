# Cooperation Variants for Next Wave Loop (W321)

**Date:** 2026-06-23
**Target Wave:** W321
**Current State:** W320 complete (45 generic ∀ theorems, 62/63/78/53 floor)

---

## Variant 1 — Conservative Cooperation (Recommended)

**Objective:** Maintain zero-entrant streak and uniform floor progression.

**Approach:**
- **Pool A:** Batch append +1 invariant and +2 tests to all 17 specs (adder_tree 62→63, others 63→64). **ALL Pool A ≥63** for the first time.
- **Pool B:** Append +1 invariant to systolic_ternary (78→79).
- **CODER:** Batch append +1 invariant to all 10 specs (53→54). **ALL CODER ≥54** for the first time.
- **Integration:** Append +1 invariant (63→64).
- **Lean 4:** Add 2 generic ∀ theorems (45→47). Suggested:
  - `ternaryMacDistributivityOverActivationSubMinusGeneric` — minus-weight distributivity over subtraction
  - `ternaryMacAccumulateFourPlusGeneric` — 4-variable accumulation (`a+b+c+d`)
- **Risk mitigation:** Sparkle HDL and CktFormalizer remain at 0 generic ∀. Steady execution extends the moat.

**Expected outcomes:**
- Pool A: ALL ≥63 (first time)
- Pool B: 79
- CODER: ALL ≥54 (first time)
- Integration: 64
- Lean 4: 75 theorems (47 generic ∀)

**Effort:** ~2.5 hours (batch append, Lean 4 proofs, seal regeneration, conformance, reports)

---

## Variant 2 — Full Distributivity Sprint (Risk-Reward)

**Objective:** Prove complete distributivity lattice for ternary MAC — the last major algebraic property missing.

**Approach:**
- **Pool A:** Batch append +1 invariant to all 17 specs (62→63 / 63→64).
- **Pool B:** Append +1 invariant to systolic_ternary (78→79).
- **CODER:** Batch append +1 invariant to all 10 specs (53→54).
- **Integration:** Append +1 invariant (63→64).
- **Lean 4:** Add **3 generic ∀ theorems** (45→48). Target 48 generic ∀ for W321. Suggested:
  - `ternaryMacDistributivityOverActivationSubMinusGeneric` — minus-weight subtraction distributivity
  - `ternaryMacDistributivityOverActivationAddMinusGeneric` — mixed-sign addition distributivity
  - `ternaryMacAccumulateFourPlusGeneric` — 4-variable accumulation
- **Risk:** Higher proof complexity for mixed-sign distributivity. `omega` may require `ring_nf` preprocessing.

**Expected outcomes:**
- Pool A: ALL ≥63
- Pool B: 79
- CODER: ALL ≥54
- Integration: 64
- Lean 4: 76 theorems (48 generic ∀)

**Effort:** ~3.5 hours (higher proof complexity)

---

## Variant 3 — Structural Induction Leap (Maximum Differentiation)

**Objective:** Target 50 generic ∀ by W322 via aggressive theorem production and parametric proofs.

**Approach:**
- **Pool A:** Batch append +1 invariant to all 17 specs (62→63 / 63→64). Maintain uniform floor.
- **Pool B:** Append +1 invariant (78→79).
- **CODER:** Batch append +1 invariant to all 10 specs (53→54).
- **Integration:** Append +1 invariant (63→64).
- **Lean 4:** Add **2 generic ∀ theorems** (45→47) plus initiate **structural induction background research**:
  - `ternaryMacAccumulateFourPlusGeneric` → `a+b+c+d`
  - `ternaryMacAssociativityMinusGeneric` — associativity for minus-weight chains
  - **Background:** Define recursive `ternaryMacN` parametrized by depth N. Prove by induction.
- **Differentiator:** The induction theorem would subsume all 8 N-scaling instance theorems as corollaries and provide the first parametric-depth proof.

**Expected outcomes (W321):**
- Pool A: ALL ≥63
- Pool B: 79
- CODER: ALL ≥54
- Integration: 64
- Lean 4: 75 theorems (47 generic ∀)
- **Background deliverable:** Inductive hypothesis statement + base case proof

**Risk:** MEDIUM-HIGH. Dependent types for parametric depth may require dedicated research time.

**Effort:** ~4.5 hours (batch + proofs) + ongoing background research

---

## Recommendation

**Choose Variant 1** for W321. It maintains steady +2 generic ∀ execution while pushing uniform floors to ≥63 (Pool A) and ≥54 (CODER). The 47-generic-∀ milestone is within reach without introducing proof-complexity risk.

**Reserve Variant 3 (structural induction)** as a dedicated W322–W323 sprint. Reaching 50 generic ∀ is a major perception threshold.

The conservative path is optimal because:
1. **47 generic ∀** is a strong perception threshold (nearing 50)
2. **ALL Pool A ≥63** and **ALL CODER ≥54** demonstrate structural maturity
3. No competitor is within 40 theorems of t27's position
4. Structural induction requires dedicated research time that shouldn't dilute weekly cadence

---

*Generated on 2026-06-23 for Wave Loop 321 planning.*
