# Wave Loop 851 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1642 (expected)
- **PR:** #1643 (expected)
- **Branch:** `wave-loop-851`
- **Parent branch:** `wave-loop-850` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[521][2]^6 Pt`

Same established pattern as W850, outer dimension +2 to 521, `MID_IDX = 521 // 2 = 260`.

- 521 × 64 = 33,344 elements
- 33,344 × 32 bits = 1,067,008-bit packed vector (~1.018 MiBit)
- Generator: copy `scripts/gen_w850.py` → `scripts/gen_w851.py`
- Spec: `specs/scratch/w851_bench_module_521x2p6_aos_var_call_write.t27`
- Test name: `accepts_w851_bench_module_521x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w851_bench_module_521x2p6_aos_var_call_write.t27`
- module header f-string: `module w851_bench_module_521x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 260`
- Verify with: `grep -n "module w851\|w850\|OUTER = \|MID_IDX" scripts/gen_w851.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 311/0)

## Variant B (stretch): `[519][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 519 × 192 = 99,264 elements
- 99,264 × 32 bits = 3,176,448-bit packed vector (~3.027 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[519][2]^6 Pt` negative-index writes

Keep the W850 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~33k elements produces ~99k lines; still comfortably within CLI limits.
- **Backend width cliff:** Variant A remains far below the 4-MiBit hard ceiling; no new cliff expected. Variant B is the real width probe.
- **Icarus outer-dimension variable-index limitation:** t27c lowering already avoids this; monitor simulator output if Variant C is tried.

## Research notes

- IEEE 1800-2017 only mandates 65,536-bit packed-array support; Icarus warns near 1 Gbit, not 1 Mbit. Recent upstream commit `128c621` fixed a bound-normalization bug that could accidentally produce billion-bit vectors.
- Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors; modern Icarus does not exhibit this limit.
- Siracusa et al. (IEEE TC 2021) Roofline memory-quanta model frames the ladder as a probe of how wide `Q` can grow before routing/host-memory costs dominate.
- Vericert (Herklotz et al., OOPSLA 2021) and CompCert (Leroy, 2009) provide verified-compilation analogs for bit-exact source-to-hardware mappings.
- Vitis HLS UG1399 `compact=bit` is the commercial analog for packing interface structs into wide vectors.

## Close-out checklist

- [ ] `scripts/gen_w851.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 311/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W851_YYYY-MM-DD.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-852.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-851.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1642`, push branch `wave-loop-851`, open PR #1643
