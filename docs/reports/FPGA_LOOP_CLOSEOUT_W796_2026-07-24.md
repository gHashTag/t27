# Wave Loop 796 Closeout Report

**Date:** 2026-07-24
**Branch:** `wave-loop-796`
**Parent branch:** `wave-loop-795` HEAD (`58d5a870a`)
**Issue:** #1521
**PR:** #1522 (to open)
**Cooperation variant:** A (recommended)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Summary

Wave Loop 796 extended the module-scope packed array-of-struct ladder to `[411][2]^6 Pt`.
The witness contains 26,304 elements flattened to a single 841,728-bit packed
SystemVerilog vector (~0.803 MiBit), still well under the 4-MiBit simulation cliff,
and required zero compiler, reference-model, or `FROZEN_HASH` changes.

## What landed

- `specs/scratch/w796_bench_module_411x2p6_aos_var_call_write.t27`
  - 26,304 elements, 841,728-bit packed vector (~0.803 MiBit).
  - Module-scope `pub var dst : [411][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w796.py`
  - Generator for the W796 witness; `OUTER = 411`, `MID_IDX = 205`.
  - Note: both the destination path and the module header f-string had to be
    manually fixed after copying from W795 (generator copy hazard).
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w796_bench_module_411x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w796_bench_module_411x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.claude/plans/wave-loop-797.md`
  - Next-wave plan with three cooperation variants.

## Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged
  `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 256 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W796 | PASS |
| `t27c icarus-lowerable` W796 | PASS (`lowerable`) |
| `t27c icarus-simulate` W796 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W796 | PASS (reference-model OK) |
| `t27c seal --save` W796 | PASS |

## Weak-point audit (2026-07-24)

- **Open PR ladder:** PRs #1484, #1486, #1488, #1489, #1491, #1493, #1494, #1496,
  #1498, #1500, #1502, #1504, #1506, #1508, #1510, #1512, #1514, #1516, #1518, #1520
  remain open awaiting review. Continue branching from the previous wave HEAD.
- **Pre-existing `verilog_array_literal_expr` regression:** still fails on
  `r_ca_2_synthetic_no_comment_only_call_argument`; out of scope for the witness
  ladder and tracked for a separate issue.
- **FPGA E2E CI red:** `sby` missing + Yosys static-cast error in generated `uart.v`.
- **Warning debt:** 780 clippy warnings and 627 release warnings need a dedicated
  cleanup sprint.
- **Vivado-in-Docker CI gap:** private image not yet published.
- **30-day traceability by subject:** ~9.4% (120/1282?); the subject-only count is
  noisy because bulk subject lines do not carry `Closes #N`; continue putting
  `Closes #N` in commit subjects and bodies.

## Scientific / engineering background

- IEEE 1800-2017 §7.4.1/7.4.3 define packed-array width as the product of packed
  dimensions, with no power-of-two restriction. Variant A emits a single
  841,728-bit packed vector, which is legal SystemVerilog.
- Lutsig's verified array lowering and CIRCT's `HWLegalizeModules` show that
  flattening nested arrays to wide packed vectors is a well-founded compiler
  discipline, even when outer dimensions are non-power-of-two.
- Icarus issue #1134 documents assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids that construct entirely.
- Yosys issue #2677 / #4653 confirm that arrays of packed structs remain
  unsupported in the native frontend; t27's packed-vector lowering avoids the
  gap.
- Recent 2025–2026 ternary/MVL literature reinforces that flattening ternary
  aggregate data to wide binary packed vectors is a pragmatic, toolchain-compatible
  path while native MVL fabrics mature:
  - **An RTL-Based General Synthesis Methodology for Device-Independent Ternary
    Logic Circuits** (IEEE Access 2025): Verilog-based ternary syntax extension,
    GT-LOGIC library, and device-independent synthesis across memristor-CMOS,
    CNTFET, T-CMOS, and DEPFET.
  - **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based
    Gate-Level Netlist** (Chinese Journal of Electronics 2026): RTL-to-CNFET
    ternary synthesis framework scaling to 500k+ gates.
  - **Analysis, Simulation and Design of Ternary Logic Circuits Based on CNTFETs
    in Verilog-A** (Int. J. Numerical Modelling 2025): CNTFET-based ternary
    NOR/NAND/decoder gates in Verilog-A for EDA compatibility.
  - **Area and Power Optimised Ternary Comparator using Hybrid CNTFET-RRAM
    Technology** (ICOECA 2026): hybrid CNTFET+memristor ternary comparator with
    ~52% delay reduction and ~58% power reduction.

## Cooperation variants for Wave Loop 797

- **Variant A (recommended):** continue odd outer-dimension ladder with `[413][2]^6 Pt`
  (~0.807 MiBit, 26,432 elements, 845,824-bit packed vector). Zero compiler changes
  expected.
- **Variant B:** keep `[411][2]^6 Pt` width but move the packed var to bench/function
  scope to exercise function-local non-power-of-two packed arrays.
- **Variant C:** keep `[411][2]^6 Pt` width and add `if`-guarded indexed signed field
  writes to exercise control-flow + packed-vector writes.

## Artifacts

- Witness: `specs/scratch/w796_bench_module_411x2p6_aos_var_call_write.t27`
- Generator: `scripts/gen_w796.py`
- Seal: `.trinity/seals/scratch_w796_bench_module_411x2p6_aos_var_call_write.json`
- Integration test: `bootstrap/tests/icarus_lowerable.rs`
- Plan: `.claude/plans/wave-loop-797.md`
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W796_2026-07-24.md`

*φ² + φ⁻² = 3 | TRINITY*
