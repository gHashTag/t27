# Wave Loop 775 — Issue TBD

**Branch:** `wave-loop-775`  
**Parent branch:** `wave-loop-774` HEAD (`433102763`)  
**Date:** 2026-07-24  
**Issue:** #1485  
**PR:** #1486 (TBD)  
**Cooperation variant:** A (recommended)  
**Status:** implementation complete, pending PR review and merge

## Goal

Close Wave Loop 775 by validating a module-scope `[369][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. W774 PR #1484 is
still open awaiting review, so W775 was branched from `wave-loop-774` HEAD
to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w775_bench_module_369x2p6_aos_var_call_write.t27` is generated and parses.
2. [x] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [x] The cocotb reference model matches the t27 semantics.
4. [x] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [x] All cargo suites remain green.
6. [x] Integration test `accepts_w775_bench_module_369x2p6_aos_var_call_write` is added.
7. [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W775_2026-07-24.md` is written.
8. [x] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [ ] GitHub issue for W775 is created and PR #<issue+1> opened with `Closes #<issue>`.
10. [ ] PR #<issue+1> reviewed and merged to `master` (or stacked after W774 lands).

## Technical notes

- Shape: `[369][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `369 x 64 = 23,616`.
- Packed vector width: `23,616 x 32 = 755,712` bits (~0.721 MiBit).
- `MID_IDX = 184`; frame-condition element `[184][1][0][0][0][0][0]` is element
  `184*64 + 32 = 11,808`.
- Generator script: `scripts/gen_w775.py` (copy from `scripts/gen_w774.py`, set
  `OUTER = 369` and `MID_IDX = 184`, manually fix the f-string header).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[371][2]^6 Pt`.
- **Variant B:** keep width at ~0.721 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
