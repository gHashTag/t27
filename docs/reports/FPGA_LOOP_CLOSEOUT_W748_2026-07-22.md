# FPGA Loop Closeout — Wave Loop 748

**Issue:** #1719
**Branch:** `wave-loop-748`
**Date:** 2026-07-22
**Variant:** A — module-scope `[315][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 748 validated a module-scope `[315][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 645,120 bits (~0.616 MiBit, 20,160 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 315 |
| Total elements | 20,160 |
| Packed vector width | 645,120 bits |
| Approximate size | ~0.616 MiBit |
| Generator script | `scripts/gen_w748.py` |
| Witness spec | `specs/scratch/w748_bench_module_315x2p6_aos_var_call_write.t27` |
| Spec lines | ~59,911 |
| Spec size | ~1,379 KB |
| Mid element | `[157][1][0][0][0][0][0]` (element 10,080) |
| Last element | `[314][1][1][1][1][1][1]` (element 20,159) |
| Last raw x | 40,318 (wraps modulo 32,768 → 7,550) |
| Last raw y | 40,319 (wraps modulo 32,768 → 7,551) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 208 passed; 0 failed.
- `t27c parse` W748: PASS.
- `t27c icarus-lowerable` W748: PASS (`lowerable`).
- `t27c icarus-simulate` W748: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W748: PASS (reference-model OK).
- `t27c seal --save` W748: PASS (saved to `.trinity/seals/scratch_w748_bench_module_315x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w748_bench_module_315x2p6_aos_var_call_write.json`.

## What changed

- Added `scripts/gen_w748.py` (copied from `scripts/gen_w747.py`, `OUTER=315`, `MID_IDX=157`).
- Added witness spec `specs/scratch/w748_bench_module_315x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w748_bench_module_315x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-748.md`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.

## Weak points / next ring backlog

- **L1 TRACEABILITY:** 424 commits in the last 30 days still lack `Closes #N`.
  Wave-loop closeout commits now include issue references, but the historical
  backlog remains.
- **L4 TESTABILITY:** 57 `.t27` specs still lack `test`/`invariant`/`bench` blocks.
- **L7 UNITY:** 19 `scripts/*.sh` wrappers remain on the critical path beyond the
  two permitted exceptions, plus 9 untracked `.sh` hooks under `.agents/` and
  `.codex/hooks/`.
- **FPGA SSOT:** `fpga/HARDWARE_SSOT.md` is canonical, but `CLAUDE.md` and stale
  FPGA docs still need alignment with the QMTech Wukong V1 / `cli/dlc10` view.
- **L6 CEILING:** `FORMAT-SPEC-001.json` `GF256.bias_caveat` remains `UNRECONCILED` / `OPEN`.
- **Simulator stress wave:** Plan a Wave Loop near the ~4-MiBit packed-vector
  boundary to measure Icarus/Yosys wall-clock limits.

## Scientific / engineering background

- IEEE 1800-2017 §7.4.1/7.4.3 confirm packed-array total width is the product of
  packed dimensions, with no power-of-two restriction.
- Lutsig (CPP 2021) and CIRCT `HWLegalizeModules` validate flattening nested
  arrays to a single wide packed vector.
- Icarus #1134 and Yosys #2677 / #4653 show that arrays of packed structs are
  fragile; t27's scalar flattening sidesteps both.
- 2025–2026 literature reinforces the packed-vector strategy:
  - **Takahe** — universal open-source synthesis with `--radix 3`, balanced
    ternary cell library, nextpnr iCE40 FPGA export.
  - **Tlsys** — CNFET-based ternary RTL-to-netlist flow with >500k-gate designs.
  - **SONIC / SimulationEngine** — ternary EDA toolchain with Verilog FPGA
    export, REBEL-2 CPU, Basys3 emitter.
  - **Trinity B002** — zero-DSP balanced-ternary inference architecture for
    Xilinx 7-series, open-source Yosys/NextPNR-Xilinx/OpenXC7 flow, QMTech-board
    support, aligning with t27's FPGA SSOT target.

## Cooperation variants for Wave Loop 749

1. **Variant A (recommended): `[317][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 649,216-bit packed vector, 20,288 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 317.
   - **Recommended.**

2. **Variant B: `[315][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W748 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[315][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.616 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W748 committed with `Closes #1719`; branch `wave-loop-749` created.
