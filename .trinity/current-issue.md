# Wave Loop 567 — Current Issue

**Issue #1538** — Next step after 2-D array-of-struct return call deduplication.
**Branch:** `wave-loop-567` (to be created from `wave-loop-566`).
**Previous:** Wave Loop 566 closed (#1537, branch `wave-loop-566`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it extends
the W566 2-D result to 3-D, proving the rank-agnostic paths are truly generic.

## Cooperation variants

1. **Variant A — Recommended: 3-D array-of-struct return call deduplication.**
   Add a bench witness where a function returns `[N][M][K]Pt` and the same call
   is used at multiple whole-array or indexed sites in one block. Verify that the
   W563 CSE descriptor (`call_returning_cse_value_info`) and the multi-D slice
   access paths cooperate at three dimensions. Example:
   ```t27
   let cube : [2][2][2]Pt = make_cube(...);
   assert_eq(cube[0][1][0].x, 1);
   assert_eq(cube, make_cube(...));
   assert_eq(make_cube(...), [2][2][2]Pt{ ... });
   ```

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**
   Generalize the W566 local 2-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and to participate in whole-array / indexed assertions. This may require
   extending module packed-array declaration and the constant-eval / initializer
   paths.

3. **Variant C: negative / boundary witnesses for non-lowerable 2-D
   array-of-struct returns.**
   Add witnesses where a function returns `[N][M]Pt` and `Pt` contains `string`,
   `enum`, `f32`, or an unresolved-import field, proving the structural classifier
   rejects the whole return type.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w567_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W568
  variants recorded in `.trinity/current-issue.md`.
