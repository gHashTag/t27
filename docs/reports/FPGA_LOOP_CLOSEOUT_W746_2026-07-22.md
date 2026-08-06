# FPGA Loop Closeout — Wave Loop 746

**Issue:** #1717
**Branch:** `wave-loop-746`
**Date:** 2026-07-22
**Variant:** A — module-scope `[311][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 746 validated a module-scope `[311][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 636,928 bits (~0.608 MiBit, 19,904 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 311 |
| Total elements | 19,904 |
| Packed vector width | 636,928 bits |
| Approximate size | ~0.608 MiBit |
| Generator script | `scripts/gen_w746.py` |
| Witness spec | `specs/scratch/w746_bench_module_311x2p6_aos_var_call_write.t27` |
| Spec lines | ~59,151 |
| Spec size | ~1,362 KB |
| Mid element | `[155][1][0][0][0][0][0]` (element 9,952) |
| Last element | `[310][1][1][1][1][1][1]` (element 19,903) |
| Last raw x | 39,806 (wraps modulo 32,768 → 7,038) |
| Last raw y | 39,807 (wraps modulo 32,768 → 7,039) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 206 passed; 0 failed.
- `t27c parse` W746: PASS.
- `t27c icarus-lowerable` W746: PASS (`lowerable`).
- `t27c icarus-simulate` W746: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W746: PASS (reference-model OK).
- `t27c seal --save` W746: PASS (saved to `.trinity/seals/scratch_w746_bench_module_311x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w746_bench_module_311x2p6_aos_var_call_write.json`.

## What changed

- Added `scripts/gen_w746.py` (copied from `scripts/gen_w745.py`, `OUTER=311`, `MID_IDX=155`).
- Added witness spec `specs/scratch/w746_bench_module_311x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w746_bench_module_311x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-746.md`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.

## Weak points / next ring backlog

- **L1 TRACEABILITY:** 422 commits in the last 30 days still lack `Closes #N`.
  Wave-loop closeout commits now include issue references, but the historical
  backlog remains.
- **L4 TESTABILITY:** 57 `.t27` specs still lack `test`/`invariant`/`bench` blocks.
- **L7 UNITY:** 19 `scripts/*.sh` wrappers remain on the critical path beyond the
  two permitted exceptions, plus untracked `.sh` hooks under `.agents/` and
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
  - **Park et al., IEEE Access 2025** — device-independent ternary Verilog
    synthesis with GT-LOGIC library and 63.39% cell-count reduction.
  - **TVHDL (ISMVL 2026)** — balanced ternary VHDL extension with GHDL simulation.
  - **SONIC / SimulationEngine** — ternary EDA toolchain with Verilog FPGA export,
    REBEL-2 CPU, Basys3 emitter.

## Cooperation variants for Wave Loop 747

1. **Variant A (recommended): `[313][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 641,024-bit packed vector, 20,032 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 313.
   - **Recommended.**

2. **Variant B: `[311][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W746 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[311][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.608 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W746 committed with `Closes #1717`; branch `wave-loop-747` created.
