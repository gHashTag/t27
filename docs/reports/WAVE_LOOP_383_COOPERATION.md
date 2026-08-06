# Wave Loop 383 — Cooperation Variants for W384

**Date:** 2026-07-01
**Branch:** `trinity-rust-rings`
**Basis:** `docs/reports/WAVE_LOOP_383_REPORT.md`

---

## Strategic situation

- **Formal lead:** 276 `ternaryMac` generic ∀ = **276×** the public Sparkle HDL / Verilean theorem count (0 generic ∀). No other formal competitor is visible.
- **Hardware lead:** `fpga/verilog/ternary_mac_demo_top.bit` is ready, but **board flashing is blocked** by a missing DLC10 JTAG cable (`dlc10 idcode` fails with `VID=0x03FD`).
- **Backend health:** `gen-verilog` now supports:
  - Module-level `var mem : [N]T` memory read/write (W382).
  - Module-level `const lut : [N]T = [N]T{...}` ROM/initialization (W383).
  - Function-local `var buf : [N]T` with numeric-literal index access (W383).
  - Remaining sub-gaps: multi-dimensional arrays, non-literal dynamic indices on local arrays, inferred RAM style (distributed vs. block), and array literals in expression context.
- **Streak:** 117 consecutive zero-IGLA-failure waves; conformance gate is **563/563 PASS**.
- **Lean wall clock:** `lake build Trinity.TernaryInference` remains fast for 276 generic theorems, so there is headroom for 4–8 more theorems per wave.

## Candidate variants for W384

### Variant A — Proof-only push to 280 `ternaryMac` generic ∀ (low risk)

**Goal:** Continue the formulaic proof lattice expansion to **280 `ternaryMac` generic ∀**.

**Deliverables**
- 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
  - 62-variable plus accumulation (`AccumulateSixtyTwoPlusGeneric`).
  - 61-variable minus accumulation lattice (`AccumulateSixtyOneMinusGeneric`).
  - Depth-44 identity cancellation (`mac^44(x, a, [.plus,.minus,...]) = x`).
  - Zero-weight 19-pair closure (19 zero before + 1 plus + 19 zero after).
- Forward-append W384 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.
- No compiler backend work.

**Acceptance**
- `lake build Trinity.TernaryInference` passes in < 30 s.
- `t27c suite` passes with 0 failures.
- Commit closes #1276.

**Pros / cons**
- + Safest, fastest, preserves streak.
- − Leaves the remaining array/RAM sub-gaps untouched; does not reduce the bitstream-not-loaded vulnerability.

### Variant B — Proof push + complete local array indexing (recommended)

**Goal:** Reach **280 `ternaryMac` generic ∀** and close the remaining function-local array indexing gap by supporting non-literal (variable) indices inside combinational functions.

**Deliverables**
- 4 new generic ∀ theorems (same as Variant A).
- Compiler: extend `ExprIndex` lowering so that `buf[i]` works when `i` is a function parameter or local scalar variable, not only a numeric literal. Options:
  - Emit a small case statement or indexed memory inside a Verilog function if the tool supports it.
  - Or flatten to per-element regs and a conditional mux chain (`i == 0 ? buf_0 : (i == 1 ? buf_1 : ...)`).
- Add a regression spec exercising variable-index read/write on a function-local array.
- Forward-append W384 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- New variable-index regression spec passes `yosys read_verilog -sv`.
- `t27c suite` passes with 0 failures.
- Commit closes #1276.

**Pros / cons**
- + Finishes the function-local array story started in W383.
- + Makes small datapath buffers and lookup tables usable in more spec patterns.
- − Higher implementation risk than Variant A; variable indexing inside Verilog functions has synthesis constraints.

### Variant C — Hardware-first pause, retry board bring-up if cable arrives (conditional)

**Goal:** Pause the proof-count push at 276 and spend the wave on physical board bring-up, **only if the DLC10 cable arrives**.

**Deliverables**
- No new Lean theorems (keep 276).
- No new compiler backend work.
- Obtain or locate the Xilinx DLC10/Platform Cable USB adapter.
- Run `dlc10 idcode`, `dlc10 sram`, and `dlc10 flash` against the QMTech Wukong V1 board.
- Load `fpga/verilog/ternary_mac_demo_top.bit` and capture live evidence (IDCODE, LED/UART output, logic analyzer traces).
- Update `docs/reports/FPGA_EVIDENCE_W384.md` with physical board evidence.
- Forward-append W384 blocks to all 27 IGLA specs (with invariant-only or shallow tests to keep the streak).

**Acceptance**
- `dlc10 idcode` returns `0x13631093`.
- Either SRAM load or flash load succeeds and produces observable board output.
- `t27c suite` passes with 0 failures.
- Commit closes #1276.

**Pros / cons**
- + Converts the long-standing "bitstream ready, board not connected" vulnerability into concrete evidence.
- + Highest external impact if successful.
- − Blocked on hardware procurement; cannot be chosen unless the cable is available.
- − If the cable does not arrive, the wave must fall back to Variant A or B.

---

*phi² + 1/phi² = 3 | TRINITY*
