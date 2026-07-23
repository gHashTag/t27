# FPGA Loop Closeout — Wave Loop 749

**Issue:** #1720
**Branch:** `wave-loop-749`
**Date:** 2026-07-23
**Variant:** A — module-scope `[317][2]^6 Pt` array-of-struct variable
initialized from a function call, with indexed signed field writes and read-back.

## Summary

Wave Loop 749 validated a module-scope `[317][2]^6 Pt` packed array-of-struct
variable, initialized from a function call, and exercised with indexed signed
field writes. The witness is 649,216 bits (~0.620 MiBit, 20,288 elements). No
compiler or reference-model changes were required; the witness passes all
structural, simulation, cocotb reference-model, and seal gates.

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 317 |
| Total elements | 20,288 |
| Packed vector width | 649,216 bits |
| Approximate size | ~0.620 MiBit |
| Generator script | `scripts/gen_w749.py` |
| Witness spec | `specs/scratch/w749_bench_module_317x2p6_aos_var_call_write.t27` |
| Spec lines | ~60,291 |
| Spec size | ~1,388 KB |
| Mid element | `[158][1][0][0][0][0][0]` (element 10,144) |
| Last element | `[316][1][1][1][1][1][1]` (element 20,287) |
| Last raw x | 40,574 (wraps modulo 32,768 → 7,806) |
| Last raw y | 40,575 (wraps modulo 32,768 → 7,807) |

## Validation results

- `cargo build --release -p t27c`: PASS.
- `cargo test -p t27c --bin t27c`: 1,494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 209 passed; 0 failed.
- `t27c parse` W749: PASS.
- `t27c icarus-lowerable` W749: PASS (`lowerable`).
- `t27c icarus-simulate` W749: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W749: PASS (reference-model OK).
- `t27c seal --save` W749: PASS (saved to `.trinity/seals/scratch_w749_bench_module_317x2p6_aos_var_call_write.json`).
- Empty Icarus baseline saved:
  `.trinity/icarus-baselines/specs/scratch/w749_bench_module_317x2p6_aos_var_call_write.t27.baseline`.

## What changed

- Added `scripts/gen_w749.py` (copied from `scripts/gen_w748.py`, `OUTER=317`, `MID_IDX=158`).
- Added witness spec `specs/scratch/w749_bench_module_317x2p6_aos_var_call_write.t27`.
- Added integration test `accepts_w749_bench_module_317x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal and empty Icarus baseline under `.trinity/`.
- Updated `.trinity/experience.md` and `.trinity/current-issue.md`.
- Created closeout plan `.claude/plans/wave-loop-749.md`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.

## Weak points / next ring backlog

- **L1 TRACEABILITY:** 514 commits in the last 30 days; 109 still lack `Closes #N`.
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
  - **Tlsys** — first ternary RTL-to-CNFET gate-level synthesis framework, with
    designs over 500,000 gates (*Chinese Journal of Electronics*, 2026).
    DOI: <https://doi.org/10.23919/cje.2025.00.418>.
  - **Ternary VHDL (TVHDL)** — balanced-ternary extension to IEEE 1076-2008
    VHDL with open-source library and GHDL simulation (ISMVL 2026).
    DOI: <https://doi.org/10.1109/ismvl68998.2026.00041>.
  - **SONIC / SimulationEngine** — C# ternary EDA toolchain with event-driven
    gate-level simulation, REBEL-2 balanced ternary CPU, Verilog export and
    Basys3 emitter (ISMVL 2026).
    DOI: <https://doi.org/10.1109/ismvl68998.2026.00042>.
  - **REBEL-6** — 32-trit balanced ternary ISA with RV32I-to-REBEL compiler
    pipeline for C (ISMVL 2025).
    DOI: <https://doi.org/10.1109/ismvl64713.2025.00028>.
  - **Trinity v2.0.x / B002** — zero-DSP ternary-weight autoregressive LLM
    inference on QMTech XC7A100T using OpenXC7, ~63 tok/s @ 92 MHz, ~1 W
    (Zenodo 2025/2026).
    DOIs: <https://doi.org/10.5281/zenodo.18939352>,
    <https://doi.org/10.5281/zenodo.19224235>.
  - **RTL-Based General Synthesis Methodology for Ternary Logic** — generic
    ternary RTL-to-gate-level synthesis with GT-LOGIC library, 63.39% cell-count
    reduction vs. MUX-based synthesis, demonstrated on memristor-CMOS, CNTFET,
    T-CMOS, and DEPFET (IEEE Access, 2025).
    URL: <https://sah.borca.ai/papers/281362292>.

## Cooperation variants for Wave Loop 750

1. **Variant A (recommended): `[319][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 653,312-bit packed vector, 20,416 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 319.
   - **Recommended.**

2. **Variant B: `[317][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W749 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[317][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at ~0.620 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] Witness generated and lowerable.
- [x] Simulation and cocotb reference-model gates pass.
- [x] Seal and Icarus baseline saved.
- [x] Cargo test suites pass.
- [x] Closeout report written.
- [x] Experience, current-issue, and memory updated.
- [x] W749 committed with `Closes #1720`; branch `wave-loop-750` created.
