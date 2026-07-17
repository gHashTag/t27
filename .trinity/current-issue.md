# Wave Loop 562 — Current Issue

**Issue #1533** — Next step after negative / boundary witnesses for
non-lowerable struct returns.
**Branch:** `wave-loop-562` (to be created from `wave-loop-561`).
**Previous:** Wave Loop 561 closed (#1532, branch `wave-loop-561`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
continues the W556–W560 call-deduplication series, but it requires three
prerequisite gaps (array-of-struct literal lowering, bench-local AoS
variables, and 1-D AoS element field access) to be fixed first.

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

2. **Variant B: whole-struct comparison for structs with array-typed fields.**
   Extend the W555 whole-array probe path to scalar-struct variables whose
   fields are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` and
   bench assignment cross-checks for structs such as
   `struct { xs: [4]i8, ys: [4]i8 }`.

3. **Variant C: explicit side-effect / non-deterministic bench classifier.**
   Add (or extend) an AST classifier that rejects `bench` blocks containing
   unbounded loops or other non-deterministic constructs from the
   deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556–W560
   deduplication optimization is only valid for pure calls in terminating
   blocks.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w562_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W563
  variants recorded in `.trinity/current-issue.md`.
