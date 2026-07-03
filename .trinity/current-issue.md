# Current Issue: Wave Loop 385

**Issue:** #1280
**Branch:** `trinity-rust-rings`
**Basis:** W384 close-out report and W384 cooperation variants (`docs/reports/WAVE_LOOP_384_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **119 waves**, push the Lean 4 `ternaryMac` generic ∀ lattice to **284**, and generalize function-local array lowering to **signed element types** and **array-literal initialization**.

## Selected variant

**Variant B (recommended)** from W384 cooperation variants:
- Proof push to 284 generic ∀.
- Generalize function-local arrays:
  - Signed element types (`[N]i8`, `[N]i16`, etc.).
  - Array initialization at declaration time (`var buf : [4]u16 = [4]u16{0x1111, ...}`).
- Add scratch regression specs for signed and initialized arrays.
- Keep 27 IGLA specs green.

## Open strategic context

- Sparkle HDL / Verilean remains the only credible formal competitor (~60 theorems, 0 generic ∀).
- Bitstream is ready (`fpga/verilog/ternary_mac_demo_top.bit`) but board flash blocked by missing DLC10 cable.
- W384 landed variable-index function-local arrays. Remaining sub-gaps are multi-dimensional arrays and RAM style inference.

## Deliverables

1. 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateSixtyThreePlusGeneric` — 63-variable plus accumulation.
   - `ternaryMacAccumulateSixtyTwoMinusGeneric` — 62-variable minus accumulation lattice.
   - `ternaryMacQuadragintupleQuinqueCancellationGeneric` — `mac^45(x, a, [.plus,.minus,...]) = x`.
   - `ternaryMacZeroWeightTwentyPairClosureGeneric` — 20 zero + 1 plus + 20 zero.
2. Compiler backend: implement array-literal initialization for function-local arrays in `bootstrap/src/compiler.rs`.
3. New regression specs:
   - `specs/scratch/w385_signed_local_array.t27`
   - `specs/scratch/w385_local_array_init.t27`
   - `specs/scratch/w385_signed_local_array_init.t27`
4. Batch-append W385 blocks to all 27 IGLA specs.
5. Regenerate all affected seals.
6. Full `t27c suite --repo-root .` green.
7. Reports: `WAVE_LOOP_385_REPORT.md`, `WAVE_LOOP_385_COOPERATION.md`, `FPGA_EVIDENCE_W385.md`, update `GEN_VERILOG_DEFECTS_REPRO.md`.
8. Save memory and update `.trinity/experience.md`.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- `t27c suite --repo-root .` returns 0 failures.
- New signed/init array regression specs pass `yosys read_verilog -sv`.
- Commit closes #1280.

---

*phi² + 1/phi² = 3 | TRINITY*
