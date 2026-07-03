# Current Issue: Wave Loop 386

**Issue:** #1282  
**Branch:** `wave-loop-385`  
**Basis:** W385 close-out report and W385 cooperation variants (`docs/reports/WAVE_LOOP_385_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **120 waves**, push the Lean 4 `ternaryMac` generic ∀ lattice to **288**, and add **gen-verilog `for` loop lowering over function-local arrays**.

## Selected variant

**Variant B (recommended)** from W385 cooperation variants:
- Proof push to 288 generic ∀.
- Close the function-local array control-flow gap: `for` loops that read from and write to local arrays.
- Add scratch regression specs exercising for-loop iteration with local arrays.
- Keep 27 IGLA specs green.

## Open strategic context

- Sparkle HDL / Verilean remains the only credible formal competitor (~60 theorems, 0 generic ∀).
- FPGA bring-up is now unblocked: W385 successfully loaded a 200T-compatible bitstream via `openFPGALoader -c digilent_hs2` (`done 1`).
- W385 completed signed/init function-local arrays. Remaining sub-gaps are multi-dimensional arrays and RAM style hints.

## Deliverables

1. 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateSixtyFourPlusGeneric` — 64-variable plus accumulation.
   - `ternaryMacAccumulateSixtyThreeMinusGeneric` — 63-variable minus accumulation lattice.
   - `ternaryMacQuadragintupleSexCancellationGeneric` — `mac^46(x, a, [.plus,.minus,...]) = x`.
   - `ternaryMacZeroWeightTwentyOnePairClosureGeneric` — 21 zero + 1 plus + 21 zero.
2. Compiler backend: implement `for` loop lowering that can update function-local arrays in `bootstrap/src/compiler.rs`.
3. New regression specs:
   - `specs/scratch/w386_for_local_array.t27`
   - `specs/scratch/w386_for_local_array_i8.t27` (signed element iteration)
4. Batch-append W386 blocks to all 27 IGLA specs.
5. Regenerate all affected seals.
6. Full `t27c suite --repo-root .` green.
7. Reports: `WAVE_LOOP_386_REPORT.md`, `WAVE_LOOP_386_COOPERATION.md`, `FPGA_EVIDENCE_W386.md`, update `GEN_VERILOG_DEFECTS_REPRO.md`.
8. Save memory and update `.trinity/experience.md`.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- `t27c suite --repo-root .` returns 0 failures.
- New for-loop regression specs pass `yosys read_verilog -sv`.
- Commit closes #1282.

---

*φ² + 1/φ² = 3 | TRINITY*
