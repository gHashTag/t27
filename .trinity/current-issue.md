# Wave Loop 776 — Issue #1487

**Branch:** `wave-loop-776`  
**Parent branch:** `wave-loop-775` HEAD (`2e86eb0b8`)  
**Date:** 2026-07-24  
**Issue:** #1487  
**PR:** #1488 (TBD)  
**Cooperation variant:** A (recommended)  
**Status:** implementation complete, pending PR review and merge

## Goal

Close Wave Loop 776 by validating a module-scope `[371][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. W774 PR #1484 and W775
PR #1486 are still open awaiting review, so W776 was branched from
`wave-loop-775` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27` is generated and parses.
2. [x] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [x] The cocotb reference model matches the t27 semantics.
4. [x] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [x] All cargo suites remain green.
6. [x] Integration test `accepts_w776_bench_module_371x2p6_aos_var_call_write` is added.
7. [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W776_2026-07-24.md` is written.
8. [x] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [x] `.claude/plans/wave-loop-777.md` with three cooperation variants is created.
10. [ ] PR #1488 reviewed and merged to `master` (or stacked after W774/W775 land).

## Technical notes

- Shape: `[371][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `371 x 64 = 23,744`.
- Packed vector width: `23,744 x 32 = 759,808` bits (~0.725 MiBit).
- `MID_IDX = 185`; frame-condition element `[185][1][0][0][0][0][0]` is element
  `185*64 + 32 = 11,872`.
- Generator script: `scripts/gen_w776.py` (copy from `scripts/gen_w775.py`, set
  `OUTER = 371` and `MID_IDX = 185`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[373][2]^6 Pt`.
- **Variant B:** keep width at ~0.725 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
