# Wave Loop 571 — Current Issue

**Issue #1542** — Next step after 5-D array-of-struct return call deduplication.
**Branch:** `wave-loop-571` (to be created from `wave-loop-570`).
**Previous:** Wave Loop 570 closed (#1541, branch `wave-loop-570`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it keeps
the 5-D rank-agnostic machinery under a non-power-of-two outer dimension, which is
a stronger arithmetic stress test than the W570 power-of-two 5-D case.

## Cooperation variants

1. **Variant A — Recommended: 5-D array-of-struct return call deduplication with
   non-power-of-two outer dimension.**
   Add a bench witness where a function returns `[3][2][2][2][2]Pt` (1536-bit total
   packed width) and the same call is used at multiple whole-array or indexed
   sites in one block. Verify that the W563 CSE descriptor
   (`call_returning_cse_value_info`) and the multi-D slice access paths compute
   the correct linear offsets and width for dimension products that are not pure
   powers of two. Example:
   ```t27
   let penta : [3][2][2][2][2]Pt = make_penta(...);
   assert_eq(penta[0][1][0][1][1].x, 1);
   assert_eq(penta, make_penta(...));
   assert_eq(make_penta(...), [3][2][2][2][2]Pt{ ... });
   ```

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**
   Generalize the local multi-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array
   literal and to participate in whole-array / indexed assertions. This may
   require extending module packed-array declaration and the constant-eval /
   initializer paths, and may touch the Lean lowerability predicate.

3. **Variant C: negative / boundary witnesses for non-lowerable 5-D
   array-of-struct returns.**
   Add witnesses where a function returns `[N][M][K][L][P]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the
   structural classifier rejects the whole return type regardless of rank.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w571_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W572
  variants recorded in `.trinity/current-issue.md`.
