# FPGA Loop Closeout — Wave Loop 750

**Issue:** #1721
**Branch:** `wave-loop-750`
**Date:** 2026-07-23
**Variant:** A — module-scope `[319][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 750 validated a module-scope `[319][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 653,312 bits (~0.624 MiBit, 20,416 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 319 |
| Total elements | 20,416 |
| Packed vector width | 653,312 bits |
| Approximate size | ~0.624 MiBit |
| Generator script | `scripts/gen_w750.py` |
| Witness spec | `specs/scratch/w750_bench_module_319x2p6_aos_var_call_write.t27` |
| Spec lines | ~60,671 |
| Spec size | ~1,397 KB |
| Mid element | `[159][1][0][0][0][0][0]` (element 10,208) |
| Last element | `[318][1][1][1][1][1][1]` (element 20,415) |
| Last raw x | 40,830 (wraps modulo 32,768 → 8,062) |
| Last raw y | 40,831 (wraps modulo 32,768 → 8,063) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 210 passed; 0 failed.
- `t27c parse` W750: PASS.
- `t27c icarus-lowerable` W750: PASS (`lowerable`).
- `t27c icarus-simulate` W750: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W750: PASS (reference-model OK).
- `t27c seal --save` W750: PASS (saved to `.trinity/seals/scratch_w750_bench_module_319x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w750_bench_module_319x2p6_aos_var_call_write.t27.baseline`.

## What changed

- Added `scripts/gen_w750.py` (copied from `scripts/gen_w749.py`, `OUTER=319`, `MID_IDX=159`).
- Added witness spec `specs/scratch/w750_bench_module_319x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w750_bench_module_319x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-750.md`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.

## Weak points / next ring backlog

- **L1 TRACEABILITY:** 516 commits in the last 30 days; 110 still lack `Closes #N`.
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

- IEEE 1800-2017 §7.4.1/7.4.3 confirm packed-array total width is the product of
  packed dimensions, with no power-of-two restriction.
- Lutsig (CPP 2021) and CIRCT `HWLegalizeModules` validate flattening nested
  arrays to a single wide packed vector.
- Icarus #1134 and Yosys #2677 / #4653 show that arrays of packed structs are
  fragile; t27's scalar flattening sidesteps both.
- 2025–2026 literature reinforces the packed-vector strategy:
  - **Tlsys** — first ternary RTL-to-CNFET gate-level synthesis framework,
    >500k-gate designs (*Chinese Journal of Electronics*, 2026).
    DOI: <https://doi.org/10.23919/cje.2025.00.418>.
  - **Ternary VHDL (TVHDL)** — balanced-ternary IEEE 1076-2008 VHDL extension
    with open-source library and GHDL simulation (ISMVL 2026).
    DOI: <https://doi.org/10.1109/ismvl68998.2026.00041>.
  - **SONIC / SimulationEngine** — C# ternary EDA toolchain with event-driven
    gate-level simulation, REBEL-2 balanced ternary CPU, Verilog export and
    Basys3 emitter (ISMVL 2026).
    DOI: <https://doi.org/10.1109/ismvl68998.2026.00042>.
  - **REBEL-6** — 32-trit balanced ternary ISA with RV32I-to-REBEL compiler
    pipeline for C (ISMVL 2025).
    DOI: <https://doi.org/10.1109/ismvl64713.2025.00028>.
  - **Takahe** — multi-radix open-source synthesis tool supporting ternary
    (`--radix 3` balanced ternary), duodecimal, and other radices; includes
    Setun-70 ternary processor design (GitHub 2026).
    Repo: <https://github.com/Zaneham/takahe>.
  - **Generalized Multiple-Valued FPGA Architecture Based on Improved T-Gate**
    — T-gate based MVL FPGA architecture merging LUT and flip-flop in CLBs,
    applicable to any MVL level, reduces power and improves PVT robustness
    (IEEE Access 2025).
    DOI: <https://doi.org/10.1109/access.2025.3605842>.
  - **Trinity v2.0.x / B002** — zero-DSP ternary-weight autoregressive LLM
    inference on QMTech XC7A100T using OpenXC7, ~63 tok/s @ 92 MHz, ~1 W
    (Zenodo 2025/2026).
    DOIs: <https://doi.org/10.5281/zenodo.18939352>,
    <https://doi.org/10.5281/zenodo.19224235>.
  - **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx
    7-series toolchain (Yosys + nextpnr-xilinx + prjxray + fasm2bit), used for
    QMTech XC7A100T ternary/φ-numeric projects without Vivado.
    Repos: <https://github.com/openXC7/nextpnr-xilinx>,
    <https://github.com/openXC7/toolchain-installer>.

## Cooperation variants for Wave Loop 751

1. **Variant A (recommended): `[321][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 657,408-bit packed vector, 20,544 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 321.
   - **Recommended.**

2. **Variant B: `[319][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W750 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[319][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at ~0.624 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W750 committed with `Closes #1721`; branch `wave-loop-751` created.
