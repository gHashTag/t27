# Wave Loop 839 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1618
- **PR:** #1619
- **Branch:** `wave-loop-839`
- **Parent branch:** `wave-loop-838` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[497][2]^6 Pt`

Same established pattern as W838, outer dimension +2 to 497, `MID_IDX = 497 // 2 = 248`.

- 497 × 64 = 31,792 elements
- 31,792 × 32 bits = 1,017,344-bit packed vector (~0.970 MiBit)
- Generator: copy `scripts/gen_w838.py` → `scripts/gen_w839.py`
- Spec: `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27`
- Test name: `accepts_w839_bench_module_497x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27`
- module header f-string: `module w839_bench_module_497x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 248`
- Verify with: `grep -n "module w839\|w838\|OUTER = \|MID_IDX" scripts/gen_w839.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 299/0)

## Variant B (stretch): `[495][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 495 × 192 = 94,656 elements
- 94,656 × 32 bits = 3,028,992-bit packed vector (~2.888 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[495][2]^6 Pt` negative-index writes

Keep the W838 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for 31k+ elements produces ~90k–100k lines; still comfortably within CLI limits.
- **Backend width cliff:** no new cliff expected up to ~1.017 MBit; Icarus and the t27c lowering path have been stable through W838.

## Close-out checklist

- [ ] `scripts/gen_w839.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 299/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W839_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-840.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-839.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1618`, push branch `wave-loop-839`, open PR #1619
