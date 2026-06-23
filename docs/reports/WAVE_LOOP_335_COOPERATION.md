# Wave Loop 335 — Cooperation Variants for W336

**Date:** 2026-06-23
**Prepared for:** IGLA CODER + IGLA RACE coordination
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 335 achieved **92 generic ∀ theorems** — the **92× competitor milestone**.
The 11-variable accumulation boundary test succeeded: `simp+omega` scales to 11
variables without timeout, confirming unprecedented automation depth.

Four new ternary hardware/software projects emerged in April–June 2026
(TWLA, TernaryCore, Litespark, Balanced_Ternary), but **none include formal
verification**. The competitive moat widens.

W336 targets:
- Pool A floor ≥78, CODER ≥68, Pool B ≥95, Integration ≥78
- Lean 4 generic ∀ ≥95 (3 new theorems)
- 12-variable accumulation boundary test, mixed-weight scalar associativity
- Ecosystem integration research (TorchLean, ATLAAS)

---

## Variant A — Depth Sprint (Accumulation Boundary Test)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacAccumulateTwelvePlusGeneric` (12-variable plus accumulation — **omega saturation test**)
  - `ternaryMacAccumulateTwelveMinusGeneric` (12-variable minus accumulation)
  - `ternaryMacMixedWeightAssociativityScalarGeneric` (mixed-weight scalar associativity)

**Rationale:**
The 12-variable theorem is the ultimate stress test. If `simp+omega` succeeds,
t27 establishes automation scalability to 12 variables — a result worth publishing.
If it fails, we document the omega boundary at 11. Mixed-weight scalar
associativity closes a gap in the algebraic lattice.

**Risk:**
12-variable expressions may exceed Lean 4's `omega` solver capacity, causing
timeouts (>5s) or stack overflow. If it fails, replace with `AccumulateElevenMinus`
(already proven in W335) plus a safer theorem like `PsumScalingGeneric`.

**Commit message pattern:** `feat(w336): W336 IGLA CODER+RACE — 12-variable accumulation boundary test, 95 generic ∀`

---

## Variant B — Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacScalarAssociativityMixedGeneric` (`mac(mac(0,a,.plus),b,.minus) = mac(0,a-b,.plus)`)
  - `ternaryMacPsumScalingGeneric` (`mac(psum,k*a,.plus) = k*mac(psum,a,.plus)`)
  - `ternaryMacAccumulateTwelvePlusGeneric` (12-variable plus accumulation — controlled experiment)

**Rationale:**
Maintains the proven cadence while closing the mixed-weight scalar associativity
gap and testing the 12-variable boundary as a controlled experiment. If the
12-variable theorem fails, we remove it and still deliver 2 solid theorems.
`PsumScalingGeneric` generalizes scalar linearity (W321) to arbitrary accumulators.

**Commit message pattern:** `feat(w336): W336 IGLA CODER+RACE — Pool A 77→78, CODER 67→68, Pool B 94→95, Lean 4 92→95 generic ∀`

---

## Variant C — Ecosystem Integration (TorchLean + TernaryCore Collaboration)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **2 generic ∀ theorems** (conservative)
- **NEW:** Research task — evaluate TorchLean and TernaryCore for collaboration

**Rationale:**
With four new ternary projects (TWLA, TernaryCore, Litespark, Balanced_Ternary)
and TorchLean as a formalization framework, the strategic priority shifts to
**ecosystem positioning**:

1. **TorchLean** — t27's ternary MAC theorems could be expressed as TorchLean
   network invariants. Propose: "Add ternary weight operator and MAC accumulation
   theorems to TorchLean's operator library."

2. **TernaryCore** — t27's generic ∀ proofs could replace TernaryCore's
   simulation-based verification with machine-checked proofs. Propose:
   "Formalize TernaryCore's ternary MAC unit in Lean 4 using t27's algebra."

**Task breakdown:**
1. Read TorchLean paper (arXiv:2602.22631) and identify operator extension points.
2. Draft a note showing how `ternaryMacAccumulateElevenPlusGeneric` maps to
   TorchLean's IBP/CROWN certificate structure.
3. Read TernaryCore RTL and identify formalization targets.
4. Propose a joint formalization roadmap.

**Risk:**
External collaboration timelines uncertain. Recommend starting with a 1-page
proposal and GitHub issue.

**Commit message pattern:** `feat(w336): W336 IGLA CODER+RACE + research(w336): TorchLean/TernaryCore ecosystem integration`

---

## Recommendation

**Execute Variant B (Balanced Expansion)** as the primary W336 plan. The
proven 3-theorem cadence has delivered 92 theorems over 13 consecutive waves
without a single build failure. Variant B closes the mixed-weight associativity
gap while testing the 12-variable boundary.

**Parallel track:** Initiate Variant C ecosystem integration as a **lightweight
research spike** (~10% effort). Draft a 1-page note mapping t27's theorems to
TorchLean operators and post it as a GitHub issue or blog post. TernaryCore
outreach can follow if TorchLean responds positively.

**Avoid Variant A** unless the 12-variable boundary test is explicitly prioritized.
The risk of omega timeout is moderate; Variant B handles this gracefully by
including it as a controlled experiment with a fallback theorem.

---

## GitHub Issue Review

**Open issues relevant to W336:**
- No new ternary-specific formal verification issues found in GitHub search.
- TorchLean repository (lean-dojo/TorchLean) has no ternary operator issues.
- TernaryCore repository (shepherdscientific/ternarycore) has no formal verification issues.

**Recommended actions:**
1. Open issue on lean-dojo/TorchLean: "Feature request: ternary weight operator
   and MAC accumulation theorems" — link t27's 92 generic ∀ theorems as motivation.
2. Open issue on shepherdscientific/ternarycore: "Collaboration proposal:
   machine-checked ternary MAC proofs in Lean 4" — offer t27's algebra as foundation.

---

*Cooperation variants generated by Trinity Agent (Queen) — AEL v2.0*
*Wave Loop 335 | φ² + 1/φ² = 3 | TRINITY*
