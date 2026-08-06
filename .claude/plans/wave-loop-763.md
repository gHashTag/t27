# Wave Loop 763 Plan

**Issue:** #1734
**Branch:** `wave-loop-763`
**Date:** 2026-07-23

## Goal

Close Wave Loop 763 by validating a module-scope `[345][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

## PHI LOOP decomposition

- [x] **Issue** - Confirm #1734 scope and Variant A.
- [x] **Spec** - Generate `specs/scratch/w763_bench_module_345x2p6_aos_var_call_write.t27`.
- [x] **TDD** - Include `test` write schedule and `bench` read-back assertions.
- [x] **Impl** - No compiler changes; reuse W632 inner-dimension offset formula.
- [x] **Gen** - Run direct witness generation from `scripts/gen_w763.py`.
- [x] **Seal** - `t27c seal --save` the witness.
- [x] **Verify** - `parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, cargo suites.
- [ ] **Land** - Merge `wave-loop-763` to `master` with `Closes #1734`.
- [x] **Learn** - Save W763 memory + update `skills-wave-loop-recipe.md` + `MEMORY.md`.

## Three cooperation variants for next Wave Loop

1. **Variant A (recommended):** continue the odd outer-dimension ladder with `[347][2]^6 Pt`.
2. **Variant B:** keep width at ~0.670 MiBit but move the packed var to bench/function scope.
3. **Variant C:** add `if`-guarded indexed signed field writes at the current width.

## Definition of done

All acceptance criteria in `.trinity/current-issue.md` are green and the
closeout report is written to `docs/reports/FPGA_LOOP_CLOSEOUT_W763_2026-07-23.md`.

---

phi^2 + 1/phi^2 = 3 | TRINITY
