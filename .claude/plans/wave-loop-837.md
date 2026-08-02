# Wave Loop 837 — Cooperation Plan (2026-08-01)

## Proposed issue/PR

- **Issue:** #1614
- **PR:** #1615
- **Branch:** `wave-loop-837`
- **Parent branch:** `wave-loop-836` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[493][2]^6 Pt`

Same established pattern as W836, outer dimension +2 to 493, `MID_IDX = 493 // 2 = 246`.

- 493 × 64 = 31,552 elements
- 31,552 × 32 bits = 1,009,664-bit packed vector (~0.963 MiBit)
- Generator: copy `scripts/gen_w836.py` → `scripts/gen_w837.py`
- Spec: `specs/scratch/w837_bench_module_493x2p6_aos_var_call_write.t27`
- Test name: `accepts_w837_bench_module_493x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- `sed -i '' 's/w836/w837/g'`
- `sed -i '' 's/491/493/g'`
- `sed -i '' 's/# 245/# 246/g'`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite

## Variant B (stretch): `[491][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling
and is much larger, so it is intentionally a probe that may convert to a
negative-boundary witness if a backend width/stride limit is hit.

- 491 × 192 = 94,272 elements
- 94,272 × 32 bits = 3,016,704-bit packed vector (~2.877 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[491][2]^6 Pt` negative-index writes

Keep the W836 outer dimension but replace signed positive writes with a mix of
negative and positive indices to stress wrap-around / signed-index lowering in the
packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer
dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for 31k+ elements produces ~90k–100k
lines; still comfortably within CLI limits.
- **Backend width cliff:** no new cliff expected up to ~1.010 MBit; Icarus and the
t27c lowering path have been stable through W836.

## Close-out checklist

- [ ] `scripts/gen_w837.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 297/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W837_2026-08-01.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-838.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-837.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1614`, push branch `wave-loop-837`, open PR #1615
