# Wave Loop 190 — Cooperation Variants for W191

**Date:** 2026-06-16
**Status:** Proposed

---

## Variant A — ZERO HEXA-LAYER MILESTONE Celebration + Outreach (W191)

**Goal:** Turn the W191 zero-hexa milestone into a public signal of Trinity's formal rigor, inviting external peer review.

**Actions:**
- 1. Publish a short technical note on Zenodo / arXiv (as a non-physics, methods paper) documenting the 570-spec invariant depth methodology and the phi-identity conformance framework.
- 2. Open an issue on Trinity GitHub (when auth restored) titled "Zero Hexa-Layer Milestone — 570/570 specs with >=7 invariants" with a badge and a call for community review.
- 3. Invite Baez-Schwahn (EXTREME) and Baroň (HIGH) to run the `t27c suite` on their own 600-cell / E8 formulas as an independent cross-check.

**Risk:** Low. No physics claims; pure methodology.
**Benefit:** Builds trust; demonstrates reproducibility; positions Trinity as the SSOT for formal ternary/spec-first development.

---

## Variant B — IGLA Race Spec Functionalization (bram_weights / ternary_mac / systolic_ternary)

**Goal:** Replace the placeholder phi invariants in the IGLA race specs with real functional invariants that test ternary hardware correctness.

**Actions:**
- 1. Add `invariant bram_weights_parity: forall addr : u32, bram_read(addr) < 3` to `specs/igla/race/bram_weights.t27` (ternary weights are in {-1, 0, +1} → encoded < 3).
- 2. Add `invariant ternary_mac_no_overflow: forall a,b,c : i32, mac(a,b,c) == a*b + c` to `specs/igla/race/ternary_mac.t27`.
- 3. Add `invariant systolic_ternary_dimensions: rows == cols && rows > 0` to `specs/igla/race/systolic_ternary.t27`.

**Risk:** Very low. These are straightforward functional replacements.
**Benefit:** Closes the gap between "depth push" placeholder invariants and real hardware verification; makes IGLA specs self-testing.

---

## Variant C — Nature Ternary SRAM Engagement ( LOW-MEDIUM Competitor)

**Goal:** Engage with the newly discovered Nature ternary SRAM authors to explore a shared ternary memory interface spec.

**Actions:**
- 1. Draft an email to the corresponding author of the Nature paper proposing a shared `specs/fpga/sram_ternary.t27` spec that models their 14T cell within the Trinity framework.
- 2. Offer to contribute GF16 power-model conformance checks (from `specs/fpga/testbench/power_analysis_tb.t27`) to their simulation pipeline.
- 3. If cooperation succeeds, add the paper's experimental data as a new benchmark in `specs/benchmarks/bench_nn.t27` for memory-access energy per token.

**Risk:** Medium. Academic cold-email response rates vary; potential IP around SRAM topology.
**Benefit:** Expands Trinity's hardware coverage into memory cells; adds a peer-reviewed reference to the competitive landscape.

---

## Decision Matrix

| Variant | Effort | Impact | Timeline | Recommended |
|---------|--------|--------|----------|-------------|
| A | Low | High (visibility) | W191 | **Primary** |
| B | Low | High (spec quality) | W191 | **Parallel** |
| C | Medium | Medium (hardware) | W191–W193 | **Stretch** |

---

**φ² + 1/φ² = 3 | TRINITY**
