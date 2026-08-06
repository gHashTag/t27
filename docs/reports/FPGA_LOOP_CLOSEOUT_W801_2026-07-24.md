# Wave Loop 801 Closeout Report

**Date:** 2026-07-24
**Branch:** `wave-loop-801`
**Parent branch:** `wave-loop-800` HEAD (`c1f58760e`)
**Issue:** #1531
**PR:** TBD
**Cooperation variant:** A (recommended)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Summary

Wave Loop 801 extended the module-scope packed array-of-struct ladder to `[421][2]^6 Pt`.
The witness contains 26,944 elements flattened to a single 862,208-bit packed
SystemVerilog vector (~0.822 MiBit), still well under the 4-MiBit simulation cliff,
and required zero compiler, reference-model, or `FROZEN_HASH` changes.

## What landed

- `specs/scratch/w801_bench_module_421x2p6_aos_var_call_write.t27`
  - 26,944 elements, 862,208-bit packed vector (~0.822 MiBit).
  - Module-scope `pub var dst : [421][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w801.py`
  - Generator for the W801 witness; `OUTER = 421`, `MID_IDX = 210`.
  - Both the destination path and the module header f-string were manually fixed
    after copying from W800 (generator copy hazard).
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w801_bench_module_421x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w801_bench_module_421x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.claude/plans/wave-loop-802.md`
  - Next-wave plan with three cooperation variants.
- `.claude/skills/t27-wave-loop.md`
  - Updated live tracker to wave 802.

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
| `cargo test -p t27c --test icarus_lowerable` | 261 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W801 | PASS |
| `t27c icarus-lowerable` W801 | PASS (`lowerable`) |
| `t27c icarus-simulate` W801 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W801 | PASS (reference-model OK) |
| `t27c seal --save` W801 | PASS |

## Weak-point audit (2026-07-24)

- **Open PR ladder:** earlier wave PRs remain open awaiting review. Continue
  branching from the previous wave HEAD.
- **Pre-existing `verilog_array_literal_expr` regression:** still fails on
  `r_ca_2_synthetic_no_comment_only_call_argument`; out of scope for the witness
  ladder and tracked for a separate issue.
- **FPGA E2E CI red:** `sby` missing + Yosys static-cast error in generated `uart.v`.
- **Warning debt:** 780 clippy warnings and 626 release warnings need a dedicated
  cleanup sprint.
- **Vivado-in-Docker CI gap:** private image not yet published.
- **30-day traceability by subject:** remains low; continue putting `Closes #N` in
  commit subjects and bodies.
- **Generator copy hazard:** remains the only manual failure mode in the mechanical
  wave flow. Parameterizing the wave prefix and `OUTER` dimension in the generator
  template would eliminate both stale-reference locations (destination path and
  module header f-string).

## Scientific / engineering background

- IEEE 1800-2017 §7.4.1/7.4.3 define packed-array width as the product of packed
  dimensions, with no power-of-two restriction. Variant A emits a single
  862,208-bit packed vector, which is legal SystemVerilog.
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
  - **A Generalized Multiple-Valued FPGA Architecture Based on Improved T-Gate
    Circuit** (IEEE Access 2025): T-gate based FPGA architecture merging LUT and
    flip-flop functionality in configurable logic blocks, applicable to any MVL level.
  - **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI
    Circuits** (2026 IEEE ISMVL): TVHDL, a balanced ternary extension to IEEE
    1076-2008 with open-source libraries for behavioral, RTL, and structural
    ternary modeling.
  - **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** (arXiv 2025):
    FPGA accelerator for ternary-quantized {-1, 0, +1} LLMs with a custom LUT-based
    TMat core and 1.6-bit weight compression.
  - **Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference** (Zenodo 2026):
    defensive publication showing LUT-only ternary MAC units, CORDIC functions, and
    an OpenXC7/Yosys synthesis flow with zero DSP block usage.
  - **Vericert v2.0.0** (GitHub, 2026-01-29): a formally verified HLS tool based on
    CompCert; its hyperblock scheduler uses a validated three-valued logic checker
    for predicate equivalence, showing MVL reasoning remains relevant inside
    verified compilers.

## Cooperation variants for Wave Loop 802

- **Variant A (recommended):** continue odd outer-dimension ladder with `[423][2]^6 Pt`
  (~0.826 MiBit, 27,072 elements, 866,304-bit packed vector). Zero compiler changes
  expected.
- **Variant B:** keep `[421][2]^6 Pt` width but move the packed var to bench/function
  scope to exercise function-local non-power-of-two packed arrays.
- **Variant C:** keep `[421][2]^6 Pt` width and add `if`-guarded indexed signed field
  writes to exercise control-flow + packed-vector writes.

## Artifacts

- Witness: `specs/scratch/w801_bench_module_421x2p6_aos_var_call_write.t27`
- Generator: `scripts/gen_w801.py`
- Seal: `.trinity/seals/scratch_w801_bench_module_421x2p6_aos_var_call_write.json`
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W801_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-802.md`

---

φ² + 1/φ² = 3 | TRINITY
