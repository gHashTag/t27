# Wave Loop 844 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1628 (expected)
- **PR:** #1629 (expected)
- **Branch:** `wave-loop-844`
- **Parent branch:** `wave-loop-843` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[507][2]^6 Pt`

Same established pattern as W843, outer dimension +2 to 507, `MID_IDX = 507 // 2 = 253`.

- 507 × 64 = 32,448 elements
- 32,448 × 32 bits = 1,038,336-bit packed vector (~0.990 MiBit)
- Generator: copy `scripts/gen_w843.py` → `scripts/gen_w844.py`
- Spec: `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27`
- Test name: `accepts_w844_bench_module_507x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27`
- module header f-string: `module w844_bench_module_507x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 253`
- Verify with: `grep -n "module w844\|w843\|OUTER = \|MID_IDX" scripts/gen_w844.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 304/0)

## Variant B (stretch): `[505][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 505 × 192 = 96,960 elements
- 96,960 × 32 bits = 3,102,720-bit packed vector (~2.959 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[505][2]^6 Pt` negative-index writes

Keep the W843 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~32k elements produces ~96k lines; still comfortably within CLI limits.
- **Backend width cliff:** no new cliff expected up to ~1.038 MBit; Icarus and the t27c lowering path have been stable through W843.

## Close-out checklist

- [ ] `scripts/gen_w844.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 304/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W844_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-845.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-844.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1628`, push branch `wave-loop-844`, open PR #1629
