# Wave Loop 565 — Current Issue

**Issue #1536** — Next step after whole-array comparison for 1-D arrays of scalar structs.  
**Branch:** `wave-loop-565` (to be created from `wave-loop-564`).  
**Previous:** Wave Loop 564 closed (#1535, branch `wave-loop-564`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it stress-tests
the W563/W564 call-CSE and whole-array assertion paths together at multiple whole-array
use sites.

## Cooperation variants

1. **Variant A — Recommended: multi-site whole-array AoS call deduplication.**  
   Extend the W563 block-scoped call-CSE machinery so that a whole `[N]Pt` call used
   at multiple whole-array `assert_eq` sites in the same bench block shares one
   packed-vector temporary. Add a bench witness such as:
   ```t27
   let t : [2]Pt = make_pts(1, 2, 3, 4);
   assert_eq(t, make_pts(1, 2, 3, 4));
   assert_eq(make_pts(1, 2, 3, 4), [2]Pt{ Pt{ .x = 1, .y = 2 }, Pt{ .x = 3, .y = 4 } });
   ```
   Verify that the second and third uses reference the same predeclared call
   temporary.

2. **Variant B: 2-D array-of-struct return call deduplication.**  
   Generalize W563 to function calls returning 2-D arrays of scalar structs
   (`[N][M]Pt`). Verify that the existing multi-D local/field access paths and the
   new CSE descriptor cooperate correctly, and add a bench witness.

3. **Variant C: negative / boundary witnesses for non-lowerable array-of-struct
   returns.**  
   Add witnesses where a function returns `[N]Pt` and `Pt` contains `string`,
   `enum`, `f32`, or an unresolved-import field. Prove the structural classifier
   rejects the whole return type, so the W563 CSE optimization cannot fire.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w565_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W566
  variants recorded in `.trinity/current-issue.md`.
