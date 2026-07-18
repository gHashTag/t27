# Wave Loop 584 — Current Issue

**Issue #1555** — Next step after module-scope 3-D array-of-struct constant
whole-array comparison (W583).
**Branch:** `wave-loop-584` (to be created from `wave-loop-583`).
**Previous:** Wave Loop 583 closed (#1554, branch `wave-loop-583`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A continues the rank-scaling
sequence that dominated W566–W582. Variant B keeps rank 16 but introduces a
non-power-of-two outer dimension at the 4-MiBit scale. Variant C extends the
module-scope work started in W583 to multi-site call deduplication and
module-level variable initialization from calls.

## Cooperation variants

1. **Variant A — Recommended: 17-D array-of-struct return call deduplication.**
   Add a bench witness where a function returns
   `[2]^17 Pt` (4,194,304-bit total packed width, 131,072 elements) and the same
   call is used at multiple whole-array or indexed sites in one block. Verify that
   the recursive literal emission, CSE descriptor, and multi-D slice-access paths
   scale cleanly to seventeen dimensions. Follow the W573–W582 witness structure
   (bind the literal to a local variable before asserting equality to avoid the
   Icarus `$display` overflow). Remember that `Pt { x: i16, y: i16 }` requires
   indexed element indices `e` with `2*e+1 ≤ 32767`, i.e. `e ≤ 16383`.

2. **Variant B: 16-D array-of-struct return with a non-power-of-two outer
   dimension.**
   Add a bench witness where a function returns
   `[3][2]^16 Pt` (3,932,160-bit total packed width, 196,608 elements). The
   non-p2 outer extent is the strongest stress test for product-based
   width/index arithmetic at rank 16, following the W569/W571 pattern. Indexed
   probes must still respect the signed `i16` field range (`e ≤ 16383`).

3. **Variant C: module-level 3-D array-of-struct variable initialization and
   multi-site call deduplication.**
   Extend W583 to a module `var dst : [2][2][2]Pt = make_cube(0)` initialized
   from a function-call return, or to multiple call sites reading the module
   const and asserting equality. This exercises interaction between W557
   call-array CSE and module-scope constants/variables while keeping file
   size small.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w584_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W585
  variants recorded in `.trinity/current-issue.md`.
