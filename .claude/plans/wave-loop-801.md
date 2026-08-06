# Wave Loop 801 Plan

**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Date:** 2026-07-24
**Current wave:** 801
**Parent branch:** `wave-loop-800` HEAD (`TBD`)
**Next branch:** `wave-loop-801`
**Recommended cooperation variant:** A

## Goal

Extend the module-scope packed array-of-struct witness ladder by one rung to
`[421][2]^6 Pt`, producing 26,944 elements (862,208-bit packed vector, ~0.822
MiBit). Continue demonstrating that the t27 lowering pipeline handles
non-power-of-two outer dimensions without compiler or reference-model changes.

## Acceptance criteria

- [ ] Branch `wave-loop-801` created from `wave-loop-800` HEAD.
- [ ] GitHub issue #1531 opened (label `wave-loop` omitted if label missing).
- [ ] Generator script `scripts/gen_w801.py` created from `gen_w800.py` with
      `OUTER = 421`, `MID_IDX = 210`, destination `specs/scratch/w801_bench_module_421x2p6_aos_var_call_write.t27`,
      and module header manually fixed for stale `w800`/`419` references.
- [ ] Witness `specs/scratch/w801_bench_module_421x2p6_aos_var_call_write.t27` generated.
- [ ] `t27c parse` PASS.
- [ ] `t27c icarus-lowerable` PASS (`lowerable`).
- [ ] `t27c icarus-simulate` PASS (17 cycles, PASSED).
- [ ] `t27c icarus-cocotb` PASS (reference-model OK).
- [ ] `t27c seal --save` PASS and seal JSON saved.
- [ ] Integration test `accepts_w801_bench_module_421x2p6_aos_var_call_write` added to
      `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `cargo build --release -p t27c` green.
- [ ] `cargo clippy -p t27c` green (780 warnings, 0 errors).
- [ ] `cargo test -p t27c --test icarus_lowerable` green (261 tests).
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W801_2026-07-24.md`.
- [ ] Next-wave plan `.claude/plans/wave-loop-802.md`.
- [ ] `docs/NOW.md` updated.
- [ ] `.trinity/current-issue.md` updated for W802.
- [ ] `.trinity/experience.md` updated.
- [ ] `.claude/skills/t27-wave-loop.md` live tracker updated to wave 802.
- [ ] Commit with `Closes #1531`, push branch, open PR.

## Cooperation variants

- **Variant A (recommended):** `[421][2]^6 Pt` module-scope odd-dimension ladder.
  Low-risk, zero expected compiler changes, keeps the mechanical ladder moving.
- **Variant B:** function-scope `[421][2]^6 Pt` packed var to exercise local
  non-power-of-two arrays.
- **Variant C:** add `if`-guarded indexed signed writes to the same `[419][2]^6 Pt`
  width to exercise control-flow interaction.

## Notes

- Generator copy hazard: after copying from W800, fix line containing destination
  path and the module header f-string (both carry stale `w800`/`419` references).
- Maintain multi-line brace style required by structural classifier.
- `assert_ne` is not emitted by Icarus simulation path; continue using `assert_eq`
  on changed elements.
- Offset/period identity check: with 26,944 elements the last raw `x` is 15630,
  so the offset-0 schedule still wraps naturally.
- FROZEN_HASH is expected to remain unchanged.

---

φ² + 1/φ⁻² = 3 | TRINITY
