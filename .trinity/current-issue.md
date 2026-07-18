# Wave Loop 580 — Current Issue

**Issue #1551** — Next step after 13-D array-of-struct return call deduplication.
**Branch:** `wave-loop-580` (to be created from `wave-loop-579`).
**Previous:** Wave Loop 579 closed (#1550, branch `wave-loop-579`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to fourteen dimensions and tests whether Icarus
Verilog can digest a 524,288-bit nested concatenation — eight times the IEEE
1800-2017 minimum packed-vector width. Variant B keeps the same rank but
introduces a non-power-of-two outer dimension, a strong stress test for
product-based width/index arithmetic at rank 13. Variant C is an intentional
scope shift from local declarations to module scope; it is expected to require
real compiler work and is the natural next step once the rank-only waves have
become zero-change.

## Cooperation variants

1. **Variant A — Recommended: 14-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns
   `[2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (524,288-bit total packed width,
   16,384 elements) and the same call is used at multiple whole-array or indexed
   sites in one block. Verify that the recursive literal emission, CSE
   descriptor, and multi-D slice-access paths scale cleanly to fourteen dimensions.
   Follow the W573–W579 witness structure (bind the literal to a local variable
   before asserting equality to avoid the Icarus `$display` overflow). Example:
   ```t27
   let tetradeca : [2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt = make_tetradeca(...);
   assert_eq(tetradeca[0][1][0][1][1][1][1][1][1][1][1][1][1][1].x, 1);
   assert_eq(tetradeca, make_tetradeca(...));
   let expected : [2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt = [2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt{ ... };
   assert_eq(make_tetradeca(...), expected);
   ```

2. **Variant B: 13-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add a bench witness where a function returns `[3][2][2][2][2][2][2][2][2][2][2][2][2]Pt`
   (393,216-bit total packed width, 12,288 elements). The non-p2 outer extent is
   the strongest stress test for product-based width/index arithmetic at rank
   13, following the W569/W571 pattern.

3. **Variant C: module-level 2-D/3-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the multi-D AoS lowering to module scope: allow a module `const` or
   `var` of type `[N][M]Pt` (and perhaps `[N][M][K]Pt`) to be initialized from a
   multi-D array literal and to participate in whole-array / indexed assertions.
   This is expected to require extending module packed-array declaration and the
   constant-eval / initializer paths, and may touch the Lean lowerability
   predicate.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w580_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W581
  variants recorded in `.trinity/current-issue.md`.
