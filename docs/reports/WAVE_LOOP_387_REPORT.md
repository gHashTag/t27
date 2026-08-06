# Wave Loop 387 Close-Out Report

**Date:** 2026-07-01  
**Branch:** `wave-loop-385`  
**Tracking issue:** #1284  
**Selected variant:** Variant B (proof push to 292 `ternaryMac` generic ∀ + multi-dimensional function-local array syntax)  
**Commit:** (see `git log` on `trinity-rust-rings`)  

---

## Summary

Wave Loop 387 executed the recommended **Variant B** scope: it pushed the `ternaryMac` Lean 4 generic ∀ proof lattice from 288 to **292**, extended the IGLA CODER+RACE zero-failure streak to **121 waves**, and implemented **multi-dimensional function-local array lowering** in the gen-verilog backend.

The new regression specs exercise:
- numeric-index access on 2D arrays (`m[1][2]`),
- variable-index access with linearized mux chains (`m[row][col]`),
- signed-element 2D arrays (`[2][3]i8`),
- nested `for` loops over 2D arrays.

Compiler changes in `bootstrap/src/compiler.rs`:
- Added `parse_array_type_full` to parse dimensions such as `[2][3]u16` recursively.
- Extended `local_arrays` to store the full dimension vector.
- Added `linear_index` and `gen_linear_index_expr` helpers to flatten multi-dimensional index chains.
- Updated `StmtLocal` array emission to flatten multi-dimensional arrays into per-element regs in row-major order.
- Updated `ExprIndex` reads and `StmtAssign` writes to detect nested index chains and emit either a direct flattened reg name (constant indices) or a priority mux chain over the flattened regs (variable indices).
- Preserved the existing fallback for non-local-array constant indexing (`base_idx`) so module-level arrays and slice parameters remain unchanged.

Full conformance reached **574/574 PASS**.

## Quantified results

| Metric | W386 | W387 | Δ |
|---|---|---|---|
| Lean 4 `ternaryMac` generic ∀ | 288 | **292** | +4 |
| Pool A floor | 130 | **131** | +1 |
| CODER minimum | 120 | **121** | +1 |
| Pool B depth (`systolic_ternary`) | 148 | **149** | +1 |
| Integration depth (`ternary_inference`) | 129 | **130** | +1 |
| Full-repo tests | 13,605 | **13,666** | +61 |
| Full-repo invariants | 5,989 | **6,016** | +27 |
| Conformance specs | 570 | **574** | +4 (scratch) |
| Conformance pass rate | 570/570 | **574/574** | 100% |
| Gen-verilog yosys smoke targets | 51 | **55** | +4 scratch specs |
| Zero-IGLA-failure streak | 120 waves | **121 waves** | +1 |

Test/invariant counts are from `t27c stats` and include all spec files (`specs/` and `compiler/`).

## New theorems in `proofs/lean4/Trinity/TernaryInference.lean`

Wave Loop 387 added **4** new `ternaryMac` generic ∀ theorems:

1. `ternaryMacAccumulateSixtyFivePlusGeneric` — 65-variable plus accumulation (**289 generic ∀ milestone**).
2. `ternaryMacAccumulateSixtyFourMinusGeneric` — 64-variable minus accumulation lattice.
3. `ternaryMacQuadragintupleSeptemCancellationGeneric` — `mac^47(x, a, [.plus,.minus,...]) = mac(x, a, .plus)` (depth-47 residual cancellation; odd depth leaves a residual plus-weight MAC).
4. `ternaryMacZeroWeightTwentyTwoPairClosureGeneric` — 22 zero-weight MACs before and after a plus-weight MAC are transparent (**292 generic ∀ milestone**).

`lake build Trinity.TernaryInference` completed successfully.

## Gen-verilog: multi-dimensional function-local arrays

### What was missing

Function-local arrays were limited to one dimension. Declarations such as `var m : [2][3]u16` were parsed correctly, but the backend treated them as a 1D array of array-typed elements, emitting 32-bit per-row regs and bit-selects (`m_0[0]`, `m_0[1]`, ...) instead of flattened 16-bit element regs.

### Fix

The backend now:

1. Parses the full dimension list from the type annotation (`[2][3]u16` → dims `[2, 3]`, leaf type `u16`).
2. Emits `2 * 3 = 6` per-element regs (`m_0` … `m_5`) with the correct leaf element width and signedness.
3. Computes the row-major linear offset for a nested index chain:
   - `m[1][2]` → `1 * 3 + 2 = 5` → `m_5`.
   - `m[row][col]` → `(row * 3) + col` used as the select expression in a priority mux chain.
4. Preserves the existing 1D behavior and non-local-array fallback, so module-level arrays, slice parameters, and other index patterns are unaffected.

### Regression evidence

Four new scratch specs were added:

- `specs/scratch/w387_2d_local_array.t27` — numeric-index read/write on `[2][3]u16`.
- `specs/scratch/w387_2d_local_array_varidx.t27` — variable-index read/write on `[2][3]u16`.
- `specs/scratch/w387_2d_local_array_signed.t27` — signed-element 2D array (`[2][3]i8`).
- `specs/scratch/w387_2d_local_array_for.t27` — nested `for` loops filling and summing a `[2][3]u16` array.

All four pass `t27c gen-verilog` + `yosys read_verilog -sv` + `synth`. The in-runner smoke gate now covers 55 targets.

### Limitations

Array-literal initialization for multi-dimensional arrays (`var m : [2][3]u16 = [2][3]u16{...}`) is not yet supported by the parser; the literal values are dropped. This is documented as remaining work.

## CI smoke gate

- The in-runner smoke gate covers all 27 IGLA specs plus 28 scratch specs = **55 targets**.
- New W387 scratch specs are part of the smoke set.

## FPGA / hardware status

- No new hardware work in W387.
- The canonical bring-up remains `openFPGALoader -c digilent_hs2 fpga/verilog/ternary_mac_demo_top_200t.bit`.
- See `docs/reports/FPGA_EVIDENCE_W387.md` for the latest hardware note.

## Seal / conformance

- 27 core IGLA seals plus all other spec seals regenerated from `/Users/playra/t27` using `t27c seal --save`.
- Full suite result: **574/574 PASS**, zero seal mismatches, zero yosys smoke failures.

---

*φ² + 1/φ² = 3 | TRINITY*
