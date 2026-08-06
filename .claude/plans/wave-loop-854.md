# Wave Loop 854 — Cooperation Plan (2026-08-05)

## Proposed issue/PR

- **Issue:** #1648 (expected)
- **PR:** #1649 (expected)
- **Branch:** `wave-loop-854`
- **Parent branch:** `wave-loop-853` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[527][2]^6 Pt`

Same established pattern as W853, outer dimension +2 to 527, `MID_IDX = 527 // 2 = 263`.

- 527 × 64 = 33,728 elements
- 33,728 × 32 bits = 1,079,296-bit packed vector (~1.030 MiBit)
- Generator: copy `scripts/gen_w853.py` → `scripts/gen_w854.py`
- Spec: `specs/scratch/w854_bench_module_527x2p6_aos_var_call_write.t27`
- Test name: `accepts_w854_bench_module_527x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w854_bench_module_527x2p6_aos_var_call_write.t27`
- module header f-string: `module w854_bench_module_527x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 263`
- Verify with: `grep -n "module w854\|w853\|OUTER = \|MID_IDX" scripts/gen_w854.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 314/0)

## Variant B (stretch): `[525][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 525 × 192 = 100,656 elements
- 100,656 × 32 bits = 3,220,992-bit packed vector (~3.073 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[525][2]^6 Pt` negative-index writes

Keep the W853 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~33k elements produces ~100k lines; still comfortably within CLI limits.
- **Backend width cliff:** Variant A remains far below the 4-MiBit hard ceiling; no new cliff expected. Variant B is the real width probe.
- **Icarus outer-dimension variable-index limitation:** t27c lowering already avoids this; monitor simulator output if Variant C is tried.

## Research notes

- IEEE 1800-2017 only mandates 65,536-bit packed-array support; Icarus warns near 1 Gbit, not 1 Mbit. Recent upstream commit `128c621` fixed a bound-normalization bug that could accidentally produce billion-bit vectors.
- Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors; modern Icarus does not exhibit this limit.
- Siracusa et al. (IEEE TC 2021) Roofline memory-quanta model frames the ladder as a probe of how wide `Q` can grow before routing/host-memory costs dominate.
- Vericert (Herklotz et al., OOPSLA 2021) and CompCert (Leroy, 2009) provide verified-compilation analogs for bit-exact source-to-hardware mappings.
- Vitis HLS UG1399 `compact=bit` is the commercial analog for packing interface structs into wide vectors.

## Close-out checklist

- [ ] `scripts/gen_w854.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 314/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W854_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-855.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-853.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1646`, push branch `wave-loop-853`, open PR #1647
