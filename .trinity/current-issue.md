# Wave Loop 574 — Current Issue

**Issue #1545** — Next step after 7-D array-of-struct return call deduplication.
**Branch:** `wave-loop-574` (to be created from `wave-loop-573`).
**Previous:** Wave Loop 573 closed (#1544, branch `wave-loop-573`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to eight dimensions and will test whether Icarus
Verilog can digest an 8192-bit nested concatenation. Variant B keeps the same
rank but introduces a non-power-of-two outer dimension, a strong stress test
for product-based width/index arithmetic. Variant C is an intentional scope
shift from local declarations to module scope; it is expected to require real
compiler work and is the natural next step once the rank-only waves have become
zero-change.

## Cooperation variants

1. **Variant A — Recommended: 8-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns `[2][2][2][2][2][2][2][2]Pt`
   (8192-bit total packed width, 256 elements) and the same call is used at
   multiple whole-array or indexed sites in one block. Verify that the recursive
   literal emission, CSE descriptor, and multi-D slice-access paths scale cleanly
   to eight dimensions. Follow the W573 witness structure (bind the literal to a
   local variable before asserting equality to avoid the Icarus `$display`
   overflow). Example:
   ```t27
   let octa : [2][2][2][2][2][2][2][2]Pt = make_octa(...);
   assert_eq(octa[0][1][0][1][1][1][1][1].x, 1);
   assert_eq(octa, make_octa(...));
   let expected : [2][2][2][2][2][2][2][2]Pt = [2][2][2][2][2][2][2][2]Pt{ ... };
   assert_eq(make_octa(...), expected);
   ```

2. **Variant B: 7-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add a bench witness where a function returns `[3][2][2][2][2][2][2]Pt`
   (6144-bit total packed width, 192 elements). The non-p2 outer extent is the
   strongest stress test for product-based width/index arithmetic at rank 7,
   following the W569/W571 pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array
   literal and to participate in whole-array / indexed assertions. This is
   expected to require extending module packed-array declaration and the
   constant-eval / initializer paths, and may touch the Lean lowerability
   predicate.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w574_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W575
  variants recorded in `.trinity/current-issue.md`.
