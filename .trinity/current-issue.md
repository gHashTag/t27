# Wave Loop 791 — Issue #1511

**Branch:** `wave-loop-791`
**Parent branch:** `wave-loop-790` HEAD (`d758b7c8a`)
**Date:** 2026-07-24
**Issue:** #1511
**PR:** #1512
**Cooperation variant:** A (recommended)
**Status:** closeout complete

## Goal

Close Wave Loop 791 by validating a module-scope `[401][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Earlier wave PRs remain open awaiting review, so W791 was branched from
`wave-loop-790` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w791_bench_module_401x2p6_aos_var_call_write.t27` is generated and parses.
2. [x] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [x] The cocotb reference model matches the t27 semantics.
4. [x] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [x] All cargo suites remain green, including `cargo clippy -p t27c`.
6. [x] Integration test `accepts_w791_bench_module_401x2p6_aos_var_call_write` is added.
7. [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W791_2026-07-24.md` is written.
8. [x] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [x] `.claude/plans/wave-loop-792.md` with three cooperation variants is created.
10. [x] Commit with `Closes #1511`, push `wave-loop-791`, open PR #1512.

## Technical notes

- Shape: `[401][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `401 x 64 = 25,664`.
- Packed vector width: `25,664 x 32 = 821,248` bits (~0.783 MiBit).
- `MID_IDX = 200`; frame-condition element `[200][1][0][0][0][0][0]` is element
  number `200*64 + 32 = 12,832`.
- Generator script: `scripts/gen_w791.py` (copy from `scripts/gen_w790.py`, set
  `OUTER = 401` and `MID_IDX = 200`, fix module prefix including the hardcoded
  wave prefix inside the f-string header).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[403][2]^6 Pt`.
- **Variant B:** keep width at ~0.783 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
