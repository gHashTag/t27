# Wave Loop 358 — Cooperation Variants for W359

**Date:** 2026-06-30
**Source:** WAVE_LOOP_358_REPORT.md
**Next:** Wave Loop 359 (target: July 7, 2026)

---

## Strategic Context

Wave Loop 358 delivered 176 generic ∀ theorems, 34-variable accumulation, and decuple cancellation. The **92-wave zero-IGLA-failure streak** continues.

**Critical blocker discovered:** The `t27c` Verilog backend generates structurally broken output for ternary MAC and 27/36 generated `.v` files. FPGA pre-work (recommended in W357 Variant B) is **impossible** without fixing the backend or creating a hand-written synthesis-ready module.

**Competitive landscape:** Silent since June 23 (ISCA 2026 + SPRIND pitch days). No new generic ∀ ternary theorems published. Trinity's formal moat is **176×**.

**Primary strategic tension:** Fix the Verilog backend (enables FPGA evidence) vs. continue formal depth expansion (maintains competitive moat). W359 must choose.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended for:** Maintaining moat while backend is fixed externally

### W359 Targets

| Metric | W358 → W359 |
|--------|-------------|
| Pool A invariants | 100 → 101 |
| CODER invariants | 90 → 91 |
| Pool B invariants | 118 → 119 |
| Integration invariants | 100 → 101 |
| Lean 4 generic ∀ | 176 → 180 |

### Lean 4 Theorems (4)

1. **35-variable accumulation probe** (`ternaryMacAccumulateThirtyFivePlusGeneric`) — test omega boundary at 35 variables
2. **34-variable minus accumulation** (`ternaryMacAccumulateThirtyFourMinusGeneric`) — maintain dual-polarity parity
3. **Undecuple cancellation** (`ternaryMacUndecupleCancellationGeneric`) — depth-11 alternating identity
4. **Zero-weight reordering closure** (`ternaryMacZeroWeightReorderingClosureGeneric`) — prove any permutation of zero-weight MACs in a chain preserves the final result

### Pros
- Zero risk; proven formula
- Extends accumulation depth record to 35 variables
- Maintains 176× competitive moat

### Cons
- Does not address Verilog backend or silicon evidence gap
- Competitors gain hardware traction while Trinity stays software-only
- Diminishing marginal returns on theorem count alone

---

## Variant B ⭐ — Recommended (Formal Depth + Verilog Backend Fix)

**Risk:** MEDIUM | **Reward:** VERY HIGH | **Recommended for:** Addressing the critical blocker

### W359 Targets

| Metric | W358 → W359 |
|--------|-------------|
| Pool A invariants | 100 → 101 |
| CODER invariants | 90 → 91 |
| Pool B invariants | 118 → 119 |
| Integration invariants | 100 → 101 |
| Lean 4 generic ∀ | 176 → 180 |
| **Verilog deliverable** | **Synthesis-ready ternary MAC module** |

### Lean 4 Theorems (4)

1. **35-variable accumulation probe** (`ternaryMacAccumulateThirtyFivePlusGeneric`) — expected build ~3.0s; if timeout, fallback to 34-variable minus
2. **34-variable minus accumulation** (`ternaryMacAccumulateThirtyFourMinusGeneric`) — dual-polarity parity at depth 34
3. **Undecuple cancellation** (`ternaryMacUndecupleCancellationGeneric`) — depth-11 identity, first undecuple in any framework
4. **Zero-weight reordering closure** (`ternaryMacZeroWeightReorderingClosureGeneric`) — any permutation of zero-weight MACs preserves result

### Verilog Backend Fix (parallel)

**Goal:** Produce a synthesis-ready ternary MAC Verilog module.

**Approach:** Hybrid — minimal backend fix + hand-written module.

**Step 1 — Hand-written ternary MAC module** (fallback, guaranteed to work):
- Create `fpga/verilog/ternary_mac_synth.v`
- Module `ternary_mac_top` with:
  - Inputs: `clk`, `rst_n`, `en`, `a[7:0]`, `w_code[1:0]`, `acc_in[31:0]`
  - Decode `w_code`: `2'b01` → +1, `2'b10` → -1, `2'b00`/`2'b11` → 0
  - Compute `prod = (w_code == 2'b01) ? a : (w_code == 2'b10) ? -$signed(a) : 0`
  - Output: `acc_out = acc_in + prod` (registered)
- Use `fpga/openxc7-synth/test_top.v` as style template
- Add testbench `fpga/verilog/tb_ternary_mac.v` with cocotb or simple `$display` verification

**Step 2 — Synthesis attempt**:
- Run `yosys -p 'read_verilog ternary_mac_synth.v; synth_xilinx -top ternary_mac_top; stat'`
- Extract LUT count, FF count, estimated fmax
- Document in `docs/reports/FPGA_EVIDENCE_W359.md`

**Step 3 — Backend fix probe** (if time permits):
- Investigate `bootstrap/src/compiler.rs` for Verilog generation path
- Fix `MemberAccess` lowering to emit proper field extraction instead of `base_field` concatenation
- Fix variable declaration preservation (`let` locals are silently dropped)
- Target: make `./target/release/t27c gen-verilog specs/igla/race/ternary_mac.t27` produce syntactically valid output

**Fallback chain:**
- If backend fix fails → hand-written module still delivers synthesis metrics
- If `yosys` synthesis fails → document why and retry with simpler module
- If OpenXC7 unavailable → synthesis metrics alone are still valuable evidence

### Pros
- Directly addresses the #1 critical blocker (Verilog backend)
- Produces measurable silicon evidence (LUT count, fmax) even without OpenXC7 bitstream
- Combined 180 generic ∀ + synthesis metrics creates unbeatable narrative
- Maintains zero-IGLA-failure streak on formal side

### Cons
- Verilog backend fix may consume >50% of wave bandwidth
- Risk of synthesis failures or toolchain blockers
- Higher coordination complexity; requires parallel hardware engineering

---

## Variant C — Aggressive (Backend Fix + Full FPGA Sprint + Maximum Formal)

**Risk:** HIGH | **Reward:** VERY HIGH | **Recommended for:** Closing competitive vulnerability in one wave

### W359 Targets

| Metric | W358 → W359 |
|--------|-------------|
| Pool A invariants | 100 → 101 |
| CODER invariants | 90 → 91 |
| Pool B invariants | 118 → 119 |
| Integration invariants | 100 → 101 |
| Lean 4 generic ∀ | 176 → 180 |
| **Verilog deliverable** | **Backend-fixed generated Verilog + hand-written module** |
| **FPGA deliverable** | **Synthesis metrics + bitstream attempt** |

### Lean 4 Theorems (4)

1. **35-variable accumulation probe** (`ternaryMacAccumulateThirtyFivePlusGeneric`)
2. **34-variable minus accumulation** (`ternaryMacAccumulateThirtyFourMinusGeneric`)
3. **Undecuple cancellation** (`ternaryMacUndecupleCancellationGeneric`)
4. **Quadruple mixed-weight zero psum closure** (`ternaryMacQuadrupleMixedWeightZeroPsumClosureGeneric`) — prove `mac(mac(mac(mac(psum, a, .zero), b, .plus), c, .zero), d, .minus) = mac(psum, b - d, .minus)`. First theorem with interleaved zero-weight and mixed-weight operations.

### Verilog Backend Fix (parallel, primary)

**Goal:** Fix `t27c gen-verilog` to produce syntactically valid ternary MAC output.

**Critical fixes needed:**
1. `MemberAccess` lowering: emit `w[1:0]` instead of `w_code` for struct field access
2. Variable declaration preservation: ensure `let decoded = ternary_decode(w);` generates a wire/reg declaration
3. Slice operations: lower `.len()` to a parameter; lower `a[idx]` to proper array indexing
4. Function call lowering: inline `ternary_dot` instead of emitting recursive call

**Acceptance criteria:** `./target/release/t27c gen-verilog specs/igla/race/ternary_mac.t27` produces a file that passes `yosys -p 'read_verilog'` without errors.

### Hand-Written Module (parallel, fallback)

Create `fpga/verilog/ternary_mac_synth.v` as described in Variant B. This ensures synthesis metrics regardless of backend fix outcome.

### FPGA Sprint (parallel)

**Goal:** Attempt bitstream generation for the hand-written ternary MAC module.

**Steps:**
1. Set up OpenXC7 toolchain (install `nextpnr-himbaechel`, `fasm2frames`, `xc7frames2bit`)
2. Create XDC constraint file for QMTech Wukong V1
3. Synthesize ternary MAC with `yosys` + `nextpnr-himbaechel`
4. Generate `.fasm` → `.bit` via `fasm2frames` + `xc7frames2bit`
5. Flash to FPGA via `cli/dlc10` and verify with loopback test
6. Document in `docs/reports/FPGA_EVIDENCE_W359.md`

**Fallback chain:**
- If OpenXC7 setup fails → synthesis metrics from `yosys stat` still valuable
- If bitstream generation fails → synthesis metrics + build log as evidence
- If flashing fails → synthesis metrics + bitstream file as evidence

### Pros
- Maximum pressure on all fronts: formal depth, backend fix, silicon evidence
- If successful, creates an unbeatable competitive narrative (180 generic ∀ + working silicon)
- Positions Trinity for NSF SHF or DARPA Fast and Curious grant applications
- Generates content for arXiv submission

### Cons
- HIGH risk of partial failure due to toolchain complexity
- FPGA sprint may derail the zero-IGLA-failure streak if attention is split too thin
- Requires >50% of wave bandwidth on non-formal tasks
- OpenXC7 setup overhead may exceed one wave

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Formal depth expansion | ✅ | ✅ | ✅ |
| New algebraic dimension | ❌ | ✅ | ✅ |
| Verilog backend fix | ❌ | 🟡 (hybrid) | ✅ |
| Silicon evidence (synthesis) | ❌ | ✅ | ✅ |
| Silicon evidence (bitstream) | ❌ | ❌ | 🟡 |
| Risk of failure | LOW | MEDIUM | HIGH |
| Competitive moat (theorems) | 180× | 180× | 180× |
| Addresses #1 blocker | ❌ | ✅ | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W359.** It balances the low-risk, high-ROI formal theorem expansion with a pragmatic approach to the Verilog backend blocker: create a hand-written synthesis-ready ternary MAC module while probing the backend fix. The 35-variable accumulation probe will test the omega boundary; if it passes, Trinity reaches 35-variable depth — unprecedented. The zero-weight reordering closure theorem adds an 18th proof lattice dimension. Simultaneously, the hand-written Verilog module guarantees synthesis metrics (LUT count, fmax) even if the backend fix fails, beginning to address the silicon evidence gap.

**Trigger for Variant C:** If the hand-written module synthesizes cleanly in W359 and OpenXC7 toolchain setup is straightforward, escalate to Variant C for W360.

**2026 is the year of Lean 4 HDL.** Trinity leads with 176 generic ∀. The Verilog backend is the bridge from software credibility to silicon credibility. Fix it, and Trinity becomes unbeatable.
