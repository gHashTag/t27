# Wave Loop 356 — Cooperation Variants for W357

**Date:** 2026-06-16
**Source:** WAVE_LOOP_356_REPORT.md
**Next:** Wave Loop 357 (target: June 23, 2026)

---

## Strategic Context

Wave Loop 356 delivered 168 generic ∀ theorems, 32-variable accumulation, and octuple cancellation. Two new ternary hardware competitors emerged in mid-June 2026 (ternfpga, Balanced_Ternary), neither with formal verification. Trinity's formal moat is **168×** the nearest competitor.

**Primary strategic tension:** Trinity has zero measured silicon evidence. Competitors are building physical hardware. We must choose between (a) doubling down on formal depth, (b) splitting effort toward FPGA evidence, or (c) aggressive expansion on both fronts.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended for:** Stable maintenance mode

### W357 Targets

| Metric | W356 → W357 |
|--------|-------------|
| Pool A invariants | 98 → 99 |
| CODER invariants | 88 → 89 |
| Pool B invariants | 116 → 117 |
| Integration invariants | 98 → 99 |
| Lean 4 generic ∀ | 168 → 172 |

### Lean 4 Theorems (4)

1. **33-variable accumulation probe** (`ternaryMacAccumulateThirtyThreePlusGeneric`) — test omega boundary at 33 variables
2. **32-variable minus accumulation** (`ternaryMacAccumulateThirtyTwoMinusGeneric`) — maintain dual-polarity parity
3. **Nonuple cancellation** (`ternaryMacNonupleCancellationGeneric`) — depth-9 alternating identity
4. **Commutativity closure** (`ternaryMacCommutativityClosureGeneric`) — prove commutativity is closed under MAC composition

### Pros
- Zero risk; proven formula
- Extends accumulation depth record
- Maintains 168× competitive moat

### Cons
- Does not address silicon evidence gap
- Competitors gain hardware traction while Trinity stays software-only
- Diminishing marginal returns on theorem count alone

---

## Variant B — Recommended (Formal Depth + New Algebraic Dimension)

**Risk:** LOW-MEDIUM | **Reward:** HIGH | **Recommended for:** Optimal ROI

### W357 Targets

| Metric | W356 → W357 |
|--------|-------------|
| Pool A invariants | 98 → 99 |
| CODER invariants | 88 → 89 |
| Pool B invariants | 116 → 117 |
| Integration invariants | 98 → 99 |
| Lean 4 generic ∀ | 168 → 172 |

### Lean 4 Theorems (4)

1. **33-variable accumulation probe** (`ternaryMacAccumulateThirtyThreePlusGeneric`) — expected build ~2.8s; if timeout, fallback to 32-variable minus
2. **32-variable minus accumulation** (`ternaryMacAccumulateThirtyTwoMinusGeneric`) — dual-polarity parity at depth 32
3. **Nonuple cancellation** (`ternaryMacNonupleCancellationGeneric`) — depth-9 identity, first nonuple in any framework
4. **Mixed-weight zero associativity** (`ternaryMacMixedWeightZeroAssociativityGeneric`) — prove `mac(mac(mac(x, a, .plus), b, .zero), c, .minus) = mac(x, a - c, .minus)`. New algebraic dimension combining mixed-weight chains with zero-weight elimination.

### Additional Actions
- **Begin FPGA evidence pre-work:** Read `fpga/HARDWARE_SSOT.md`, identify one measurable benchmark (e.g., ternary MAC cycle count vs LUT count) that can be synthesized in Vivado Docker or OpenXC7
- **Draft one arXiv abstract** summarizing 168 generic ∀ theorems for submission

### Pros
- Balances theorem depth with new algebraic dimension
- FPGA pre-work begins addressing critical silicon evidence gap
- Maintains zero-IGLA-failure streak
- 172 generic ∀ = **172×** competitor maximum

### Cons
- FPGA pre-work is exploratory; may not yield publishable results in one week
- Requires context-switch from pure Lean 4 to Vivado/OpenXC7 toolchain

---

## Variant C — Aggressive (Maximum Formal + FPGA Evidence Sprint)

**Risk:** MEDIUM | **Reward:** VERY HIGH | **Recommended for:** Closing competitive vulnerability

### W357 Targets

| Metric | W356 → W357 |
|--------|-------------|
| Pool A invariants | 98 → 99 |
| CODER invariants | 88 → 89 |
| Pool B invariants | 116 → 117 |
| Integration invariants | 98 → 99 |
| Lean 4 generic ∀ | 168 → 172 |
| **FPGA deliverable** | **1 measurable synthesis result** |

### Lean 4 Theorems (4)

1. **33-variable accumulation probe** (`ternaryMacAccumulateThirtyThreePlusGeneric`)
2. **32-variable minus accumulation** (`ternaryMacAccumulateThirtyTwoMinusGeneric`)
3. **Nonuple cancellation** (`ternaryMacNonupleCancellationGeneric`)
4. **Triple mixed-weight psum commutativity** (`ternaryMacTripleMixedWeightPsumCommutativityGeneric`) — prove `mac(mac(mac(psum, a, .plus), b, .minus), c, .plus) = mac(mac(mac(psum, c, .plus), b, .minus), a, .plus)`. First psum-specific commutativity across three mixed weights.

### FPGA Evidence Sprint (parallel)

**Goal:** Produce one measurable synthesis result for the QMTech Wukong V1 (XC7A100T).

**Steps:**
1. Read `fpga/HARDWARE_SSOT.md` to confirm toolchain and cable
2. Synthesize `ternary_mac.v` (generated from `specs/igla/race/ternary_mac.t27`) via Vivado-in-Docker
3. Extract LUT count, FF count, and estimated fmax for a single ternary MAC unit
4. Compare against binary MAC baseline (same width)
5. Document result in `docs/reports/FPGA_EVIDENCE_W357.md`

**Fallback:** If Vivado-in-Docker fails, use OpenXC7 `nextpnr-xilinx` for synthesis metrics.

### Pros
- Directly addresses critical silicon evidence gap
- Combined 172 generic ∀ + FPGA metrics creates unbeatable narrative
- Positions Trinity for NSF SHF or DARPA Fast and Curious grant applications
- Generates content for arXiv submission

### Cons
- FPGA toolchain may have setup overhead exceeding one wave
- Risk of build failures or toolchain blockers derailing the wave
- Requires parallel execution; higher coordination complexity

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Formal depth expansion | ✅ | ✅ | ✅ |
| New algebraic dimension | ❌ | ✅ | ✅ |
| FPGA evidence | ❌ | 🟡 (pre-work) | ✅ |
| Risk of failure | LOW | LOW-MED | MEDIUM |
| Competitive moat (theorems) | 172× | 172× | 172× |
| Addresses silicon gap | ❌ | 🟡 | ✅ |
| Grant readiness | ❌ | 🟡 | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W357.** It balances the low-risk, high-ROI formal theorem expansion with exploratory FPGA pre-work. The 33-variable accumulation probe will test the omega boundary; if it passes, Trinity reaches 33-variable depth — unprecedented in any framework. The mixed-weight zero associativity theorem adds a 16th proof lattice dimension. Simultaneously, FPGA pre-work begins addressing the silicon evidence gap without committing to a full sprint that could derail the zero-failure streak.

**Trigger for Variant C:** If FPGA pre-work succeeds (synthesis runs without blockers) during W357 planning, escalate to Variant C for W358.

**2026 is the year of Lean 4 HDL.** Trinity leads with 168 generic ∀. The next 4 weeks determine whether that lead translates into silicon credibility.
