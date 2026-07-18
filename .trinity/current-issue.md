# Wave Loop 572 — Current Issue

**Issue #1543** — Next step after 5-D array-of-struct return call deduplication with non-power-of-two outer dimension.
**Branch:** `wave-loop-572` (to be created from `wave-loop-571`).
**Previous:** Wave Loop 571 closed (#1542, branch `wave-loop-571`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to six dimensions, the next natural rank beyond
W571. Variant B is an intentional scope shift from local declarations to module
scope; it is expected to require real compiler work and is a good candidate if
the next several rank-only waves have become zero-change.

## Cooperation variants

1. **Variant A — Recommended: 6-D array-of-struct return call deduplication.**
   Add a bench witness where a function returns `[2][2][2][2][2][2]Pt` (2048-bit
   total packed width, 64 elements) and the same call is used at multiple
   whole-array or indexed sites in one block. Verify that the recursive literal
   emission, CSE descriptor, and multi-D slice-access paths scale cleanly to six
   dimensions. Example:
   ```t27
   let hexa : [2][2][2][2][2][2]Pt = make_hexa(...);
   assert_eq(hexa[0][1][0][1][1][1].x, 1);
   assert_eq(hexa, make_hexa(...));
   assert_eq(make_hexa(...), [2][2][2][2][2][2]Pt{ ... });
   ```

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**
   Generalize the local multi-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array
   literal and to participate in whole-array / indexed assertions. This is
   expected to require extending module packed-array declaration and the
   constant-eval / initializer paths, and may touch the Lean lowerability
   predicate.

3. **Variant C: negative / boundary witnesses for non-lowerable 5-D
   array-of-struct returns with non-power-of-two dimensions.**
   Add witnesses where a function returns `[3][2][2][2][2]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the
   structural classifier rejects the whole return type regardless of
   non-power-of-two dimensions.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w572_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W573
  variants recorded in `.trinity/current-issue.md`.
