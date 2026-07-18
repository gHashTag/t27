# Wave Loop 579 — Current Issue

**Issue #1550** — Next step after 12-D array-of-struct return call deduplication.
**Branch:** `wave-loop-579` (to be created from `wave-loop-578`).
**Previous:** Wave Loop 578 closed (#1549, branch `wave-loop-578`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to thirteen dimensions and tests whether Icarus
Verilog can digest a 262,144-bit nested concatenation — four times the IEEE
1800-2017 minimum packed-vector width. Variant B keeps the same rank but
introduces a non-power-of-two outer dimension, a strong stress test for
product-based width/index arithmetic at rank 12. Variant C is an intentional
scope shift from local declarations to module scope; it is expected to require
real compiler work and is the natural next step once the rank-only waves have
become zero-change.

## Cooperation variants

1. **Variant A — Recommended: 13-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns
   `[2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (262,144-bit total packed width,
   8,192 elements) and the same call is used at multiple whole-array or indexed
   sites in one block. Verify that the recursive literal emission, CSE
   descriptor, and multi-D slice-access paths scale cleanly to thirteen dimensions.
   Follow the W573–W578 witness structure (bind the literal to a local variable
   before asserting equality to avoid the Icarus `$display` overflow). Example:
   ```t27
   let trideca : [2][2][2][2][2][2][2][2][2][2][2][2][2]Pt = make_trideca(...);
   assert_eq(trideca[0][1][0][1][1][1][1][1][1][1][1][1][1].x, 1);
   assert_eq(trideca, make_trideca(...));
   let expected : [2][2][2][2][2][2][2][2][2][2][2][2][2]Pt = [2][2][2][2][2][2][2][2][2][2][2][2][2]Pt{ ... };
   assert_eq(make_trideca(...), expected);
   ```

2. **Variant B: 12-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add a bench witness where a function returns `[3][2][2][2][2][2][2][2][2][2][2][2]Pt`
   (196,608-bit total packed width, 6,144 elements). The non-p2 outer extent is
   the strongest stress test for product-based width/index arithmetic at rank
   12, following the W569/W571 pattern.

3. **Variant C: module-level 2-D/3-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the multi-D AoS lowering to module scope: allow a module `const` or
   `var` of type `[N][M]Pt` (and perhaps `[N][M][K]Pt`) to be initialized from a
   multi-D array literal and to participate in whole-array / indexed assertions.
   This is expected to require extending module packed-array declaration and the
   constant-eval / initializer paths, and may touch the Lean lowerability
   predicate.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w579_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W580
  variants recorded in `.trinity/current-issue.md`.
