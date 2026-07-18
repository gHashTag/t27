# Wave Loop 582 — Current Issue

**Issue #1553** — Next step after 15-D array-of-struct return call deduplication.
**Branch:** `wave-loop-582` (to be created from `wave-loop-581`).
**Previous:** Wave Loop 581 closed (#1552, branch `wave-loop-581`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it pushes
the rank-agnostic machinery to sixteen dimensions and tests whether Icarus
Verilog can digest a 2,097,152-bit nested concatenation — thirty-two times the
IEEE 1800-2017 minimum packed-vector width. Variant B keeps the same rank but
introduces a non-power-of-two outer dimension, a strong stress test for
product-based width/index arithmetic at rank 15. Variant C is an intentional
scope shift from local declarations to module scope; it is expected to require
real compiler work and is the natural next step once the rank-only waves have
become zero-change.

## Cooperation variants

1. **Variant A — Recommended: 16-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns
   `[2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (2,097,152-bit total packed width,
   65,536 elements) and the same call is used at multiple whole-array or indexed
   sites in one block. Verify that the recursive literal emission, CSE
   descriptor, and multi-D slice-access paths scale cleanly to sixteen dimensions.
   Follow the W573–W581 witness structure (bind the literal to a local variable
   before asserting equality to avoid the Icarus `$display` overflow). Remember
   that `Pt { x: i16, y: i16 }` requires indexed element indices `e` with
   `2*e+1 ≤ 32767`, i.e. `e ≤ 16383`.

2. **Variant B: 15-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add a bench witness where a function returns `[3][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt`
   (1,572,864-bit total packed width, 49,152 elements). The non-p2 outer extent is
   the strongest stress test for product-based width/index arithmetic at rank
   15, following the W569/W571 pattern. Indexed probes must still respect the
   signed `i16` field range (`e ≤ 16383`).

3. **Variant C: module-level 2-D/3-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the multi-D AoS lowering to module scope: allow a module `const` or
   `var` of type `[N][M]Pt` (and perhaps `[N][M][K]Pt`) to be initialized from a
   multi-D array literal and to participate in whole-array / indexed assertions.
   This is expected to require extending module packed-array declaration and the
   constant-eval / initializer paths, and may touch the Lean lowerability
   predicate.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w582_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W583
  variants recorded in `.trinity/current-issue.md`.
