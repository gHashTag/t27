# FPGA Loop Closeout - Wave Loop 753

**Issue:** #1724
**Branch:** `wave-loop-753`
**Date:** 2026-07-23
**Variant:** A - module-scope `[325][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 753 validated a module-scope `[325][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 665,600 bits (~0.635 MiBit, 20,800 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 325 |
| Total elements | 20,800 |
| Packed vector width | 665,600 bits |
| Approximate size | ~0.635 MiBit |
| Generator script | `scripts/gen_w753.py` |
| Witness spec | `specs/scratch/w753_bench_module_325x2p6_aos_var_call_write.t27` |
| Spec lines | ~61,811 |
| Spec size | ~1,389 KB |
| Mid element | `[162][1][0][0][0][0][0]` (element 10,400) |
| Last element | `[324][1][1][1][1][1][1]` (element 20,799) |
| Last raw x | 41,598 (wraps modulo 32,768 -> 8,830) |
| Last raw y | 41,599 (wraps modulo 32,768 -> 8,831) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 213 passed; 0 failed.
- `t27c parse` W753: PASS.
- `t27c icarus-lowerable` W753: PASS (`lowerable`).
- `t27c icarus-simulate` W753: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W753: PASS (reference-model OK).
- `t27c seal --save` W753: PASS (saved to `.trinity/seals/scratch_w753_bench_module_325x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w753_bench_module_325x2p6_aos_var_call_write.t27.baseline`.

## What changed

- Added `scripts/gen_w753.py` (copied from `scripts/gen_w752.py`, `OUTER=325`, `MID_IDX=162`).
- Added witness spec `specs/scratch/w753_bench_module_325x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w753_bench_module_325x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-753.md`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.

## Weak points / next ring backlog

- **L1 TRACEABILITY:** 524 commits in the last 30 days; 113 still lack `Closes #N`.
  Wave-loop closeout commits now include issue references, but the historical
  backlog remains.
- **L4 TESTABILITY:** 852 `.t27` specs exist; 445 still lack `test`/`invariant`/`bench` blocks.
- **L7 UNITY:** 19 `scripts/*.sh` wrappers remain on the critical path beyond the
  two permitted exceptions, plus 9 untracked `.sh` hooks under `.agents/` and
  `.codex/hooks/`.
- **FPGA SSOT:** `fpga/HARDWARE_SSOT.md` is canonical, but `CLAUDE.md` and stale
  FPGA docs still need alignment with the QMTech Wukong V1 / `cli/dlc10` view.
- **L6 CEILING:** `FORMAT-SPEC-001.json` `GF256.bias_caveat` remains `UNRECONCILED` / `OPEN`.
- **Simulator stress wave:** Plan a Wave Loop near the ~4-MiBit packed-vector
  boundary to measure Icarus/Yosys wall-clock limits.

## Scientific / engineering background

- IEEE 1800-2017 SS7.4.1/7.4.3 confirm packed-array total width is the product of
  packed dimensions, with no power-of-two restriction.
- Lutsig (CPP 2021) and CIRCT `HWLegalizeModules` validate flattening nested
  arrays to a single wide packed vector.
- Icarus #1134 and Yosys #2677 / #4653 show that arrays of packed structs are
  fragile; t27's scalar flattening sidesteps both.
- 2025-2026 literature reinforces the packed-vector strategy:
  - **Takahe** - multi-radix open-source synthesis tool supporting ternary
    (`--radix 3` balanced ternary), duodecimal, and other radices; includes
    Setun-70 ternary processor design (GitHub 2026).
    Repo: <https://github.com/Zaneham/Takahe>.
  - **SONIC / SimulationEngine** - C# EDA toolchain for ternary/mixed-radix VLSI,
    event-driven gate-level simulator with delta cycles, REBEL-2 CPU, Verilog
    export and Basys3 emitter (ISMVL 2026).
    DOI: <https://doi.org/10.1109/ismvl68998.2026.00042>.
  - **REBEL-6** - 32-trit balanced ternary ISA with RV32I-to-REBEL compiler
    pipeline for C (ISMVL 2025).
    DOI: <https://doi.org/10.1109/ismvl64713.2025.00028>.
  - **CNTFET-RRAM ternary comparator** - hybrid CNTFET-RRAM ternary comparator
    with ~52% lower delay, ~58% lower power, ~70% better PDP and ~45% fewer
    transistors vs CMOS ternary comparator (ICOECA 2026).
    DOI: <https://doi.org/10.1109/icoeca68095.2026.11485544>.
  - **In-memory balanced ternary memristor logic** - balanced ternary gates and
    decoders using tri-valued memristor resistance states (*European Physical
    Journal Plus*, 2026).
    DOI: <https://doi.org/10.1140/epjp/s13360-026-07895-z>.
  - **CNFET ternary logic cells for low-power VLSI** - STI/PTI/NTI inverters using
    CNFET and MOSFET at 45 nm, reporting 20-30% lower PDP for CNFET variants
    (VLSID 2025).
    DOI: <https://doi.org/10.1109/vlsid64188.2025.00075>.
  - **Trinity v2.0.x / B002** - zero-DSP ternary-weight autoregressive LLM inference
    on QMTech XC7A100T using OpenXC7, ~63 tok/s @ 92 MHz, ~1 W
    (Zenodo 2025/2026).
    DOIs: <https://doi.org/10.5281/zenodo.18939352>,
    <https://doi.org/10.5281/zenodo.19224235>.
  - **OpenXC7 / nextpnr-xilinx / Project X-Ray** - fully open-source Xilinx
    7-series toolchain, used for QMTech XC7A100T ternary projects without Vivado.
    Repos: <https://github.com/openXC7/nextpnr-xilinx>,
    <https://github.com/openXC7/toolchain-installer>.

## Cooperation variants for Wave Loop 754

1. **Variant A (recommended): `[327][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 669,696-bit packed vector, 20,928 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 327.
   - **Recommended.**

2. **Variant B: `[325][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W753 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[325][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at ~0.635 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W753 committed with `Closes #1724`; branch `wave-loop-754` created.
