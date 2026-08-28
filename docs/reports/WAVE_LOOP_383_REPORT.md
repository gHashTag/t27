# Wave Loop 383 Close-Out Report

**Date:** 2026-07-01
**Branch:** `trinity-rust-rings`
**Tracking issue:** #1276
**Selected variant:** Variant B (proof push to 276 `ternaryMac` generic ∀ + extend array/RAM lowering to ROM literals and function-local arrays)
**Commit:** (see `git log` on `trinity-rust-rings`)

---

## Summary

Wave Loop 383 executed the recommended **Variant B** scope: it pushed the `ternaryMac` Lean 4 generic ∀ proof lattice from 272 to **276**, extended the IGLA CODER+RACE zero-failure streak to **117 waves**, and extended the W382 array/RAM lowering in the `gen-verilog` backend to cover **ROM-style array-literal constants** and **function-local array variables**.

The new regression spec `specs/scratch/w383_rom_array.t27` exercises a module-level `const lut : [4]u16 = [4]u16{...}` ROM lookup and a function-local `var tmp : [2]u16` read/write. Generated Verilog now emits synthesizable `reg [15:0] lut [0:3];` with per-element `initial` assignments, and per-element registers for local arrays with numeric-literal index rewriting.

Full conformance reached **563/563 PASS**.

## Quantified results

| Metric | W382 | W383 | Δ |
|---|---|---|---|
| Lean 4 `ternaryMac` generic ∀ | 272 | **276** | +4 |
| Pool A floor | 126 | **127** | +1 |
| CODER minimum | 116 | **117** | +1 |
| Pool B depth (`systolic_ternary`) | 144 | **145** | +1 |
| Integration depth (`ternary_inference`) | 125 | **126** | +1 |
| Full-repo tests | 13,362 | **13,419** | +57 |
| Full-repo invariants | 5,881 | **5,908** | +27 |
| Conformance specs | 562 | **563** | +1 (scratch) |
| Conformance pass rate | 562/562 | **563/563** | 100% |
| Gen-verilog yosys smoke targets | 43 | **44** | +1 scratch spec |
| Zero-IGLA-failure streak | 116 waves | **117 waves** | +1 |

Test/invariant counts are from `t27c stats` and include all spec files (`specs/` and `compiler/`).

## New theorems in `proofs/lean4/Trinity/TernaryInference.lean`

Wave Loop 383 added **4** new `ternaryMac` generic ∀ theorems:

1. `ternaryMacAccumulateSixtyOnePlusGeneric` — 61-variable plus accumulation (**273 generic ∀ milestone**).
2. `ternaryMacAccumulateSixtyMinusGeneric` — 60-variable minus accumulation lattice.
3. `ternaryMacQuadragintupleDuoCancellationGeneric` — `mac^42(x, a, [.plus,.minus,...]) = x` (depth-42 identity cancellation).
4. `ternaryMacZeroWeightEighteenPairClosureGeneric` — 18 zero-weight MACs before and after a plus-weight MAC are transparent (**276 generic ∀ milestone**).

`lake build Trinity.TernaryInference` completed successfully.

## Gen-verilog: ROM literals and function-local arrays

### Finding

W382 added module-level `var mem : [N]T` memory lowering, but two common array patterns were still missing:

- `const lut : [N]T = [N]T{...}` at module scope was emitted as a scalar `localparam` with the array-literal syntax in the RHS, which Yosys rejected.
- `var tmp : [N]T` inside a combinational function had no lowering at all; index expressions `tmp[i]` were emitted as scalar bit-selects on a non-existent scalar register.

### Fix

Modified `bootstrap/src/compiler.rs`:

- Reused the existing `parse_array_type` helper (`bootstrap/src/compiler.rs:3859`) to detect array type annotations in both `const` and `var` declarations.
- Updated `gen_verilog_const` (`bootstrap/src/compiler.rs:4154`) so that an array-typed constant emits a synthesizable Verilog memory with an `initial` block:
  ```verilog
  // LUT: lut [4] u16
  reg [15:0] lut [0:3];
  initial begin
      lut[0] = 16'h1234;
      lut[1] = 16'h5678;
      lut[2] = 16'h9ABC;
      lut[3] = 16'hDEF0;
  end
  ```
- Updated `StmtLocal` lowering (`bootstrap/src/compiler.rs:4981`) to emit per-element registers for function-local arrays:
  ```verilog
  reg [15:0] tmp_0;
  reg [15:0] tmp_1;
  ```
- Updated `ExprIndex` lowering (`bootstrap/src/compiler.rs:5417`) so that numeric-literal index access on a function-local array identifier rewrites to the flattened reg name (`tmp_0`) instead of `tmp[0]`, which keeps the generated Verilog synthesizable inside a function context.

This is still a narrow, closed subset of array support: one-dimensional arrays with explicit `[N]T` type annotations, module-level `const`/`var` arrays, and function-local `var` arrays with numeric-literal indices. Multi-dimensional arrays, inferred RAM styles, and non-literal dynamic indices remain open.

### Regression evidence

- Added `specs/scratch/w383_rom_array.t27` exercising:
  - `const lut : [4]u16 = [4]u16{0x1234, 0x5678, 0x9ABC, 0xDEF0};`
  - `lookup(idx)` returning `lut[idx]`.
  - `var tmp : [2]u16` inside a function with write/read.
- Generated Verilog declares `reg [15:0] lut [0:3];` and `reg [15:0] tmp_0; reg [15:0] tmp_1;`.
- `yosys read_verilog -sv` + `synth -top w383_rom_array` pass with 0 problems.
- All 27 IGLA specs remain yosys-clean under the smoke gate.

## CI smoke gate

- The in-runner smoke gate now covers all 27 IGLA specs plus 17 scratch specs = **44 targets**.
- `specs/scratch/w383_rom_array.t27` is part of the scratch smoke set.

## FPGA / hardware status

- The W361 bitstream `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB) remains ready.
- Physical board flashing is still blocked by the missing Xilinx DLC10/Platform Cable USB adapter (`dlc10 idcode` fails with `VID=0x03FD`).
- See `docs/reports/FPGA_EVIDENCE_W383.md` for the latest board-flash attempt.

## Seal / conformance

- 27 core IGLA seals regenerated from `.` using `t27c seal --save`.
- Full suite result: **563/563 PASS**, zero seal mismatches, zero yosys smoke failures.

---

*phi² + 1/phi² = 3 | TRINITY*
