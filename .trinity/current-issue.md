# Wave Loop 570 — Current Issue

**Issue #1541** — Next step after 4-D array-of-struct return call deduplication with non-power-of-two outer dimension.
**Branch:** `wave-loop-570` (to be created from `wave-loop-569`).
**Previous:** Wave Loop 569 closed (#1540, branch `wave-loop-569`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to five dimensions, the next natural rank beyond
W569.

## Cooperation variants

1. **Variant A — Recommended: 5-D array-of-struct return call deduplication.**
   Add a bench witness where a function returns `[2][2][2][2][2]Pt` (or a
   non-power-of-two variant such as `[3][2][2][2][2]Pt`) and the same call is
   used at multiple whole-array or indexed sites in one block. Verify that the
   recursive literal emission, CSE descriptor, and multi-D slice-access paths
   scale cleanly to five dimensions and that the total packed width is computed
   correctly. Example:
   ```t27
   let penta : [2][2][2][2][2]Pt = make_penta(...);
   assert_eq(penta[0][1][0][1][1].x, 1);
   assert_eq(penta, make_penta(...));
   assert_eq(make_penta(...), [2][2][2][2][2]Pt{ ... });
   ```

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**
   Generalize the local multi-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array
   literal and to participate in whole-array / indexed assertions. This may
   require extending module packed-array declaration and the constant-eval /
   initializer paths, and may touch the Lean lowerability predicate.

3. **Variant C: negative / boundary witnesses for non-lowerable 4-D
   array-of-struct returns with non-power-of-two dimensions.**
   Add witnesses where a function returns `[3][2][2][2]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the
   structural classifier rejects the whole return type regardless of non-power-of-two
   dimensions.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w570_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W571
  variants recorded in `.trinity/current-issue.md`.
