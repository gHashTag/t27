# Wave Loop 765 Plan

**Issue:** #1736
**Branch:** `wave-loop-765`
**Date:** 2026-07-23

## Goal

Close Wave Loop 765 by validating a module-scope `[349][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

## PHI LOOP decomposition

- [ ] **Issue** - Confirm #1736 scope and Variant A.
- [ ] **Spec** - Generate `specs/scratch/w765_bench_module_349x2p6_aos_var_call_write.t27`.
- [ ] **TDD** - Include `test` write schedule and `bench` read-back assertions.
- [ ] **Impl** - No compiler changes; reuse W632 inner-dimension offset formula.
- [ ] **Gen** - Run direct witness generation from `scripts/gen_w765.py`.
- [ ] **Seal** - `t27c seal --save` the witness.
- [ ] **Verify** - `parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, cargo suites.
- [ ] **Land** - Merge `wave-loop-765` to `master` with `Closes #1736`.
- [ ] **Learn** - Save W765 memory + update `skills-wave-loop-recipe.md` + `MEMORY.md`.

## Three cooperation variants for next Wave Loop

1. **Variant A (recommended):** continue the odd outer-dimension ladder with `[351][2]^6 Pt`.
2. **Variant B:** keep width at ~0.682 MiBit but move the packed var to bench/function scope.
3. **Variant C:** add `if`-guarded indexed signed field writes at the current width.

## Definition of done

All acceptance criteria in `.trinity/current-issue.md` are green and the
closeout report is written to `docs/reports/FPGA_LOOP_CLOSEOUT_W765_2026-07-23.md`.

---

phi^2 + 1/phi^2 = 3 | TRINITY
