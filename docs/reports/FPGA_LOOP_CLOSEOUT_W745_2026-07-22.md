# FPGA Loop Closeout — Wave Loop 745

**Issue:** #1716
**Branch:** `wave-loop-745`
**Date:** 2026-07-22
**Variant:** A — module-scope `[309][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 745 validated a module-scope `[309][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 632,832 bits (~0.604 MiBit, 19,776 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 309 |
| Total elements | 19,776 |
| Packed vector width | 632,832 bits |
| Approximate size | ~0.604 MiBit |
| Generator script | `scripts/gen_w745.py` |
| Witness spec | `specs/scratch/w745_bench_module_309x2p6_aos_var_call_write.t27` |
| Spec lines | ~58,771 |
| Spec size | ~1,353 KB |
| Mid element | `[154][1][0][0][0][0][0]` (element 9,888) |
| Last element | `[308][1][1][1][1][1][1]` (element 19,775) |
| Last raw x | 39,550 (wraps modulo 32,768 → 6,782) |
| Last raw y | 39,551 (wraps modulo 32,768 → 6,783) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 205 passed; 0 failed.
- `t27c parse` W745: PASS.
- `t27c icarus-lowerable` W745: PASS (`lowerable`).
- `t27c icarus-simulate` W745: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W745: PASS (reference-model OK).
- `t27c seal --save` W745: PASS (saved to `.trinity/seals/scratch_w745_bench_module_309x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w745_bench_module_309x2p6_aos_var_call_write.json`.

## What changed

- Added `scripts/gen_w745.py` (copied from `scripts/gen_w744.py`, `OUTER=309`, `MID_IDX=154`).
- Added witness spec `specs/scratch/w745_bench_module_309x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w745_bench_module_309x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-745.md`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.

## Weak points / next ring backlog

- **L1 TRACEABILITY:** ~28 recent commits still lack `Closes #N`. Wave-loop
  closeout commits should always include issue references.
- **L4 TESTABILITY:** 57 `.t27` specs still lack `test`/`invariant`/`bench` blocks.
- **L7 UNITY:** 19 `scripts/*.sh` wrappers remain on the critical path beyond the
  two permitted exceptions, plus untracked `.sh` hooks under `.agents/` and
  `.codex/hooks/`.
- **FPGA SSOT:** `CLAUDE.md` and `cli/dlc10` contradict `fpga/HARDWARE_SSOT.md`
  on target board, IDCODE, and canonical loader.
- **L6 CEILING:** `FORMAT-SPEC-001.json` `GF256` `bias_caveat` remains open.
- **Simulator stress wave:** Plan a Wave Loop near the ~4-MiBit packed-vector
  boundary to measure Icarus/Yosys wall-clock limits.

## Scientific / engineering background

- IEEE 1800-2017 §7.4.1/7.4.3 confirm packed-array total width is the product of
  packed dimensions, with no power-of-two restriction.
- Lutsig (CPP 2021) and CIRCT `HWLegalizeModules` validate flattening nested
  arrays to a single wide packed vector.
- Icarus #1134 and Yosys #2677 / #4653 show that arrays of packed structs are
  fragile; t27's scalar flattening sidesteps both.
- 2025–2026 literature (5500FP/GargantuRAM, IEEE Access ternary synthesis,
  Tlsys, TVHDL, Takahe, KULeuven ternary-lut-dse, parameterized GSTE/STE)
  supports continuing the packed-vector approach for ternary and MVL backends.

## Cooperation variants for Wave Loop 746

1. **Variant A (recommended): `[311][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 636,928-bit packed vector, 19,904 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 311.
   - **Recommended.**

2. **Variant B: `[309][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W745 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[309][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.604 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W745 committed with `Closes #1716`; branch `wave-loop-746` created.

