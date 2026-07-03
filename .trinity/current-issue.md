# Current Issue: Wave Loop 382

**Issue:** #1274
**Branch:** `trinity-rust-rings`
**Basis:** W381 close-out report and W381 cooperation variants (`docs/reports/WAVE_LOOP_381_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **116 waves**, push the Lean 4 generic ∀ lattice to **272**, and land the first **incremental array/RAM lowering** capability in the gen-verilog backend.

## Selected variant

**Variant B (recommended)** from W381 cooperation variants:
- Proof push to 272 generic ∀.
- Array/RAM lowering prototype: module-level `var mem : [N]T`, read `mem[i]`, write `mem[i] = x`.
- Datapath regression spec passing yosys smoke.
- Keep 27 IGLA specs green.

## Open strategic context

- Sparkle HDL / Verilean remains the only credible formal competitor (~60 theorems, 0 generic ∀); their Functional Festival 2026 talk is on July 11, 2026.
- Bitstream is ready (`fpga/verilog/ternary_mac_demo_top.bit`) but board flash blocked by missing DLC10 cable.
- W381 closed the last tracked gen-verilog syntax/semantic defect (tuple-return lowering). Next backend target is incremental array/RAM lowering (#1258).

## Deliverables

1. 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateSixtyPlusGeneric` — 60-variable plus accumulation.
   - `ternaryMacAccumulateFiftyNineMinusGeneric` — 59-variable minus accumulation lattice.
   - `ternaryMacDuotrigintupleOctoCancellationGeneric` — `mac^39(x, a, [.plus,.minus,...]) = x`.
   - `ternaryMacZeroWeightSeventeenPairClosureGeneric` — 17 zero + 1 plus + 17 zero.
2. Compiler backend: array/RAM lowering for module-level `var mem : [N]T`.
3. New regression spec `specs/scratch/w382_ram_lowering.t27` (tiny FIFO or single-port memory).
4. Batch-append W382 blocks to all 27 IGLA specs.
5. Regenerate all affected seals.
6. Full `t27c suite --repo-root .` green.
7. Reports: `WAVE_LOOP_382_REPORT.md`, `WAVE_LOOP_382_COOPERATION.md`, `FPGA_EVIDENCE_W382.md`, update `GEN_VERILOG_DEFECTS_REPRO.md`.
8. Save memory and update `.trinity/experience.md`.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- `t27c suite --repo-root .` returns 0 failures.
- New RAM regression spec passes `yosys read_verilog -sv`.
- Commit closes #1274.

---

*phi² + 1/phi² = 3 | TRINITY*
