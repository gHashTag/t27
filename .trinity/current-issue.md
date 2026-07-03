# Current Issue: Wave Loop 384

**Issue:** #1278
**Branch:** `trinity-rust-rings`
**Basis:** W383 close-out report and W383 cooperation variants (`docs/reports/WAVE_LOOP_383_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **118 waves**, push the Lean 4 `ternaryMac` generic ∀ lattice to **280**, and close the remaining function-local array indexing gap by supporting **non-literal (variable) indices** inside combinational functions.

## Selected variant

**Variant B (recommended)** from W383 cooperation variants:
- Proof push to 280 generic ∀.
- Extend array/RAM lowering: `buf[i]` where `i` is a function parameter or local scalar variable, not only a numeric literal.
- Add a variable-index regression spec.
- Keep 27 IGLA specs green.

## Open strategic context

- Sparkle HDL / Verilean remains the only credible formal competitor (~60 theorems, 0 generic ∀); their Functional Festival 2026 talk is on July 11, 2026.
- Bitstream is ready (`fpga/verilog/ternary_mac_demo_top.bit`) but board flash blocked by missing DLC10 cable.
- W383 landed ROM and function-local numeric-index array lowering. Remaining sub-gaps are variable-index local arrays, multi-dimensional arrays, and RAM style inference.

## Deliverables

1. 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateSixtyTwoPlusGeneric` — 62-variable plus accumulation.
   - `ternaryMacAccumulateSixtyOneMinusGeneric` — 61-variable minus accumulation lattice.
   - `ternaryMacQuadragintupleQuattuorCancellationGeneric` — `mac^44(x, a, [.plus,.minus,...]) = x`.
   - `ternaryMacZeroWeightNineteenPairClosureGeneric` — 19 zero + 1 plus + 19 zero.
2. Compiler backend: extend `ExprIndex` lowering for variable indices on function-local arrays (mux chain fallback).
3. New regression spec `specs/scratch/w384_variable_index.t27`.
4. Batch-append W384 blocks to all 27 IGLA specs.
5. Regenerate all affected seals.
6. Full `t27c suite --repo-root .` green.
7. Reports: `WAVE_LOOP_384_REPORT.md`, `WAVE_LOOP_384_COOPERATION.md`, `FPGA_EVIDENCE_W384.md`, update `GEN_VERILOG_DEFECTS_REPRO.md`.
8. Save memory and update `.trinity/experience.md`.

**Acceptance**
- `lake build Trinity.TernaryInference` passes.
- `t27c suite --repo-root .` returns 0 failures.
- New variable-index regression spec passes `yosys read_verilog -sv`.
- Commit closes #1278.

---

*phi² + 1/phi² = 3 | TRINITY*
