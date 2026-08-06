# Wave Loop 802 Plan

**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Date:** 2026-07-24
**Current wave:** 802
**Parent branch:** `wave-loop-801` HEAD (`TBD`)
**Next branch:** `wave-loop-802`
**Recommended cooperation variant:** A

## Goal

Extend the module-scope packed array-of-struct witness ladder by one rung to
`[423][2]^6 Pt`, producing 27,072 elements (866,304-bit packed vector, ~0.826
MiBit). Continue demonstrating that the t27 lowering pipeline handles
non-power-of-two outer dimensions without compiler or reference-model changes.

## Acceptance criteria

- [ ] Branch `wave-loop-802` created from `wave-loop-801` HEAD.
- [ ] GitHub issue #1533 opened (label `wave-loop` omitted if label missing).
- [ ] Generator script `scripts/gen_w802.py` created from `gen_w801.py` with
      `OUTER = 423`, `MID_IDX = 211`, destination `specs/scratch/w802_bench_module_423x2p6_aos_var_call_write.t27`,
      and module header manually fixed for stale `w801`/`421` references.
- [ ] Witness `specs/scratch/w802_bench_module_423x2p6_aos_var_call_write.t27` generated.
- [ ] `t27c parse` PASS.
- [ ] `t27c icarus-lowerable` PASS (`lowerable`).
- [ ] `t27c icarus-simulate` PASS (17 cycles, PASSED).
- [ ] `t27c icarus-cocotb` PASS (reference-model OK).
- [ ] `t27c seal --save` PASS and seal JSON saved.
- [ ] Integration test `accepts_w802_bench_module_423x2p6_aos_var_call_write` added to
      `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `cargo build --release -p t27c` green.
- [ ] `cargo clippy -p t27c` green (780 warnings, 0 errors).
- [ ] `cargo test -p t27c --test icarus_lowerable` green (262 tests).
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W802_2026-07-24.md`.
- [ ] Next-wave plan `.claude/plans/wave-loop-803.md`.
- [ ] `docs/NOW.md` updated.
- [ ] `.trinity/current-issue.md` updated for W803.
- [ ] `.trinity/experience.md` updated.
- [ ] `.claude/skills/t27-wave-loop.md` live tracker updated to wave 803.
- [ ] Commit with `Closes #1533`, push branch, open PR.

## Cooperation variants

- **Variant A (recommended):** `[423][2]^6 Pt` module-scope odd-dimension ladder.
  Low-risk, zero expected compiler changes, keeps the mechanical ladder moving.
- **Variant B:** function-scope `[423][2]^6 Pt` packed var to exercise local
  non-power-of-two arrays.
- **Variant C:** add `if`-guarded indexed signed writes to the same `[421][2]^6 Pt`
  width to exercise control-flow interaction.

## Notes

- Generator copy hazard: after copying from W801, fix line containing destination
  path and the module header f-string (both carry stale `w801`/`421` references).
- Maintain multi-line brace style required by structural classifier.
- `assert_ne` is not emitted by Icarus simulation path; continue using `assert_eq`
  on changed elements.
- Offset/period identity check: with 27,072 elements the last raw `x` is 15662,
  so the offset-0 schedule still wraps naturally.
- FROZEN_HASH is expected to remain unchanged.

---

φ² + 1/φ² = 3 | TRINITY
