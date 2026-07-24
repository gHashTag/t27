# Wave Loop 794 — Issue #1517

**Branch:** `wave-loop-794`
**Parent branch:** `wave-loop-793` HEAD (`d92cc7dfb`)
**Date:** 2026-07-24
**Issue:** #1517
**PR:** #1518
**Cooperation variant:** A (recommended)
**Status:** closeout complete

## Goal

Close Wave Loop 794 by validating a module-scope `[407][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Earlier wave PRs remain open awaiting review, so W794 was branched from
`wave-loop-793` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w794_bench_module_407x2p6_aos_var_call_write.t27` is generated and parses.
2. [x] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [x] The cocotb reference model matches the t27 semantics.
4. [x] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [x] All cargo suites remain green, including `cargo clippy -p t27c`.
6. [x] Integration test `accepts_w794_bench_module_407x2p6_aos_var_call_write` is added.
7. [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W794_2026-07-24.md` is written.
8. [x] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [x] `.claude/plans/wave-loop-795.md` with three cooperation variants is created.
10. [x] Commit with `Closes #1517`, push `wave-loop-794`, open PR #1518.

## Technical notes

- Shape: `[407][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `407 x 64 = 26,048`.
- Packed vector width: `26,048 x 32 = 833,536` bits (~0.795 MiBit).
- `MID_IDX = 203`; frame-condition element `[203][1][0][0][0][0][0]` is element
  number `203*64 + 32 = 13,024`.
- Generator script: `scripts/gen_w794.py` (copy from `scripts/gen_w793.py`, set
  `OUTER = 407` and `MID_IDX = 203`, fix destination path and module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[409][2]^6 Pt`.
- **Variant B:** keep width at ~0.795 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
