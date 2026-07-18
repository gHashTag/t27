# Wave Loop 566 — Current Issue

**Issue #1537** — Next step after multi-site whole-array AoS call deduplication.  
**Branch:** `wave-loop-566` (to be created from `wave-loop-565`).  
**Previous:** Wave Loop 565 closed (#1536, branch `wave-loop-565`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it moves
from 1-D to 2-D arrays of scalar structs, stress-testing the multi-D CSE
descriptor and slice-access paths together.

## Cooperation variants

1. **Variant A — Recommended: 2-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns `[2][3]Pt` and the same call is
   used at multiple whole-array or indexed sites in one block. Verify that the
   W563 CSE descriptor (`call_returning_cse_value_info` already parses multi-D
   arrays) and the existing multi-D slice access paths cooperate. Example:
   ```t27
   let t : [2][3]Pt = make_grid(...);
   assert_eq(t[0][1].x, 1);
   assert_eq(t, make_grid(...));
   assert_eq(make_grid(...), [2][3]Pt{ ... });
   ```

2. **Variant B: whole-array `assert_eq` for 2-D arrays of scalar structs.**  
   Extend W564 to allow `[N][M]Pt{...}` array literals as whole-array expected
   values in bench `assert_eq`. This may require only a witness, or a small
   width-inference / literal-emission adjustment if multi-D struct array
   literals are not yet handled in `ExprArrayLiteral`.

3. **Variant C: negative / boundary witnesses for non-lowerable 2-D
   array-of-struct returns.**  
   Add witnesses where a function returns `[N][M]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field. Prove the structural
   classifier rejects the whole return type.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w566_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W567
  variants recorded in `.trinity/current-issue.md`.
