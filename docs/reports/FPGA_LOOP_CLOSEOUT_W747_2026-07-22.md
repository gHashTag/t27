# FPGA Loop Closeout — Wave Loop 747

**Issue:** #1718
**Branch:** `wave-loop-747`
**Date:** 2026-07-22
**Variant:** A — module-scope `[313][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 747 validated a module-scope `[313][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 641,024 bits (~0.612 MiBit, 20,032 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 313 |
| Total elements | 20,032 |
| Packed vector width | 641,024 bits |
| Approximate size | ~0.612 MiBit |
| Generator script | `scripts/gen_w747.py` |
| Witness spec | `specs/scratch/w747_bench_module_313x2p6_aos_var_call_write.t27` |
| Spec lines | ~59,531 |
| Spec size | ~1,371 KB |
| Mid element | `[156][1][0][0][0][0][0]` (element 10,016) |
| Last element | `[312][1][1][1][1][1][1]` (element 20,031) |
| Last raw x | 40,062 (wraps modulo 32,768 → 7,294) |
| Last raw y | 40,063 (wraps modulo 32,768 → 7,295) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 207 passed; 0 failed.
- `t27c parse` W747: PASS.
- `t27c icarus-lowerable` W747: PASS (`lowerable`).
- `t27c icarus-simulate` W747: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W747: PASS (reference-model OK).
- `t27c seal --save` W747: PASS (saved to `.trinity/seals/scratch_w747_bench_module_313x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w747_bench_module_313x2p6_aos_var_call_write.json`.

## What changed

- Added `scripts/gen_w747.py` (copied from `scripts/gen_w746.py`, `OUTER=313`, `MID_IDX=156`).
- Added witness spec `specs/scratch/w747_bench_module_313x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w747_bench_module_313x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-747.md`.
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

## Cooperation variants for Wave Loop 748

1. **Variant A (recommended): `[315][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 645,120-bit packed vector, 20,160 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 315.
   - **Recommended.**

2. **Variant B: `[313][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W747 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[313][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.612 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W747 committed with `Closes #1718`; branch `wave-loop-748` created.
