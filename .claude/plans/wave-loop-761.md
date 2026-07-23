# Wave Loop 761 Plan

**Issue:** #1732
**Branch:** `wave-loop-761`
**Date:** 2026-07-23

## Goal

Close Wave Loop 761 by validating a module-scope `[341][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

## PHI LOOP decomposition

- [x] **Issue** - Confirm #1732 scope and Variant A.
- [x] **Spec** - Generate `specs/scratch/w761_bench_module_341x2p6_aos_var_call_write.t27`.
- [x] **TDD** - Include `test` write schedule and `bench` read-back assertions.
- [x] **Impl** - No compiler changes; reuse W632 inner-dimension offset formula.
- [x] **Gen** - Run `tri gen` (or direct witness generation) from `scripts/gen_w761.py`.
- [x] **Seal** - `t27c seal --save` the witness.
- [x] **Verify** - `parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, cargo suites.
- [x] **Land** - Merge `wave-loop-761` to `master` with `Closes #1732`.
- [x] **Learn** - Save W761 memory + update `skills-wave-loop-recipe.md` + `MEMORY.md`.

## Three cooperation variants for next Wave Loop

1. **Variant A (recommended):** continue the odd outer-dimension ladder with `[343][2]^6 Pt`.
2. **Variant B:** keep width at ~0.663 MiBit but move the packed var to bench/function scope.
3. **Variant C:** add `if`-guarded indexed signed field writes at the current width.

## Definition of done

All acceptance criteria in `.trinity/current-issue.md` are green and the
closeout report is written to `docs/reports/FPGA_LOOP_CLOSEOUT_W761_2026-07-23.md`.

---

phi^2 + 1/phi^2 = 3 | TRINITY
