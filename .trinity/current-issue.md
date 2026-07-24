# Wave Loop 789 — Issue #1507

**Branch:** `wave-loop-789`
**Parent branch:** `wave-loop-788` HEAD (`44fa559e7`)
**Date:** 2026-07-24
**Issue:** #1507
**PR:** #1508
**Cooperation variant:** A (recommended)
**Status:** closeout complete

## Goal

Close Wave Loop 789 by validating a module-scope `[397][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Earlier wave PRs remain open awaiting review, so W789 was branched from
`wave-loop-788` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w789_bench_module_397x2p6_aos_var_call_write.t27` is generated and parses.
2. [x] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [x] The cocotb reference model matches the t27 semantics.
4. [x] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [x] All cargo suites remain green, including `cargo clippy -p t27c`.
6. [x] Integration test `accepts_w789_bench_module_397x2p6_aos_var_call_write` is added.
7. [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W789_2026-07-24.md` is written.
8. [x] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [x] `.claude/plans/wave-loop-790.md` with three cooperation variants is created.
10. [ ] PR reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Shape: `[397][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `397 x 64 = 25,408`.
- Packed vector width: `25,408 x 32 = 813,056` bits (~0.775 MiBit).
- `MID_IDX = 198`; frame-condition element `[198][1][0][0][0][0][0]` is element
  number `198*64 + 32 = 12,704`.
- Generator script: `scripts/gen_w789.py` (copy from `scripts/gen_w788.py`, set
  `OUTER = 397` and `MID_IDX = 198`, fix module prefix including the hardcoded
  wave prefix inside the f-string header).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[399][2]^6 Pt`.
- **Variant B:** keep width at ~0.775 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
