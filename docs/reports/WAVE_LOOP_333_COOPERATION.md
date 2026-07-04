# Wave Loop 333 — Cooperation Variants for W334

**Date:** 2026-06-23  
**Prepared for:** IGLA CODER + IGLA RACE coordination  
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 333 achieved **86 generic ∀ theorems** — the omega boundary milestone.
Two new HIGH methodological competitors emerge:
1. **Graphiti** (ASPLOS '26) — Lean 4 formally verified dataflow circuits from EPFL/ETH.
2. **PQC Hardware Masking** (arXiv:2604.18717) — Lean 4 "universal proofs" for hardware accelerators.

Neither is ternary-specific, but both validate the Lean 4 + hardware verification
thesis and could theoretically extend to ternary MAC. t27's 86× generic ∀ lead
remains commanding.

W334 targets:
- Pool A floor ≥76, CODER ≥66, Pool B ≥94, Integration ≥76
- Lean 4 generic ∀ ≥89 (3 new theorems)
- Ring inverse minus weights + scalar associativity
- TorchLean collaboration outreach

---

## Variant A — Defensive Sprint (Lean 4 Prioritized)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **5 generic ∀ theorems** (aggressive) —
  - `ternaryMacRingInverseMinusGeneric` (minus-weight additive inverse)
  - `ternaryMacRingInverseZeroGeneric` (zero-weight additive identity)
  - `ternaryMacScalarAssociativityPlusGeneric` (scalar nesting)
  - `ternaryMacScalarAssociativityMinusGeneric` (scalar nesting)
  - `ternaryMacPsumScalingGeneric` (psum scaling)

**Rationale:**
With Graphiti and PQC universal proofs entering the Lean 4 hardware space, the
strategic priority is to **accelerate theorem production** and maintain the
86× gap. 5 theorems in one wave would push t27 to 91 generic ∀ — a 91×
multiplier that becomes psychologically untouchable for any competitor.

**Risk:**
5 new theorems require diverse proof strategies. May need `ring_nf` or `native_decide`
for some goals, increasing build time. Also increases risk of `simp` argument
accumulation (currently unused `identityWeights` warning already present).

**Commit message pattern:** `feat(w334): W334 IGLA CODER+RACE — defensive sprint, 91 generic ∀, ring structure complete`

---

## Variant B — Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** —
  - `ternaryMacRingInverseMinusGeneric` (`mac(0, -a, .minus) = -mac(0, a, .minus)`)
  - `ternaryMacScalarAssociativityPlusGeneric` (`mac(0, k*a, .plus) = k*mac(0, a, .plus)`)
  - `ternaryMacPsumScalingGeneric` (`mac(k*psum, a, .plus) = ...`)

**Rationale:**
Maintains the proven cadence while completing the ring structure (minus-weight
inverse) and extending scalar properties to nested contexts. This is the
sustainable pace — 3 theorems per wave has been reliable for 10+ consecutive waves.

**Commit message pattern:** `feat(w334): W334 IGLA CODER+RACE — Pool A 75→76, CODER 65→66, Pool B 92→93, Lean 4 86→89 generic ∀`

---

## Variant C — TorchLean Collaboration + Lean Sprint

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **2 generic ∀ theorems** (conservative)
- **NEW:** Contact TorchLean authors + evaluate Graphiti integration potential

**Rationale:**
The two new HIGH threats (Graphiti, PQC masking) are both Lean 4 + hardware
verification projects. Rather than competing purely on theorem count, t27 should
**build alliances** and integrate with complementary projects:
1. **TorchLean** — extend their SSA/DAG IR with ternary MAC primitives.
2. **Graphiti** — evaluate whether t27's ternary MAC theorems can be expressed
   as Graphiti dataflow circuit properties (potential cross-citation).

**Task breakdown:**
1. Open GitHub issue on `lean-dojo/TorchLean`: "Feature request: ternary MAC
   primitive for BitNet b1.58 hardware verification."
2. Read Graphiti paper (ASPLOS '26) and draft a note on how ternary MAC
   accumulation theorems map to dataflow circuit equivalence properties.
3. Cross-reference PQC masking paper — identify if ring-theoretic foundations
   could be reused for ternary MAC ring structure proofs.

**Risk:**
External collaboration timelines are unpredictable. Graphiti authors may not
respond; TorchLean may have architectural constraints preventing custom primitives.

**Commit message pattern:** `feat(w334): W334 IGLA CODER+RACE + research(w334): TorchLean + Graphiti outreach`

---

## Recommendation

**Execute Variant B (Balanced Expansion)** as the primary W334 plan. The
3-theorem-per-wave cadence is sustainable and has delivered 86 theorems without
a single failure. Variant B completes the ring structure and extends scalar
properties — both are natural next steps from W333's ring inverse.

**Parallel track:** Initiate Variant C outreach as a **lightweight diplomatic
spike** (~10% effort). Open the TorchLean GitHub issue; if positive response
within 48 hours, escalate to a proper collaboration proposal. Graphiti evaluation
can be a 1-page summary for internal reference — no need for formal contact
unless TorchLean collaboration succeeds.

**Avoid Variant A** unless a specific competitive alert triggers sprint mode.
5 theorems in one wave risks quality degradation and build instability. The 86×
gap is already commanding; maintainability matters more than marginal speed.

---

*Cooperation variants generated by Trinity Agent (Queen) — AEL v2.0*
