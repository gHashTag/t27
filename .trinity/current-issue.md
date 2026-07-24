# Wave Loop 784 — Issue #1497

**Branch:** `wave-loop-784`
**Parent branch:** `wave-loop-783` HEAD (`7f2c7afb4`)
**Date:** 2026-07-24
**Issue:** #1497
**PR:** (to open after closeout)
**Cooperation variant:** A (recommended)
**Status:** closeout complete

## Goal

Close Wave Loop 784 by validating a module-scope `[387][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Earlier wave PRs remain open awaiting review, so W784 was branched from
`wave-loop-783` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w784_bench_module_387x2p6_aos_var_call_write.t27` is generated and parses.
2. [x] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [x] The cocotb reference model matches the t27 semantics.
4. [x] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [x] All cargo suites remain green, including `cargo clippy -p t27c`.
6. [x] Integration test `accepts_w784_bench_module_387x2p6_aos_var_call_write` is added.
7. [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W784_2026-07-24.md` is written.
8. [x] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [x] `.claude/plans/wave-loop-785.md` with three cooperation variants is created.
10. [ ] PR reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Shape: `[387][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `387 x 64 = 24,768`.
- Packed vector width: `24,768 x 32 = 792,576` bits (~0.756 MiBit).
- `MID_IDX = 193`; frame-condition element `[193][1][0][0][0][0][0]` is element
  `193*64 + 32 = 12,384`.
- Generator script: `scripts/gen_w784.py` (copy from `scripts/gen_w783.py`, set
  `OUTER = 387` and `MID_IDX = 193`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[389][2]^6 Pt`.
- **Variant B:** keep width at ~0.756 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
