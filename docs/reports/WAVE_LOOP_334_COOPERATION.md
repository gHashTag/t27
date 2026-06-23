# Wave Loop 334 — Cooperation Variants for W335

**Date:** 2026-06-23  
**Prepared for:** IGLA CODER + IGLA RACE coordination  
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 334 achieved **89 generic ∀ theorems** — the **89× competitor milestone**.
Three new competitors emerged: SC-NeuroCore (Lean 4 neuromorphic hardware, 21 theorems),
EquivFusion (MLIR equivalence checking), and ATLAAS (tensor-level RTL abstraction).
All are non-ternary but validate the Lean 4 + hardware verification thesis.

W335 targets:
- Pool A floor ≥77, CODER ≥67, Pool B ≥95, Integration ≥77
- Lean 4 generic ∀ ≥92 (3 new theorems)
- Scalar associativity minus, 11-variable accumulation, psum commutativity minus

---

## Variant A — Depth Sprint (Accumulation Boundary Test)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacAccumulateElevenPlusGeneric` (11-variable plus accumulation — **omega boundary stress test**)
  - `ternaryMacAccumulateElevenMinusGeneric` (11-variable minus accumulation)
  - `ternaryMacScalarAssociativityMinusGeneric` (scalar associativity for minus weights)

**Rationale:**
The 11-variable accumulation theorem is a strategic experiment. If `simp+omega`
succeeds at depth 11, t27 proves the automation scales beyond 10 — a significant
result for the community. If it fails, we empirically confirm the omega boundary
at depth 10 and can document this as a known limit.

**Risk:**
11-variable expressions may cause `omega` timeouts (>5s) or require `ring_nf`.
Build time could degrade. If the theorem fails, we need a fallback plan.

**Commit message pattern:** `feat(w335): W335 IGLA CODER+RACE — 11-variable accumulation boundary test, 92 generic ∀`

---

## Variant B — Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacScalarAssociativityMinusGeneric` (`mac(mac(0,a,.minus),b,.minus) = mac(0,-(a+b),.minus)`)
  - `ternaryMacPsumCommutativityMinusGeneric` (`mac(mac(psum,a,.minus),b,.minus) = mac(mac(psum,b,.minus),a,.minus)`)
  - `ternaryMacAccumulateElevenPlusGeneric` (11-variable plus accumulation)

**Rationale:**
Maintains the proven cadence while completing the scalar associativity lattice
(minus weights) and extending psum commutativity to minus weights. 11-variable
accumulation is included as a controlled experiment — if it fails, we remove it
and replace with a safer theorem (e.g., `ternaryMacDoubleMinusWeightGeneric`).

**Commit message pattern:** `feat(w335): W335 IGLA CODER+RACE — Pool A 76→77, CODER 66→67, Pool B 93→94, Lean 4 89→92 generic ∀`

---

## Variant C — Ecosystem Integration (ATLAAS + EquivFusion Collaboration)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **2 generic ∀ theorems** (conservative)
- **NEW:** Research task — evaluate ATLAAS and EquivFusion for collaboration potential

**Rationale:**
With three new methodological competitors (SC-NeuroCore, EquivFusion, ATLAAS),
the strategic priority shifts from pure theorem count to **ecosystem integration**.
Rather than competing on generic ∀ count alone, t27 should position itself as the
**algebraic foundation layer** for other projects:
1. **ATLAAS** — t27's ternary MAC theorems could serve as invariants for ATLAAS's
   RTL-to-tensor lifting validation. Propose: "Use ternary MAC algebraic properties
   as correctness invariants for LUT-based accelerator lifting."
2. **EquivFusion** — t27's semiring action theorem could be expressed as an
   equivalence property in EquivFusion's MLIR framework. Propose: "Add ternary MAC
   as a primitive in EquivFusion's MLIR dialect."

**Task breakdown:**
1. Read ATLAAS paper (arXiv:2604.13523) and identify RTL-to-tensor lifting invariants.
2. Draft a note showing how t27's `SemiringActionGeneric` maps to ATLAAS's
   equivalence proof structure.
3. Read EquivFusion paper (arXiv:2604.16571) and identify MLIR dialect extension points.
4. Propose ternary MAC primitive addition to EquivFusion's dialect.

**Risk:**
External collaboration timelines uncertain. Academic authors may not respond to
outreach. Recommend starting with a 1-page proposal and GitHub issue.

**Commit message pattern:** `feat(w335): W335 IGLA CODER+RACE + research(w335): ATLAAS/EquivFusion ecosystem integration`

---

## Recommendation

**Execute Variant B (Balanced Expansion)** as the primary W335 plan. The
proven 3-theorem cadence has delivered 89 theorems over 12 consecutive waves
without a single build failure. Variant B completes the scalar associativity and
psum commutativity lattices while testing the 11-variable boundary.

**Parallel track:** Initiate Variant C ecosystem integration as a **lightweight
research spike** (~10% effort). Draft a 1-page note mapping t27's theorems to
ATLAAS invariants and post it as a GitHub issue or blog post. If positive
response, escalate to formal collaboration.

**Avoid Variant A** unless the 11-variable boundary test is explicitly prioritized.
The risk of omega timeout is moderate; Variant B handles this gracefully by
including it as a controlled experiment with a fallback theorem.

---

*Cooperation variants generated by Trinity Agent (Queen) — AEL v2.0*
