# Wave Loop 573 — Current Issue

**Issue #1544** — Next step after 6-D array-of-struct return call deduplication.
**Branch:** `wave-loop-573` (to be created from `wave-loop-572`).
**Previous:** Wave Loop 572 closed (#1543, branch `wave-loop-572`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to seven dimensions, the next natural rank beyond
W572, and will reveal whether the Icarus Verilog toolchain has a practical limit
on very wide nested concatenations. Variant B is a tighter stress test at the
same rank with a non-power-of-two outer dimension. Variant C is an intentional
scope shift from local declarations to module scope; it is expected to require
real compiler work and is a good candidate if the rank-only waves have become
zero-change.

## Cooperation variants

1. **Variant A — Recommended: 7-D array-of-struct return call deduplication.**
   Add a bench witness where a function returns `[2][2][2][2][2][2][2]Pt`
   (4096-bit total packed width, 128 elements) and the same call is used at
   multiple whole-array or indexed sites in one block. Verify that the recursive
   literal emission, CSE descriptor, and multi-D slice-access paths scale cleanly
   to seven dimensions. Example:
   ```t27
   let septa : [2][2][2][2][2][2][2]Pt = make_septa(...);
   assert_eq(septa[0][1][0][1][1][1][1].x, 1);
   assert_eq(septa, make_septa(...));
   assert_eq(make_septa(...), [2][2][2][2][2][2][2]Pt{ ... });
   ```

2. **Variant B: 6-D array-of-struct return with a non-power-of-two outer
   dimension.**
   Add a bench witness where a function returns `[3][2][2][2][2][2]Pt` (3072-bit
   total packed width, 96 elements). The non-p2 outer extent is the strongest
   stress test for product-based width/index arithmetic at rank 6, following the
   W569/W571 pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**
   Generalize the local multi-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and to participate in whole-array / indexed assertions. This is expected to
   require extending module packed-array declaration and the constant-eval /
   initializer paths, and may touch the Lean lowerability predicate.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w573_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W574
  variants recorded in `.trinity/current-issue.md`.
