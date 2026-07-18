# Wave Loop 577 — Current Issue

**Issue #1548** — Next step after 10-D array-of-struct return call deduplication.
**Branch:** `wave-loop-577` (to be created from `wave-loop-576`).
**Previous:** Wave Loop 576 closed (#1547, branch `wave-loop-576`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to eleven dimensions and tests whether Icarus
Verilog can digest a 65,536-bit nested concatenation — exactly the IEEE
1800-2017 minimum packed-vector width. Variant B keeps the same rank but
introduces a non-power-of-two outer dimension, a strong stress test for
product-based width/index arithmetic at rank 10. Variant C is an intentional
scope shift from local declarations to module scope; it is expected to require
real compiler work and is the natural next step once the rank-only waves have
become zero-change.

## Cooperation variants

1. **Variant A — Recommended: 11-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns
   `[2][2][2][2][2][2][2][2][2][2][2]Pt` (65,536-bit total packed width,
   2,048 elements) and the same call is used at multiple whole-array or indexed
   sites in one block. Verify that the recursive literal emission, CSE
   descriptor, and multi-D slice-access paths scale cleanly to eleven dimensions.
   Follow the W573–W576 witness structure (bind the literal to a local variable
   before asserting equality to avoid the Icarus `$display` overflow). Example:
   ```t27
   let undeca : [2][2][2][2][2][2][2][2][2][2][2]Pt = make_undeca(...);
   assert_eq(undeca[0][1][0][1][1][1][1][1][1][1][1].x, 1);
   assert_eq(undeca, make_undeca(...));
   let expected : [2][2][2][2][2][2][2][2][2][2][2]Pt = [2][2][2][2][2][2][2][2][2][2][2]Pt{ ... };
   assert_eq(make_undeca(...), expected);
   ```

2. **Variant B: 10-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add a bench witness where a function returns `[3][2][2][2][2][2][2][2][2][2]Pt`
   (49,152-bit total packed width, 1,536 elements). The non-p2 outer extent is
   the strongest stress test for product-based width/index arithmetic at rank
   10, following the W569/W571 pattern.

3. **Variant C: module-level 2-D/3-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the multi-D AoS lowering to module scope: allow a module `const` or
   `var` of type `[N][M]Pt` (and perhaps `[N][M][K]Pt`) to be initialized from a
   multi-D array literal and to participate in whole-array / indexed assertions.
   This is expected to require extending module packed-array declaration and the
   constant-eval / initializer paths, and may touch the Lean lowerability
   predicate.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w577_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W578
  variants recorded in `.trinity/current-issue.md`.
