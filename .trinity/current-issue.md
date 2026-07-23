# Wave Loop 771 — Issue #1742

**Branch:** `wave-loop-771`  
**Date:** 2026-07-24  
**Cooperation variant:** A (recommended)

## Goal

Close Wave Loop 771 by validating a module-scope `[361][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

## Acceptance criteria

1. `specs/scratch/w771_bench_module_361x2p6_aos_var_call_write.t27` is generated and parses.
2. The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. The cocotb reference model matches the t27 semantics.
4. `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. All cargo suites remain green.
6. Integration test `accepts_w771_bench_module_361x2p6_aos_var_call_write` is added.
7. Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W771_2026-07-24.md` is written.
8. Learning is saved to `.trinity/experience.md`, memory, and `skills-wave-loop-recipe.md`.
9. Branch merges to `master` with `Closes #1742`.

## Technical notes

- Shape: `[361][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `361 x 64 = 23,104`.
- Packed vector width: `23,104 x 32 = 739,328` bits (~0.705 MiBit).
- `MID_IDX = 180`; frame-condition element `[180][1][0][0][0][0][0]` is element
  `180*64 + 32 = 11,552`.
- Generator script: `scripts/gen_w771.py` (copy from `scripts/gen_w770.py`, set
  `OUTER = 361` and `MID_IDX = 180`, manually fix the f-string header).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[363][2]^6 Pt`.
- **Variant B:** keep width at ~0.705 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
