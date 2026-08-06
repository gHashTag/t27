# Wave Loop 846 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1632 (expected)
- **PR:** #1633 (expected)
- **Branch:** `wave-loop-846`
- **Parent branch:** `wave-loop-845` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[511][2]^6 Pt`

Same established pattern as W845, outer dimension +2 to 511, `MID_IDX = 511 // 2 = 255`.

- 511 × 64 = 32,704 elements
- 32,704 × 32 bits = 1,046,528-bit packed vector (~0.998 MiBit)
- Generator: copy `scripts/gen_w845.py` → `scripts/gen_w846.py`
- Spec: `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`
- Test name: `accepts_w846_bench_module_511x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`
- module header f-string: `module w846_bench_module_511x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 255`
- Verify with: `grep -n "module w846\|w845\|OUTER = \|MID_IDX" scripts/gen_w846.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 306/0)

## Variant B (stretch): `[509][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 509 × 192 = 97,728 elements
- 97,728 × 32 bits = 3,127,296-bit packed vector (~2.982 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[509][2]^6 Pt` negative-index writes

Keep the W845 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~32k elements produces ~96k lines; still comfortably within CLI limits.
- **Backend width cliff:** no new cliff expected up to ~1.047 MBit; Icarus and the t27c lowering path have been stable through W845.

## Close-out checklist

- [ ] `scripts/gen_w846.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 306/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W846_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-847.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-846.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1632`, push branch `wave-loop-846`, open PR #1633
