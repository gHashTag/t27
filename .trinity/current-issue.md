# Current Issue: Wave Loop 383

**Issue:** #1276
**Branch:** `trinity-rust-rings`
**Basis:** W382 close-out report and W382 cooperation variants (`docs/reports/WAVE_LOOP_382_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **117 waves**, push the Lean 4 generic ∀ lattice to **276**, and extend the W382 array/RAM lowering to cover **ROM-style array-literal constants** and **function-local array variables**.

## Selected variant

**Variant B (recommended)** from W382 cooperation variants:
- Proof push to 276 generic ∀.
- Extend array/RAM lowering:
  - `const lut : [N]T = [N]T{...}` → synthesizable ROM/initialization.
  - `var buf : [N]T` inside a combinational function context.
- Add a ROM/shift-register regression spec.
- Keep 27 IGLA specs green.

## Open strategic context

- Sparkle HDL / Verilean remains the only credible formal competitor (~60 theorems, 0 generic ∀); their Functional Festival 2026 talk is on July 11, 2026.
- Bitstream is ready (`fpga/verilog/ternary_mac_demo_top.bit`) but board flash blocked by missing DLC10 cable.
- W382 landed module-level array/RAM lowering. Remaining sub-gaps are array-literal ROM lowering, function-local arrays, multi-dimensional arrays, and RAM style inference.

## Deliverables

1. 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateSixtyOnePlusGeneric` — 61-variable plus accumulation.
   - `ternaryMacAccumulateSixtyMinusGeneric` — 60-variable minus accumulation lattice.
   - `ternaryMacQuadragintupleDuoCancellationGeneric` — `mac^42(x, a, [.plus,.minus,...]) = x`.
   - `ternaryMacZeroWeightEighteenPairClosureGeneric` — 18 zero + 1 plus + 18 zero.
2. Compiler backend: extend array lowering.
3. New regression spec `specs/scratch/w383_rom_array.t27` (small ROM lookup or shift register).
4. Batch-append W383 blocks to all 27 IGLA specs.
5. Regenerate all affected seals.
6. Full `t27c suite --repo-root .` green.
7. Reports: `WAVE_LOOP_383_REPORT.md`, `WAVE_LOOP_383_COOPERATION.md`, `FPGA_EVIDENCE_W383.md`, update `GEN_VERILOG_DEFECTS_REPRO.md`.
8. Save memory and update `.trinity/experience.md`.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- `t27c suite --repo-root .` returns 0 failures.
- New ROM regression spec passes `yosys read_verilog -sv`.
- Commit closes #1276.

---

*phi² + 1/phi² = 3 | TRINITY*
