# Wave Loop 359 — Cooperation Variants for W360

**Date:** 2026-07-02
**Source:** WAVE_LOOP_359_REPORT.md
**Next:** Wave Loop 360 (target: July 9, 2026)

---

## Strategic Context

Wave Loop 359 delivered 180 generic ∀ theorems, 35-variable accumulation, duodecuple cancellation, zero-weight reordering closure, and the **first synthesis-ready ternary MAC** with `yosys` metrics.

The **92-wave zero-IGLA-failure streak** is now **93 waves**.

**Primary strategic tension:** Continue formal depth expansion (maintains moat) vs. push the hand-written MAC through OpenXC7 bitstream generation (closes silicon gap). W360 must choose.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended for:** Stable maintenance while backend/toolchain is fixed externally

### W360 Targets

| Metric | W359 → W360 |
|--------|-------------|
| Pool A invariants | 101 → 102 |
| CODER invariants | 91 → 92 |
| Pool B invariants | 119 → 120 |
| Integration invariants | 101 → 102 |
| Lean 4 generic ∀ | 180 → 184 |

### Lean 4 Theorems (4)

1. **36-variable accumulation probe** (`ternaryMacAccumulateThirtySixPlusGeneric`) — test omega boundary at 36 variables
2. **35-variable minus accumulation** (`ternaryMacAccumulateThirtyFiveMinusGeneric`) — maintain dual-polarity parity
3. **Tredecuple cancellation** (`ternaryMacTredecupleCancellationGeneric`) — depth-13 alternating identity (collapses to `mac(x, a, .plus)`)
4. **Zero-weight triple closure** (`ternaryMacZeroWeightTripleClosureGeneric`) — prove any permutation of three zero-weight MACs in a chain preserves result

### Pros
- Zero risk; proven formula
- Extends accumulation depth record to 36 variables
- Maintains 180× competitive moat

### Cons
- Does not address Verilog backend or silicon evidence gap
- Competitors gain hardware traction while Trinity stays software-only
- Leaves the highest-value blocker untouched

---

## Variant B ⭐ — Recommended (Formal Depth + OpenXC7 Bitstream Attempt)

**Risk:** MEDIUM-HIGH | **Reward:** VERY HIGH | **Recommended for:** Closing the silicon credibility gap

### W360 Targets

| Metric | W359 → W360 |
|--------|-------------|
| Pool A invariants | 101 → 102 |
| CODER invariants | 91 → 92 |
| Pool B invariants | 119 → 120 |
| Integration invariants | 101 → 102 |
| Lean 4 generic ∀ | 180 → 184 |
| **FPGA deliverable** | **Bitstream attempt for ternary_mac_top** |

### Lean 4 Theorems (4)

1. **36-variable accumulation probe** (`ternaryMacAccumulateThirtySixPlusGeneric`) — expected build ~3.4s; if timeout, fallback to 35-variable minus
2. **35-variable minus accumulation** (`ternaryMacAccumulateThirtyFiveMinusGeneric`) — dual-polarity parity at depth 35
3. **Tredecuple cancellation** (`ternaryMacTredecupleCancellationGeneric`) — depth-13 identity-collapse theorem
4. **Zero-weight triple closure** (`ternaryMacZeroWeightTripleClosureGeneric`) — 19th proof-lattice dimension

### OpenXC7 Bitstream Sprint (parallel)

**Goal:** Generate and (if possible) flash a `.bit` for `ternary_mac_top` on the QMTech Wukong V1.

**Steps:**
1. Install OpenXC7 toolchain per `fpga/HARDWARE_SSOT.md` §8:
   - `yosys` (already installed)
   - `nextpnr-himbaechel` or `nextpnr-xilinx` (stable-backports)
   - `fasm2frames`
   - `xc7frames2bit`
2. Create `fpga/verilog/ternary_mac_top.xdc` with QMTech Wukong V1 pin constraints
3. Synthesize with `yosys` + `nextpnr-xilinx`
4. Generate `.fasm` → `.frames` → `.bit`
5. Document in `docs/reports/FPGA_EVIDENCE_W360.md`
6. Optional: flash via `cli/dlc10 sram ternary_mac_top.bit` if board is connected

**Fallback chain:**
- If OpenXC7 install fails → synthesis metrics from W359 are still evidence
- If placement/routing fails → document why and retry with narrower accumulator width
- If flashing fails → bitstream file + synthesis log as evidence

### Pros
- Directly addresses the #1 competitive vulnerability (no silicon evidence)
- Pushes the hand-written MAC from synthesis metrics to a real bitstream
- Maintains zero-IGLA-failure streak on formal side
- 184 generic ∀ + working bitstream creates an unbeatable narrative

### Cons
- OpenXC7 setup may consume >50% of wave bandwidth
- Toolchain installation can fail on macOS arm64
- May split attention from formal work

---

## Variant C — Aggressive (Maximum Formal + Backend Fix + Full FPGA)

**Risk:** HIGH | **Reward:** VERY HIGH | **Recommended for:** Closing all gaps in one wave

### W360 Targets

| Metric | W359 → W360 |
|--------|-------------|
| Pool A invariants | 101 → 102 |
| CODER invariants | 91 → 92 |
| Pool B invariants | 119 → 120 |
| Integration invariants | 101 → 102 |
| Lean 4 generic ∀ | 180 → 184 |
| **Verilog deliverable** | **Backend-fixed generated ternary MAC** |
| **FPGA deliverable** | **Bitstream for hand-written or generated module** |

### Lean 4 Theorems (4)

1. **36-variable accumulation probe** (`ternaryMacAccumulateThirtySixPlusGeneric`)
2. **35-variable minus accumulation** (`ternaryMacAccumulateThirtyFiveMinusGeneric`)
3. **Tredecuple cancellation** (`ternaryMacTredecupleCancellationGeneric`)
4. **Quadruple mixed-weight zero closure** (`ternaryMacQuadrupleMixedWeightZeroClosureGeneric`) — interleaved zero-weight + mixed-weight permutation theorem

### Verilog Backend Fix (parallel, primary)

**Goal:** Fix `t27c gen-verilog` to produce syntactically valid ternary MAC output.

**Critical fixes:**
1. `MemberAccess` lowering: emit proper field extraction
2. Variable declaration preservation: `let` locals must become wire/reg declarations
3. Slice/index operations: lower `.len()` to parameter, `a[idx]` to Verilog array indexing
4. Function call lowering: inline `ternary_dot` instead of recursive call

**Acceptance criteria:** `./target/release/t27c gen-verilog specs/igla/race/ternary_mac.t27` produces a file that passes `yosys -p 'read_verilog'`.

### Hand-Written Module + OpenXC7 (parallel, fallback)

Use `fpga/verilog/ternary_mac_synth.v` from W359 and push through OpenXC7 bitstream flow as described in Variant B. This guarantees silicon evidence regardless of backend fix outcome.

### Pros
- Maximum pressure on all fronts: formal depth, backend fix, silicon evidence
- If successful, creates an unbeatable competitive narrative (184 generic ∀ + generated + silicon)
- Positions Trinity for NSF SHF / DARPA grant applications

### Cons
- HIGH risk of partial failure due to toolchain/backend complexity
- FPGA sprint may derail formal work if attention is split too thin
- Backend fix may exceed one wave

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Formal depth expansion | ✅ | ✅ | ✅ |
| New algebraic dimension | ✅ | ✅ | ✅ |
| Verilog backend fix | ❌ | ❌ | ✅ |
| Synthesis metrics (existing) | ✅ (W359) | ✅ (W359) | ✅ (W359) |
| Silicon evidence (bitstream) | ❌ | ✅ | ✅ |
| Risk of failure | LOW | MEDIUM-HIGH | HIGH |
| Competitive moat (theorems) | 184× | 184× | 184× |
| Addresses #1 blocker | ❌ | ✅ | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W360.** It balances the low-risk, high-ROI formal theorem expansion with a pragmatic push toward real silicon evidence: install OpenXC7 and attempt bitstream generation for the hand-written ternary MAC. The 36-variable accumulation probe will test the omega boundary; if it passes, Trinity reaches 36-variable depth — unprecedented. The zero-weight triple closure theorem adds a 19th proof lattice dimension. Simultaneously, the OpenXC7 sprint moves the project from synthesis metrics to a real `.bit` file.

**Trigger for Variant C:** If the OpenXC7 toolchain installs cleanly and the hand-written module routes quickly, escalate to Variant C by also probing the Verilog backend fix in `bootstrap/src/compiler.rs`.

**2026 is the year of Lean 4 HDL.** Trinity leads with 180 generic ∀. The hand-written MAC is the bridge from software credibility to silicon credibility. Cross it in W360.
