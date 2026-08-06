# Wave Loop 836 — Cooperation Plan (2026-08-01)

## Proposed issue/PR

- **Issue:** #1612
- **PR:** #1613
- **Branch:** `wave-loop-836`
- **Parent branch:** `wave-loop-835` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[491][2]^6 Pt`

Same established pattern as W835, outer dimension +2 to 491, `MID_IDX = 491 // 2 = 245`.

- 491 × 64 = 31,424 elements
- 31,424 × 32 bits = 1,005,568-bit packed vector (~0.959 MiBit)
- Generator: copy `scripts/gen_w835.py` → `scripts/gen_w836.py`
- Spec: `specs/scratch/w836_bench_module_491x2p6_aos_var_call_write.t27`
- Test name: `accepts_w836_bench_module_491x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- `sed -i '' 's/w835/w836/g'`
- `sed -i '' 's/489/491/g'`
- `sed -i '' 's/# 244/# 245/g'`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite

## Variant B (stretch): `[489][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling
and is much larger, so it is intentionally a probe that may convert to a
negative-boundary witness if a backend width/stride limit is hit.

- 489 × 192 = 93,888 elements
- 93,888 × 32 bits = 3,004,416-bit packed vector (~2.864 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[489][2]^6 Pt` negative-index writes

Keep the W835 outer dimension but replace signed positive writes with a mix of
negative and positive indices to stress wrap-around / signed-index lowering in the
packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer
dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for 31k+ elements produces ~90k–100k
lines; still comfortably within CLI limits.
- **Backend width cliff:** no new cliff expected up to ~1.006 MBit; Icarus and the
t27c lowering path have been stable through W835.

## Close-out checklist

- [ ] `scripts/gen_w836.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 296/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W836_2026-08-01.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-837.md`
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-836.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1612`, push `wave-loop-836`, open PR #1613
