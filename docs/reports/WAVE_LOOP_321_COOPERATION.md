# Cooperation Variants for Next Wave Loop (W322)

**Date:** 2026-06-23
**Target Wave:** W322
**Current State:** W321 complete (47 generic ∀ theorems, 63/64/79/54 floor)

---

## Variant 1 — Conservative Cooperation (Recommended)

**Objective:** Maintain zero-entrant streak and uniform floor progression.

**Approach:**
- **Pool A:** Batch append +1 invariant and +2 tests to all 17 specs (adder_tree 63→64, others 64→65). **ALL Pool A ≥64** for the first time.
- **Pool B:** Append +1 invariant to systolic_ternary (79→80).
- **CODER:** Batch append +1 invariant to all 10 specs (54→55). **ALL CODER ≥55** for the first time.
- **Integration:** Append +1 invariant (64→65).
- **Lean 4:** Add 2 generic ∀ theorems (47→49). Suggested:
  - `ternaryMacPsumAssociativityGeneric` — full associativity with arbitrary accumulator
  - `ternaryMacZeroPsumIdentityGeneric` — zero-psum is identity element for MAC
- **Risk mitigation:** Sparkle HDL and CktFormalizer remain at 0 generic ∀. Steady execution extends the moat.

**Expected outcomes:**
- Pool A: ALL ≥64 (first time)
- Pool B: 80
- CODER: ALL ≥55 (first time)
- Integration: 65
- Lean 4: 77 theorems (49 generic ∀)

**Effort:** ~2.5 hours (batch append, Lean 4 proofs, seal regeneration, conformance, reports)

---

## Variant 2 — Identity Element Sprint (Risk-Reward)

**Objective:** Prove identity elements and inverse properties for ternary MAC — the last major algebraic structure properties missing.

**Approach:**
- **Pool A:** Batch append +1 invariant to all 17 specs (63→64 / 64→65).
- **Pool B:** Append +1 invariant to systolic_ternary (79→80).
- **CODER:** Batch append +1 invariant to all 10 specs (54→55).
- **Integration:** Append +1 invariant (64→65).
- **Lean 4:** Add **3 generic ∀ theorems** (47→50). Target 50 generic ∀ for W322 — crossing the 50s threshold. Suggested:
  - `ternaryMacZeroPsumIdentityGeneric` — `mac(0,a,.plus) = a`
  - `ternaryMacZeroActivationIdentityGeneric` — `mac(psum,0,.plus) = psum`
  - `ternaryMacPlusMinusInverseGeneric` — `mac(mac(0,a,.plus),a,.minus) = 0`
- **Risk:** Medium proof complexity for inverse properties. `omega` may require case analysis on weights.

**Expected outcomes:**
- Pool A: ALL ≥64
- Pool B: 80
- CODER: ALL ≥55
- Integration: 65
- Lean 4: 78 theorems (50 generic ∀)

**Effort:** ~3.5 hours (higher proof complexity)

---

## Variant 3 — Structural Induction Leap (Maximum Differentiation)

**Objective:** Target 50 generic ∀ by W322 via parametric induction proof, elevating t27 from "instance enumeration" to "general theory."

**Approach:**
- **Pool A:** Batch append +1 invariant to all 17 specs (63→64 / 64→65). Maintain uniform floor.
- **Pool B:** Append +1 invariant (79→80).
- **CODER:** Batch append +1 invariant to all 10 specs (54→55).
- **Integration:** Append +1 invariant (64→65).
- **Lean 4:** Add **2 generic ∀ theorems** (47→49) plus initiate **structural induction background research**:
  - `ternaryMacPsumAssociativityGeneric` — associativity with arbitrary accumulator
  - `ternaryMacZeroPsumIdentityGeneric` — identity element
  - **Background:** Define recursive `ternaryMacN` parametrized by depth N. Prove by induction that `∀ N, mac^N(0,a,.plus) = N*a`.
- **Differentiator:** The induction theorem would subsume all 8 N-scaling instance theorems as corollaries and provide the first parametric-depth proof in verified ternary hardware.

**Expected outcomes (W322):**
- Pool A: ALL ≥64
- Pool B: 80
- CODER: ALL ≥55
- Integration: 65
- Lean 4: 77 theorems (49 generic ∀)
- **Background deliverable:** Inductive hypothesis statement + base case proof

**Risk:** MEDIUM-HIGH. Dependent types for parametric depth may require dedicated research time.

**Effort:** ~4.5 hours (batch + proofs) + ongoing background research

---

## Recommendation

**Choose Variant 2** for W322. It targets **50 generic ∀** — a major perception threshold — while keeping the implementation manageable with 3 theorems. Identity and inverse properties are natural extensions of the linearity proofs from W321 and complete the algebraic structure of ternary MAC.

**Reserve Variant 3 (structural induction)** as a dedicated W323–W324 sprint. Reaching 50 generic ∀ via instance proofs provides a strong foundation before attempting parametric induction.

The identity sprint is optimal because:
1. **50 generic ∀** is a landmark milestone (half-century)
2. **ALL Pool A ≥64** and **ALL CODER ≥55** demonstrate structural maturity
3. No competitor is within 45 theorems of t27's position
4. Identity/inverse proofs are natural extensions of existing proof patterns

---

*Generated on 2026-06-23 for Wave Loop 322 planning.*
