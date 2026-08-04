# Wave Loop 845 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1630 (expected)
- **PR:** #1631 (expected)
- **Branch:** `wave-loop-845`
- **Parent branch:** `wave-loop-844` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[509][2]^6 Pt`

Same established pattern as W844, outer dimension +2 to 509, `MID_IDX = 509 // 2 = 254`.

- 509 × 64 = 32,576 elements
- 32,576 × 32 bits = 1,042,432-bit packed vector (~0.994 MiBit)
- Generator: copy `scripts/gen_w844.py` → `scripts/gen_w845.py`
- Spec: `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27`
- Test name: `accepts_w845_bench_module_509x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27`
- module header f-string: `module w845_bench_module_509x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 254`
- Verify with: `grep -n "module w845\|w844\|OUTER = \|MID_IDX" scripts/gen_w845.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 305/0)

## Variant B (stretch): `[507][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 507 × 192 = 97,344 elements
- 97,344 × 32 bits = 3,115,008-bit packed vector (~2.971 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[507][2]^6 Pt` negative-index writes

Keep the W844 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~32k elements produces ~96k lines; still comfortably within CLI limits.
- **Backend width cliff:** no new cliff expected up to ~1.042 MBit; Icarus and the t27c lowering path have been stable through W844.

## Close-out checklist

- [ ] `scripts/gen_w845.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 305/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W845_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-846.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-845.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1630`, push branch `wave-loop-845`, open PR #1631
