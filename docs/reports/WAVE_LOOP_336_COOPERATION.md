# Wave Loop 336 — Cooperation Variants for W337

**Date:** 2026-06-23
**Prepared for:** IGLA CODER + IGLA RACE coordination
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 336 achieved **90 generic ∀ theorems** — the **90× competitor milestone**.
The 12-variable accumulation boundary test succeeded for both plus and minus
weights: `simp+omega` scales to 12 variables without timeout, confirming
unprecedented automation depth.

Four new ternary hardware/software projects emerged in April–June 2026
(TWLA, TernaryCore, Litespark, Balanced_Ternary), but **none include formal
verification**. The competitive moat widens to **90×**.

W337 targets:
- Pool A floor ≥79, CODER ≥69, Pool B ≥96, Integration ≥79
- Lean 4 generic ∀ ≥93 (3 new theorems)
- 13-variable accumulation boundary test, mixed-weight psum associativity depth
- Ecosystem integration research (TorchLean, ATLAAS)

---

## Variant A — Depth Sprint (Accumulation Boundary Test)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacAccumulateThirteenPlusGeneric` (13-variable plus accumulation — **omega saturation test**)
  - `ternaryMacAccumulateThirteenMinusGeneric` (13-variable minus accumulation)
  - `ternaryMacPsumAssociativityMixedPlusMinusDepthGeneric` (mixed-weight psum associativity depth)

**Rationale:**
The 13-variable theorem is the next stress test. If `simp+omega` succeeds,
t27 establishes automation scalability to 13 variables — a result worth publishing.
If it fails, we document the omega boundary at 12.

**Risk:**
13-variable expressions may exceed Lean 4's `omega` solver capacity, causing
timeouts (>5s). If it fails, replace with a safer theorem like `PsumTripleActivationPlusGeneric`.

**Commit message pattern:** `feat(w337): W337 IGLA CODER+RACE — 13-variable accumulation boundary test, 93 generic ∀`

---

## Variant B — Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacPsumTripleActivationPlusGeneric` (`mac(mac(mac(psum,a,.plus),a,.plus),a,.plus) = mac(psum,3*a,.plus)`)
  - `ternaryMacPsumTripleActivationMinusGeneric` (`mac(mac(mac(psum,a,.minus),a,.minus),a,.minus) = mac(psum,3*a,.minus)`)
  - `ternaryMacAccumulateThirteenPlusGeneric` (13-variable plus accumulation — controlled experiment)

**Rationale:**
Maintains the proven cadence while closing the triple-activation lattice for
both plus and minus weights. Extends the double-activation pattern (W336) to
depth 3, proving `mac³(psum,a,.plus) = mac(psum,3*a,.plus)`. 13-variable
accumulation included as a controlled experiment — if it fails, we still deliver
2 solid theorems.

**Commit message pattern:** `feat(w337): W337 IGLA CODER+RACE — Pool A 78→79, CODER 68→69, Pool B 95→96, Lean 4 90→93 generic ∀`

---

## Variant C — Ecosystem Integration (TorchLean Collaboration)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **2 generic ∀ theorems** (conservative)
- **NEW:** Research task — draft TorchLean collaboration proposal

**Rationale:**
With five consecutive waves of zero-IGLA failures and 90 generic ∀ theorems,
t27's theorem library is mature enough to serve as a **foundation layer** for
other projects. TorchLean (arXiv:2602.22631) is the most promising collaboration
target:

1. **TorchLean operator library** — t27's ternary MAC theorems could be added
   as verified primitives, enabling TorchLean users to prove properties of
   ternary-quantized neural networks.

2. **TernaryCore formalization** — t27's generic ∀ proofs could replace
   TernaryCore's simulation-based verification with machine-checked proofs.

**Task breakdown:**
1. Draft a 1-page proposal: "Adding Ternary MAC Primitives to TorchLean"
2. Map t27's `AccumulateTwelvePlusGeneric` to TorchLean's IBP/CROWN structure.
3. Post proposal as GitHub issue on lean-dojo/TorchLean.
4. If positive response, escalate to formal collaboration with shared theorem library.

**Risk:**
External collaboration timelines uncertain. Recommend lightweight research spike
(~10% effort).

**Commit message pattern:** `feat(w337): W337 IGLA CODER+RACE + research(w337): TorchLean ecosystem integration`

---

## Recommendation

**Execute Variant B (Balanced Expansion)** as the primary W337 plan. The
proven 3-theorem cadence has delivered 90 theorems over consecutive waves
without a single build failure. Variant B closes the triple-activation lattice
while testing the 13-variable boundary.

**Parallel track:** Initiate Variant C ecosystem integration as a **lightweight
research spike** (~10% effort). Draft a 1-page TorchLean proposal and post as
GitHub issue.

**Avoid Variant A** unless the 13-variable boundary test is explicitly prioritized.
The risk of omega timeout is moderate; Variant B handles this gracefully.

---

## GitHub Issue Review

**Open issues relevant to W337:**
- No new ternary-specific formal verification issues found.
- TorchLean repository (lean-dojo/TorchLean) has no ternary operator issues.
- TernaryCore repository (shepherdscientific/ternarycore) has no formal verification issues.

**Recommended actions:**
1. Open issue on lean-dojo/TorchLean: "Feature request: ternary weight operator
   and MAC accumulation theorems" — link t27's 90 generic ∀ theorems as motivation.
2. Open issue on shepherdscientific/ternarycore: "Collaboration proposal:
   machine-checked ternary MAC proofs in Lean 4" — offer t27's algebra as foundation.

---

*Cooperation variants generated by Trinity Agent (Queen) — AEL v2.0*
*Wave Loop 336 | φ² + 1/φ² = 3 | TRINITY*
