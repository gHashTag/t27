# Wave Loop 780 — Issue #1496

**Branch:** `wave-loop-780`
**Parent branch:** `wave-loop-779` HEAD
**Date:** 2026-07-24
**Issue:** #1496
**PR:** #1497 (to open)
**Cooperation variant:** A (recommended)
**Status:** implementation pending, plan ready

## Goal

Close Wave Loop 780 by validating a module-scope `[379][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. W774 PR #1484,
W775 PR #1486, W776 PR #1488, W777 PR #1491, W778 PR #1493, W779 PR #1495,
and PR #1489 (README/W774-W776 merge) remain open awaiting review, so W780
will be branched from `wave-loop-779` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [ ] `specs/scratch/w780_bench_module_379x2p6_aos_var_call_write.t27` is generated and parses.
2. [ ] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [ ] The cocotb reference model matches the t27 semantics.
4. [ ] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [ ] All cargo suites remain green.
6. [ ] Integration test `accepts_w780_bench_module_379x2p6_aos_var_call_write` is added.
7. [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W780_2026-07-24.md` is written.
8. [ ] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [ ] `.claude/plans/wave-loop-781.md` with three cooperation variants is created.
10. [ ] PR #1497 reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Shape: `[379][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `379 x 64 = 24,256`.
- Packed vector width: `24,256 x 32 = 776,192` bits (~0.741 MiBit).
- `MID_IDX = 189`; frame-condition element `[189][1][0][0][0][0][0]` is element
  `189*64 + 32 = 12,128`.
- Generator script: `scripts/gen_w780.py` (copy from `scripts/gen_w779.py`, set
  `OUTER = 379` and `MID_IDX = 189`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[381][2]^6 Pt`.
- **Variant B:** keep width at ~0.741 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
