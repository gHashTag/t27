# Wave Loop 382 — Cooperation Variants for W383

**Date:** 2026-07-01
**Branch:** `trinity-rust-rings`
**Basis:** `docs/reports/WAVE_LOOP_382_REPORT.md`

---

## Strategic situation

- **Formal lead:** 272 generic ∀ = **272×** the public Sparkle HDL / Verilean theorem count (0 generic ∀). No other formal competitor is visible.
- **Hardware lead:** `fpga/verilog/ternary_mac_demo_top.bit` is ready, but **board flashing is blocked** by a missing DLC10 JTAG cable (`dlc10 idcode` fails with `VID=0x03FD`).
- **Backend health:** `gen-verilog` now supports module-level array/RAM lowering for `var mem : [N]T`, read `mem[i]`, write `mem[i] = x`. Remaining sub-gaps include function-local arrays, array literals in expression context, multi-dimensional arrays, and inferred RAM style (distributed vs. block). The next large backend target remains complete array/RAM support (#1258).
- **Streak:** 116 consecutive zero-IGLA-failure waves; conformance gate is **562/562 PASS**.
- **Lean wall clock:** `lake build Trinity.TernaryInference` remains fast for 272 generic theorems, so there is headroom for 4–8 more theorems per wave.

## Candidate variants for W383

### Variant A — Proof-only push to 276 generic ∀ (low risk)

**Goal:** Continue the formulaic proof lattice expansion to **276 generic ∀**.

**Deliverables**
- 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
  - 61-variable plus accumulation (`AccumulateSixtyOnePlusGeneric`).
  - 60-variable minus accumulation lattice (`AccumulateSixtyMinusGeneric`).
  - Depth-42 identity cancellation (`mac^42(x, a, [.plus,.minus,...]) = x`).
  - Zero-weight 18-pair closure (18 zero before + 1 plus + 18 zero after).
- Forward-append W383 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.
- No compiler backend work.

**Acceptance**
- `lake build Trinity.TernaryInference` passes in < 30 s.
- `t27c suite` passes with 0 failures.
- Commit closes #1274.

**Pros / cons**
- + Safest, fastest, preserves streak.
- − Leaves the remaining array/RAM sub-gaps untouched; does not reduce the bitstream-not-loaded vulnerability.

### Variant B — Proof push + extend array/RAM lowering (recommended)

**Goal:** Reach **276 generic ∀** and extend the W382 RAM prototype to cover array literals as initializers and function-local array variables.

**Deliverables**
- 4 new generic ∀ theorems (same as Variant A).
- Compiler: extend array lowering so that:
  - `const lut : [N]T = [N]T{...}` emits a synthesizable ROM/initialization pattern.
  - Function-local `var buf : [N]T` is supported inside a combinational function context (emitted as a packed temporary or per-element regs with index access).
- Add a second regression spec exercising a small ROM lookup or shift register built on array literals.
- Forward-append W383 blocks to all 27 IGLA specs.
- Full `t27c suite` green, all seals regenerated.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- New ROM/array-literal regression spec passes `yosys read_verilog -sv`.
- `t27c suite` passes with 0 failures.
- Commit closes #1274.

**Pros / cons**
- + Continues the backend expansion started in W382.
- + Makes arrays usable in more spec patterns (lookup tables, small buffers).
- − Higher implementation risk than Variant A; function-local arrays in Verilog functions have synthesis constraints.

### Variant C — Hardware-first pause, retry board bring-up if cable arrives (conditional)

**Goal:** Pause the proof-count push at 272 and spend the wave on physical board bring-up, **only if the DLC10 cable arrives**.

**Deliverables**
- No new Lean theorems (keep 272).
- No new compiler backend work.
- Obtain or locate the DLC10 JTAG cable.
- Run `dlc10 idcode`, `dlc10 sram`, and `dlc10 flash` against the QMTech Wukong V1 board.
- Load `fpga/verilog/ternary_mac_demo_top.bit` and capture live evidence (IDCODE, LED/UART output, logic analyzer traces).
- Update `docs/reports/FPGA_EVIDENCE_W383.md` with physical board evidence.
- Forward-append W383 blocks to all 27 IGLA specs (with invariant-only or shallow tests to keep the streak).

**Acceptance**
- `dlc10 idcode` returns `0x13631093`.
- Either SRAM load or flash load succeeds and produces observable board output.
- `t27c suite` passes with 0 failures.
- Commit closes #1274.

**Pros / cons**
- + Converts the long-standing "bitstream ready, board not connected" vulnerability into concrete evidence.
- + Highest external impact if successful.
- − Blocked on hardware procurement; cannot be chosen unless the cable is available.
- − If the cable does not arrive, the wave must fall back to Variant A or B.

---

*phi² + 1/phi² = 3 | TRINITY*
