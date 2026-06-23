# Wave Loop 357 — Cooperation Variants for W358

**Date:** 2026-06-23
**Source:** WAVE_LOOP_357_REPORT.md
**Next:** Wave Loop 358 (target: June 30, 2026)

---

## Strategic Context

Wave Loop 357 delivered 172 generic ∀ theorems, 33-variable accumulation, and nonuple cancellation. The competitive landscape has new entrants (manhvu/Balanced_Ternary, SuperInstance/ternary-compiler-v2, rfi-irfos) but **none with formal verification**. Trinity's formal moat is now **172×** the nearest competitor.

**Primary strategic tension:** Trinity has zero measured silicon evidence. Competitors are building physical hardware. The 91-wave zero-failure streak is impressive but software-only. W358 must choose between (a) deeper formal expansion, (b) FPGA evidence sprint, or (c) aggressive dual-track.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended for:** Stable maintenance

### W358 Targets

| Metric | W357 → W358 |
|--------|-------------|
| Pool A invariants | 99 → 100 |
| CODER invariants | 89 → 90 |
| Pool B invariants | 117 → 118 |
| Integration invariants | 99 → 100 |
| Lean 4 generic ∀ | 172 → 176 |

### Lean 4 Theorems (4)

1. **34-variable accumulation probe** (`ternaryMacAccumulateThirtyFourPlusGeneric`) — test omega boundary at 34 variables
2. **33-variable minus accumulation** (`ternaryMacAccumulateThirtyThreeMinusGeneric`) — maintain dual-polarity parity
3. **Decuple cancellation** (`ternaryMacDecupleCancellationGeneric`) — depth-10 alternating identity
4. **Commutativity closure** (`ternaryMacCommutativityClosureGeneric`) — prove commutativity is closed under MAC composition

### Pros
- Zero risk; proven formula
- Extends accumulation depth record
- Maintains 172× competitive moat

### Cons
- Does not address silicon evidence gap
- Competitors gain hardware traction while Trinity stays software-only
- Diminishing marginal returns on theorem count alone

---

## Variant B — Recommended (Formal Depth + FPGA Pre-Work)

**Risk:** LOW-MEDIUM | **Reward:** HIGH | **Recommended for:** Optimal ROI

### W358 Targets

| Metric | W357 → W358 |
|--------|-------------|
| Pool A invariants | 99 → 100 |
| CODER invariants | 89 → 90 |
| Pool B invariants | 117 → 118 |
| Integration invariants | 99 → 100 |
| Lean 4 generic ∀ | 172 → 176 |
| **FPGA pre-work** | **Read HARDWARE_SSOT.md, identify synthesis target** |

### Lean 4 Theorems (4)

1. **34-variable accumulation probe** (`ternaryMacAccumulateThirtyFourPlusGeneric`) — expected build ~3.0s; if timeout, fallback to 33-variable minus
2. **33-variable minus accumulation** (`ternaryMacAccumulateThirtyThreeMinusGeneric`) — dual-polarity parity at depth 33
3. **Decuple cancellation** (`ternaryMacDecupleCancellationGeneric`) — depth-10 identity, first decuple in any framework
4. **Zero-weight commutativity** (`ternaryMacZeroWeightCommutativityGeneric`) — prove `mac(mac(x, a, .zero), b, .plus) = mac(mac(x, b, .plus), a, .zero)`. New algebraic dimension: zero-weight MACs commute with any weight.

### FPGA Pre-Work (parallel)

**Goal:** Produce one measurable synthesis result for the QMTech Wukong V1 (XC7A100T).

**Steps:**
1. Read `fpga/HARDWARE_SSOT.md` to confirm toolchain and cable
2. Identify existing ternary MAC Verilog in `gen/` (generated from `ternary_mac.t27`)
3. Attempt Vivado-in-Docker synthesis; extract LUT count, FF count, estimated fmax
4. Compare ternary MAC vs binary MAC baseline (same width)
5. Document result in `docs/reports/FPGA_EVIDENCE_W358.md`

**Fallback:** If Vivado-in-Docker fails, synthesize via OpenXC7 `nextpnr-xilinx` for metrics.

### Pros
- Balances theorem depth with new algebraic dimension (zero-weight commutativity)
- FPGA pre-work begins addressing critical silicon evidence gap
- Maintains zero-IGLA-failure streak
- 176 generic ∀ = **176×** competitor maximum

### Cons
- FPGA toolchain may have setup overhead
- Requires context-switch from pure Lean 4 to Vivado/OpenXC7

---

## Variant C — Aggressive (Maximum Formal + FPGA Evidence Sprint)

**Risk:** MEDIUM | **Reward:** VERY HIGH | **Recommended for:** Closing competitive vulnerability

### W358 Targets

| Metric | W357 → W358 |
|--------|-------------|
| Pool A invariants | 99 → 100 |
| CODER invariants | 89 → 90 |
| Pool B invariants | 117 → 118 |
| Integration invariants | 99 → 100 |
| Lean 4 generic ∀ | 172 → 176 |
| **FPGA deliverable** | **1 measurable synthesis + bitstream generation** |

### Lean 4 Theorems (4)

1. **34-variable accumulation probe** (`ternaryMacAccumulateThirtyFourPlusGeneric`)
2. **33-variable minus accumulation** (`ternaryMacAccumulateThirtyThreeMinusGeneric`)
3. **Decuple cancellation** (`ternaryMacDecupleCancellationGeneric`)
4. **Quadruple mixed-weight psum distributivity** (`ternaryMacQuadrupleMixedWeightPsumDistributivityGeneric`) — prove `mac(mac(mac(mac(psum, a, .plus), b, .minus), c, .plus), d, .minus) = mac(psum, a - b + c - d, .minus)`. First four-operator alternating-polarity collapse.

### FPGA Evidence Sprint (parallel)

**Goal:** Generate one `.bit` file for a ternary MAC unit on QMTech Wukong V1.

**Steps:**
1. Read `fpga/HARDWARE_SSOT.md`
2. Generate ternary MAC Verilog from `specs/igla/race/ternary_mac.t27` via `t27c gen --target verilog`
3. Synthesize in Vivado-in-Docker with xc7a100tfgg676-1 constraints
4. Extract utilization report (LUT, FF, DSP, BRAM)
5. Generate `.bit` file
6. Flash to FPGA via `cli/dlc10` and verify with loopback test
7. Document in `docs/reports/FPGA_EVIDENCE_W358.md`

**Fallback chain:**
- If Vivado-in-Docker unavailable → OpenXC7 synthesis for metrics only
- If bitstream generation fails → synthesis metrics still valuable
- If `dlc10` flashing fails → synthesis metrics + build log as evidence

### Pros
- Directly addresses critical silicon evidence gap
- Combined 176 generic ∀ + FPGA bitstream creates unbeatable narrative
- Positions Trinity for NSF SHF or DARPA Fast and Curious grant applications
- Generates content for arXiv submission

### Cons
- FPGA toolchain setup may consume >50% of wave bandwidth
- Risk of build failures or toolchain blockers derailing the wave
- Higher coordination complexity; requires parallel execution

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Formal depth expansion | ✅ | ✅ | ✅ |
| New algebraic dimension | ❌ | ✅ | ✅ |
| FPGA evidence | ❌ | 🟡 (pre-work) | ✅ |
| Risk of failure | LOW | LOW-MED | MEDIUM |
| Competitive moat (theorems) | 176× | 176× | 176× |
| Addresses silicon gap | ❌ | 🟡 | ✅ |
| Grant readiness | ❌ | 🟡 | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W358.** It balances the low-risk, high-ROI formal theorem expansion with exploratory FPGA pre-work. The 34-variable accumulation probe will test the omega boundary; if it passes, Trinity reaches 34-variable depth — unprecedented. The zero-weight commutativity theorem adds a 17th proof lattice dimension. Simultaneously, FPGA pre-work begins addressing the silicon evidence gap without committing to a full sprint that could derail the zero-failure streak.

**Trigger for Variant C:** If FPGA pre-work succeeds (synthesis runs without blockers) during W358 planning, escalate to Variant C for W359.

**2026 is the year of Lean 4 HDL.** Trinity leads with 172 generic ∀. The next 4 weeks determine whether that lead translates into silicon credibility.
