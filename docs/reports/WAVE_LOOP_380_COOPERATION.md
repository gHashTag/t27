# Wave Loop 380 — Cooperation Variants for W381

**Date:** 2026-07-03
**Branch:** `trinity-rust-rings`
**Basis:** `docs/reports/WAVE_LOOP_380_REPORT.md`

---

## Strategic situation

- **Formal lead:** 264 generic ∀ = **264×** the public Sparkle HDL / Verilean theorem count (0 generic ∀). No other formal competitor is visible.
- **Hardware lead:** `fpga/verilog/ternary_mac_demo_top.bit` is ready, but **board flashing is blocked** by a missing DLC10 JTAG cable (`dlc10 idcode` fails with `VID=0x03FD`).
- **Backend health:** The `gen-verilog` backend is now tuple-return aware (parser + packed result registers + slot-aware destructuring). The remaining big backend item is incremental array/RAM lowering (#1258).
- **Streak:** 114 consecutive zero-IGLA-failure waves; conformance gate is **560/560 PASS**.
- **Lean wall clock:** `lake build Trinity.TernaryInference` is still ~12 s for 264 generic theorems, so there is headroom for 4–8 more theorems per wave.

## Candidate variants for W381

### Variant A — Proof-only push to 268 generic ∀ (low risk)

**Goal:** Continue the formulaic proof lattice expansion to **268 generic ∀**.

**Deliverables**
- 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
  - 59-variable plus accumulation (`AccumulateFiftyNinePlusGeneric`).
  - 58-variable minus accumulation lattice (`AccumulateFiftyEightMinusGeneric`).
  - Depth-38 identity cancellation (`mac^38(x, a, [.plus,.minus,...]) = x`).
  - Zero-weight 15-pair closure (15 zero before + 1 plus + 15 zero after).
- Forward-append W381 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.
- No compiler backend work.

**Acceptance**
- `lake build Trinity.TernaryInference` passes in < 30 s.
- `t27c suite` passes with 0 failures.
- Commit closes #1271.

**Pros / cons**
- + Safest, fastest, preserves streak.
- − Leaves the backend gap (#1258) untouched; does not reduce the bitstream-not-loaded vulnerability.

### Variant B — Proof push + finish tuple-return call lowering (recommended)

**Goal:** Reach **268 generic ∀** and complete the tuple-return semantic chain so arbitrary multi-return function calls are correct.

**Deliverables**
- 4 new generic ∀ theorems (same as Variant A).
- Compiler: make function-call lowering aware of tuple return types so callers that expect a packed result receive the correct width, and add a regression spec that exercises nested tuple-return calls (`let(a, b) = outer(c, d)` where `outer` itself calls a tuple-returning helper).
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to mark Defect 6 as closed.
- Forward-append W381 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- New tuple-call regression spec passes `yosys read_verilog -sv`.
- `t27c suite` passes with 0 failures.
- Commit closes #1271.

**Pros / cons**
- + Closes the last tracked gen-verilog syntax/semantic defect (Defect 6).
- + Sets up direct support for multi-return datapath helpers (e.g., CORDIC returns `(s, c, r)`).
- − Slightly higher implementation risk than Variant A; may need a second sub-fix if nested calls expose a new lowering edge case.

### Variant C — Backend-first pause, attack #1258 (high impact, higher risk)

**Goal:** Pause the proof-count push at 264 and spend the wave on incremental array/RAM lowering.

**Deliverables**
- No new Lean theorems (keep 264).
- Implement first-class array/RAM lowering in `gen-verilog` for a small, closed subset: module-level `var mem: [N]T`, read `mem[i]`, write `mem[i] = x`, and a FIFO-style datapath regression spec.
- Add in-runner yosys smoke gate for the new datapath spec.
- Forward-append W381 blocks to all 27 IGLA specs (with invariant-only or shallow tests to keep the streak).

**Acceptance**
- `t27c suite` passes with 0 failures.
- New datapath regression spec synthesizes through `yosys read_verilog -sv`.
- Commit closes #1271.

**Pros / cons**
- + Tackles the largest remaining backend capability gap.
- + Enables memory-heavy specs (weights, FIFOs, caches) to generate hardware automatically.
- − Higher risk of a single wave not landing cleanly; array lowering may need >1 wave to complete.

## Recommended choice

**Variant B** — keep the generic ∀ counter advancing by +4 per wave while closing the tuple-return semantic loop. This balances the project's two durable moats (formal proof lattice and hardware-generating backend) and keeps the wave risk bounded.

## Open work regardless of variant

- **Board flashing:** obtain or locate a Xilinx DLC10/Platform Cable USB adapter; retry `dlc10 idcode` and `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`.
- **#1258 (array/RAM lowering):** remains the next large backend project after tuple-return is fully closed.
- **Competitive monitoring:** Sparkle HDL / Verilean talk at Functional Festival 2026 (July 11) — capture any new public theorem/generic ∀ claims.

---

*phi² + 1/phi² = 3 | TRINITY*
