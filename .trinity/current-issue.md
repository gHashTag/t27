# Wave Loop 563 — Current Issue

**Issue #1534** — Next step after whole-struct comparison for structs with
array-typed fields.
**Branch:** `wave-loop-563` (to be created from `wave-loop-562`).
**Previous:** Wave Loop 562 closed (#1533, branch `wave-loop-562`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
continues the W556–W560 call-deduplication series; it still requires the three
prerequisite gaps identified in W561 (array-of-struct literal lowering,
bench-local 1-D AoS variables, 1-D AoS element field access). Variant B is a
smaller generalization of W562 to multi-dimensional array fields. Variant C is a
defensive boundary lock for non-lowerable scalar-array fields.

## Cooperation variants

1. **Variant A — Recommended: array-of-struct return call deduplication.**
   Extend the W556–W558 / W560 block-scoped call temporary machinery to
   function calls that return fixed-size arrays of lowerable packed scalar
   structs (`[N]Pt`). Prerequisite fixes:
   - `ExprArrayLiteral` lowering for `[N]Pt` literals,
   - bench-local variable declarations for one-dimensional arrays of scalar
     structs,
   - 1-D array-of-struct element field access (`arr()[i].x`).
   Once those work, add the CSE descriptor in `call_returning_cse_value_info`.

2. **Variant B: whole-struct comparison for structs with multi-dimensional
   array-typed fields.**
   Generalize W562 to scalar struct fields that are 2-D fixed-size scalar
   arrays, e.g. `struct Tile { m: [2][3]i8, tag: u8 }`. Add a bench witness
   with whole-struct `assert_eq` and element access on the call return.

3. **Variant C: negative / boundary witnesses for non-lowerable scalar-array
   fields.**
   Add witnesses where a scalar struct field is an array of `f32`, `string`,
   `enum`, or unresolved-import type. Prove the structural classifier rejects
   the whole struct, so the W560/W562 optimization cannot fire on a call
   returning such a struct.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w563_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W564
  variants recorded in `.trinity/current-issue.md`.
