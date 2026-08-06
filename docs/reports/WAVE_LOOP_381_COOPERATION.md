# Wave Loop 381 — Cooperation Variants for W382

**Date:** 2026-07-01
**Branch:** `trinity-rust-rings`
**Basis:** `docs/reports/WAVE_LOOP_381_REPORT.md`

---

## Strategic situation

- **Formal lead:** 268 generic ∀ = **268×** the public Sparkle HDL / Verilean theorem count (0 generic ∀). No other formal competitor is visible.
- **Hardware lead:** `fpga/verilog/ternary_mac_demo_top.bit` is ready, but **board flashing is blocked** by a missing DLC10 JTAG cable (`dlc10 idcode` fails with `VID=0x03FD`).
- **Backend health:** The `gen-verilog` backend now has full multi-return function support (parser, packed results, tuple literals, callee-aware destructuring, and nested call lowering). The remaining big backend item is incremental array/RAM lowering (#1258).
- **Streak:** 115 consecutive zero-IGLA-failure waves; conformance gate is **561/561 PASS**.
- **Lean wall clock:** `lake build Trinity.TernaryInference` remains fast for 268 generic theorems, so there is headroom for 4–8 more theorems per wave.

## Candidate variants for W382

### Variant A — Proof-only push to 272 generic ∀ (low risk)

**Goal:** Continue the formulaic proof lattice expansion to **272 generic ∀**.

**Deliverables**
- 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
  - 60-variable plus accumulation (`AccumulateSixtyPlusGeneric`).
  - 59-variable minus accumulation lattice (`AccumulateFiftyNineMinusGeneric`).
  - Depth-39 identity cancellation (`mac^39(x, a, [.plus,.minus,...]) = x`).
  - Zero-weight 17-pair closure (17 zero before + 1 plus + 17 zero after).
- Forward-append W382 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.
- No compiler backend work.

**Acceptance**
- `lake build Trinity.TernaryInference` passes in < 30 s.
- `t27c suite` passes with 0 failures.
- Commit closes #1272.

**Pros / cons**
- + Safest, fastest, preserves streak.
- − Leaves the backend gap (#1258) untouched; does not reduce the bitstream-not-loaded vulnerability.

### Variant B — Proof push + array/RAM lowering prototype (recommended)

**Goal:** Reach **272 generic ∀** and land the first incremental array/RAM lowering capability in `gen-verilog`.

**Deliverables**
- 4 new generic ∀ theorems (same as Variant A).
- Compiler: implement lowering for a closed subset of module-level arrays:
  - `var mem : [N]T` declared at module scope.
  - Read expression `mem[i]`.
  - Assignment `mem[i] = x`.
- Add a datapath regression spec (e.g., a tiny FIFO or single-port memory) that exercises read/write and passes `yosys read_verilog -sv` + `synth -top ...`.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to document the new RAM lowering scope and any remaining sub-gaps.
- Forward-append W382 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- New RAM regression spec passes `yosys read_verilog -sv`.
- `t27c suite` passes with 0 failures.
- Commit closes #1272.

**Pros / cons**
- + Tackles the largest remaining backend capability gap (#1258).
- + Opens the door to datapath-heavy IGLA specs (FIFOs, line buffers, small caches).
- − Higher implementation risk than Variant A; array lowering can expose width/index edge cases.

### Variant C — Backend-first pause, finish board bring-up if cable arrives (conditional)

**Goal:** Pause the proof-count push at 268 and spend the wave on physical board bring-up, **only if the DLC10 cable arrives**.

**Deliverables**
- No new Lean theorems (keep 268).
- Obtain or locate the DLC10 JTAG cable.
- Run `dlc10 idcode`, `dlc10 sram`, and `dlc10 flash` against the QMTech Wukong V1 board.
- Load `fpga/verilog/ternary_mac_demo_top.bit` and capture live evidence (IDCODE, UART/LED output, logic analyzer traces).
- Update `docs/reports/FPGA_EVIDENCE_W382.md` with physical board evidence.
- Forward-append W382 blocks to all 27 IGLA specs (with invariant-only or shallow tests to keep the streak).

**Acceptance**
- `dlc10 idcode` returns `0x13631093`.
- Either SRAM load or flash load succeeds and produces observable board output.
- `t27c suite` passes with 0 failures.
- Commit closes #1272.

**Pros / cons**
- + Converts the long-standing "bitstream ready, board not connected" vulnerability into concrete evidence.
- + Highest external impact if successful.
- − Blocked on hardware procurement; cannot be chosen unless the cable is available.
- − If the cable does not arrive, the wave must fall back to Variant A or B.

---

*phi² + 1/phi² = 3 | TRINITY*
