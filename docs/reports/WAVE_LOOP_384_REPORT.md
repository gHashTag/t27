# Wave Loop 384 Close-Out Report

**Date:** 2026-07-01
**Branch:** `trinity-rust-rings`
**Tracking issue:** #1278
**Selected variant:** Variant B (proof push to 280 `ternaryMac` generic ∀ + variable-index local array lowering)
**Commit:** (see `git log` on `trinity-rust-rings`)

---

## Summary

Wave Loop 384 executed the recommended **Variant B** scope: it pushed the `ternaryMac` Lean 4 generic ∀ proof lattice from 276 to **280**, extended the IGLA CODER+RACE zero-failure streak to **118 waves**, and closed the remaining function-local array indexing gap by supporting **non-literal (variable) indices** inside combinational functions.

The new regression spec `specs/scratch/w384_variable_index.t27` exercises read and write with a variable index on a function-local array. The generated Verilog now emits a priority mux chain for reads and an if-else chain for writes, keeping the output synthesizable through `yosys read_verilog -sv`.

Full conformance reached **564/564 PASS**.

## Quantified results

| Metric | W383 | W384 | Δ |
|---|---|---|---|
| Lean 4 `ternaryMac` generic ∀ | 276 | **280** | +4 |
| Pool A floor | 127 | **128** | +1 |
| CODER minimum | 117 | **118** | +1 |
| Pool B depth (`systolic_ternary`) | 145 | **146** | +1 |
| Integration depth (`ternary_inference`) | 126 | **127** | +1 |
| Full-repo tests | 13,419 | **13,479** | +60 |
| Full-repo invariants | 5,908 | **5,935** | +27 |
| Conformance specs | 563 | **564** | +1 (scratch) |
| Conformance pass rate | 563/563 | **564/564** | 100% |
| Gen-verilog yosys smoke targets | 44 | **45** | +1 scratch spec |
| Zero-IGLA-failure streak | 117 waves | **118 waves** | +1 |

Test/invariant counts are from `t27c stats` and include all spec files (`specs/` and `compiler/`).

## New theorems in `proofs/lean4/Trinity/TernaryInference.lean`

Wave Loop 384 added **4** new `ternaryMac` generic ∀ theorems:

1. `ternaryMacAccumulateSixtyTwoPlusGeneric` — 62-variable plus accumulation (**277 generic ∀ milestone**).
2. `ternaryMacAccumulateSixtyOneMinusGeneric` — 61-variable minus accumulation lattice.
3. `ternaryMacQuadragintupleQuattuorCancellationGeneric` — `mac^44(x, a, [.plus,.minus,...]) = x` (depth-44 identity cancellation).
4. `ternaryMacZeroWeightNineteenPairClosureGeneric` — 19 zero-weight MACs before and after a plus-weight MAC are transparent (**280 generic ∀ milestone**).

`lake build Trinity.TernaryInference` completed successfully.

## Gen-verilog: variable-index local arrays

### Finding

W383 added function-local arrays with **numeric-literal index access** (`buf[0]`, `buf[1]`). Access with a variable index (`buf[idx]` where `idx` is a parameter or local scalar) still emitted `buf[idx]`, which is not valid for the per-element register lowering used inside Verilog functions.

### Fix

Modified `bootstrap/src/compiler.rs`:

- Added `local_arrays: HashMap<String, (usize, u32, bool)>` to `VerilogCodegen` to track arrays declared inside the current function (`bootstrap/src/compiler.rs:3702`).
- `gen_verilog_fn` now clears `local_arrays` at the start and end of each function.
- `StmtLocal` registers each function-local array and emits per-element regs using the **full flattened token escaped as a unit** (e.g. `\buf_0 `) to avoid the token-splitting issue that occurred when appending `_i` to an already-escaped keyword identifier.
- `ExprIndex` now detects three cases for local arrays:
  1. Numeric-literal index → flattened reg name (`buf_0`).
  2. Variable index on a local array → priority mux chain:
     ```verilog
     ((idx == 0) ? buf_0 : ((idx == 1) ? buf_1 : ((idx == 2) ? buf_2 : 16'd0)))
     ```
  3. Non-local array → legacy `base[idx]` behavior.
- `StmtAssign` now detects variable-index writes on local arrays and emits an if-else chain:
  ```verilog
  if (idx == 0) begin buf_0 = val; end
  else if (idx == 1) begin buf_1 = val; end
  ...
  ```

This completes the function-local array story for one-dimensional arrays with explicit `[N]T` type annotations and variable or numeric-literal indices.

### Regression evidence

- Added `specs/scratch/w384_variable_index.t27` exercising:
  - `var buf : [4]u16` inside functions.
  - Variable-index read: `return buf[idx]`.
  - Variable-index write: `buf[idx] = val`.
  - Six `test` blocks covering read/write at indices 0, 1, 2, and 3.
- Generated Verilog declares `reg [15:0] buf_0; ... reg [15:0] buf_3;`.
- `yosys read_verilog -sv` + `synth -top w384_variable_index` pass with 0 problems.
- All 27 IGLA specs remain yosys-clean under the smoke gate.

## CI smoke gate

- The in-runner smoke gate now covers all 27 IGLA specs plus 18 scratch specs = **45 targets**.
- `specs/scratch/w384_variable_index.t27` is part of the scratch smoke set.

## FPGA / hardware status

- The W361 bitstream `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB) remains ready.
- Physical board flashing is still blocked by the missing Xilinx DLC10/Platform Cable USB adapter (`dlc10 idcode` fails with `VID=0x03FD`).
- See `docs/reports/FPGA_EVIDENCE_W384.md` for the latest board-flash attempt.

## Seal / conformance

- 27 core IGLA seals plus affected non-IGLA seals regenerated from `/Users/playra/t27` using `t27c seal --save`.
- Full suite result: **564/564 PASS**, zero seal mismatches, zero yosys smoke failures.

---

*phi² + 1/phi² = 3 | TRINITY*
