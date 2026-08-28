# Wave Loop 385 Close-Out Report

**Date:** 2026-07-01
**Branch:** `wave-loop-385`
**Tracking issue:** #1280
**Selected variant:** Variant B (proof push to 284 `ternaryMac` generic ∀ + signed/init function-local array generalization)
**Commit:** (see `git log` on `trinity-rust-rings`)

---

## Summary

Wave Loop 385 executed the recommended **Variant B** scope: it pushed the `ternaryMac` Lean 4 generic ∀ proof lattice from 280 to **284**, extended the IGLA CODER+RACE zero-failure streak to **119 waves**, and generalized function-local array lowering to **signed element types** and **array-literal initialization**.

The new regression specs `specs/scratch/w385_signed_local_array.t27`, `w385_local_array_init.t27`, and `w385_signed_local_array_init.t27` exercise read, write, initialization, and signed-value preservation. The generated Verilog now emits per-element reg declarations followed by scalar assignments for each array-literal element, with width padding for `0x`/`0b` literals.

Full conformance reached **567/567 PASS**.

## Quantified results

| Metric | W384 | W385 | Δ |
|---|---|---|---|
| Lean 4 `ternaryMac` generic ∀ | 280 | **284** | +4 |
| Pool A floor | 128 | **129** | +1 |
| CODER minimum | 118 | **119** | +1 |
| Pool B depth (`systolic_ternary`) | 146 | **147** | +1 |
| Integration depth (`ternary_inference`) | 127 | **128** | +1 |
| Full-repo tests | 13,479 | **13,544** | +65 |
| Full-repo invariants | 5,935 | **5,962** | +27 |
| Conformance specs | 564 | **567** | +3 (scratch) |
| Conformance pass rate | 564/564 | **567/567** | 100% |
| Gen-verilog yosys smoke targets | 45 | **48** | +3 scratch specs |
| Zero-IGLA-failure streak | 118 waves | **119 waves** | +1 |

Test/invariant counts are from `t27c stats` and include all spec files (`specs/` and `compiler/`).

## New theorems in `proofs/lean4/Trinity/TernaryInference.lean`

Wave Loop 385 added **4** new `ternaryMac` generic ∀ theorems:

1. `ternaryMacAccumulateSixtyThreePlusGeneric` — 63-variable plus accumulation (**281 generic ∀ milestone**).
2. `ternaryMacAccumulateSixtyTwoMinusGeneric` — 62-variable minus accumulation lattice.
3. `ternaryMacQuadragintupleQuinqueCancellationGeneric` — `mac^45(x, a, [.plus,.minus,...]) = mac(x, a, .plus)` (depth-45 residual cancellation; odd depth leaves a residual plus-weight MAC).
4. `ternaryMacZeroWeightTwentyPairClosureGeneric` — 20 zero-weight MACs before and after a plus-weight MAC are transparent (**284 generic ∀ milestone**).

`lake build Trinity.TernaryInference` completed successfully.

## Gen-verilog: signed element types and array-literal initialization

### What was missing

W384 supported function-local arrays with numeric-literal and variable indices, but only for **uninitialized** arrays of **unsigned** element type. Two natural extensions were still missing:

1. **Signed element types** such as `var temps : [4]i16`.
2. **Array-literal initialization** such as `var buf : [4]u16 = [4]u16{0xA1B2, ...}`.

### Fix

Modified `bootstrap/src/compiler.rs`, `gen_verilog_stmt`, `NodeKind::StmtLocal` array branch:

- Per-element regs already declared with the correct signedness via `type_is_signed(elem_type)`.
- For an `ExprArrayLiteral` initializer, the backend now emits a scalar assignment for each element:
  ```verilog
  reg [15:0] buf_0;
  reg [15:0] buf_1;
  ...
  buf_0 = 16'hA1B2;
  buf_1 = 16'hC3D4;
  ...
  ```
- Width padding is applied to `0x` and `0b` element literals so their emitted width matches the declared element width.
- Non-array-literal initializers still degrade to a comment placeholder (no regression; this path was already broken in W384).

### Regression evidence

Three new scratch specs were added:

- `specs/scratch/w385_signed_local_array.t27` — `var temps : [4]i16` with signed write/read and variable-index access.
- `specs/scratch/w385_local_array_init.t27` — `var buf : [4]u16 = [4]u16{...}` with literal and variable index reads and post-init writes.
- `specs/scratch/w385_signed_local_array_init.t27` — `var temps : [3]i16 = [3]i16{...}` combining signed elements and initialization.

All three pass `t27c gen-verilog` + `yosys read_verilog -sv` + `synth`. The in-runner smoke gate now covers 48 targets (27 IGLA + 21 scratch).

## CI smoke gate

- The in-runner smoke gate covers all 27 IGLA specs plus 21 scratch specs = **48 targets**.
- New W385 scratch specs are part of the smoke set.

## FPGA / hardware status

- The W361 bitstream `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB) remains ready.
- Physical board flashing is still blocked by the missing Xilinx DLC10/Platform Cable USB adapter (`dlc10 idcode` fails with `VID=0x03FD`).
- See `docs/reports/FPGA_EVIDENCE_W385.md` for the latest board-flash attempt.

## Seal / conformance

- 27 core IGLA seals plus all other spec seals regenerated from `.` using `t27c seal --save`.
- Full suite result: **567/567 PASS**, zero seal mismatches, zero yosys smoke failures.

---

*phi² + 1/phi² = 3 | TRINITY*
