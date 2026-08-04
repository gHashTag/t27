# Wave Loop 847 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1634 (expected)
- **PR:** #1635 (expected)
- **Branch:** `wave-loop-847`
- **Parent branch:** `wave-loop-846` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[513][2]^6 Pt`

Same established pattern as W846, outer dimension +2 to 513, `MID_IDX = 513 // 2 = 256`.

- 513 × 64 = 32,832 elements
- 32,832 × 32 bits = 1,050,624-bit packed vector (~1.002 MiBit)
- Generator: copy `scripts/gen_w846.py` → `scripts/gen_w847.py`
- Spec: `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27`
- Test name: `accepts_w847_bench_module_513x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27`
- module header f-string: `module w847_bench_module_513x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 256`
- Verify with: `grep -n "module w847\|w846\|OUTER = \|MID_IDX" scripts/gen_w847.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 307/0)

## Variant B (stretch): `[511][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 511 × 192 = 98,112 elements
- 98,112 × 32 bits = 3,139,584-bit packed vector (~2.993 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[511][2]^6 Pt` negative-index writes

Keep the W846 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~32k elements produces ~97k lines; still comfortably within CLI limits.
- **Backend width cliff:** Variant A crosses the 1-MiBit psychological line but stays far below the 4-MiBit hard ceiling; no new cliff expected. Variant B is the real width probe.
- **Icarus outer-dimension variable-index limitation:** t27c lowering already avoids this; monitor simulator output if Variant C is tried.

## Close-out checklist

- [ ] `scripts/gen_w847.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 307/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W847_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-848.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-847.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1634`, push branch `wave-loop-847`, open PR #1635
