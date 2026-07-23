# Wave Loop 766 — Issue #1737

**Branch:** `wave-loop-766`  
**Date:** 2026-07-23  
**Cooperation variant:** A (recommended)

## Goal

Close Wave Loop 766 by validating a module-scope `[351][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

## Acceptance criteria

1. `specs/scratch/w766_bench_module_351x2p6_aos_var_call_write.t27` is generated and parses.
2. The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. The cocotb reference model matches the t27 semantics.
4. `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. All cargo suites remain green.
6. Integration test `accepts_w766_bench_module_351x2p6_aos_var_call_write` is added.
7. Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W766_2026-07-23.md` is written.
8. Learning is saved to `.trinity/experience.md`, memory, and `skills-wave-loop-recipe.md`.
9. Branch merges to `master` with `Closes #1737`.

## Technical notes

- Shape: `[351][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `351 × 64 = 22,464`.
- Packed vector width: `22,464 × 32 = 718,848` bits (~0.686 MiBit).
- `MID_IDX = 175`; frame-condition element `[175][1][0][0][0][0][0]` is element
  `175*64 + 32 = 11,232`.
- Generator script: `scripts/gen_w766.py` (copy from `scripts/gen_w765.py`, set
  `OUTER = 351` and `MID_IDX = 175`, manually fix the f-string header).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 ≡ 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[353][2]^6 Pt`.
- **Variant B:** keep width at ~0.686 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
