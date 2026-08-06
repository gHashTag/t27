# Wave Loop 778 — Issue #1492

**Branch:** `wave-loop-778`
**Parent branch:** `wave-loop-777` HEAD (`995d94f0c`)
**Date:** 2026-07-24
**Issue:** #1492
**PR:** #1493 (to open)
**Cooperation variant:** A (recommended)
**Status:** implementation pending, plan ready

## Goal

Close Wave Loop 778 by validating a module-scope `[375][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. W774 PR #1484,
W775 PR #1486, W776 PR #1488, W777 PR #1491, and PR #1489 (README/W774-W776
merge) remain open awaiting review, so W778 will be branched from
`wave-loop-777` HEAD to avoid blocking the sequence.

## Acceptance criteria

- [ ] Generator `scripts/gen_w863.py` with `OUTER = 545`, `MID_IDX = 272`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w863_bench_module_545x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w863_bench_module_545x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1670`, push branch, open PR to `master`.

1. [ ] `specs/scratch/w778_bench_module_375x2p6_aos_var_call_write.t27` is generated and parses.
2. [ ] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [ ] The cocotb reference model matches the t27 semantics.
4. [ ] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [ ] All cargo suites remain green.
6. [ ] Integration test `accepts_w778_bench_module_375x2p6_aos_var_call_write` is added.
7. [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W778_2026-07-24.md` is written.
8. [ ] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. [ ] `.claude/plans/wave-loop-779.md` with three cooperation variants is created.
10. [ ] PR #1493 reviewed and merged to `master` (or stacked after earlier waves land).

## Notes

- Shape: `[375][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `375 x 64 = 24,000`.
- Packed vector width: `24,000 x 32 = 768,000` bits (~0.733 MiBit).
- `MID_IDX = 187`; frame-condition element `[187][1][0][0][0][0][0]` is element
  `187*64 + 32 = 12,000`.
- Generator script: `scripts/gen_w778.py` (copy from `scripts/gen_w777.py`, set
  `OUTER = 375` and `MID_IDX = 187`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

---

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[377][2]^6 Pt`.
- **Variant B:** keep width at ~0.733 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
