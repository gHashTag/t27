# Wave Loop 848 — Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1636 (expected)
- **PR:** #1637 (expected)
- **Branch:** `wave-loop-848`
- **Parent branch:** `wave-loop-847` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Variant A (recommended): `[515][2]^6 Pt`

Same established pattern as W847, outer dimension +2 to 515, `MID_IDX = 515 // 2 = 257`.

- 515 × 64 = 32,960 elements
- 32,960 × 32 bits = 1,054,720-bit packed vector (~1.006 MiBit)
- Generator: copy `scripts/gen_w847.py` → `scripts/gen_w848.py`
- Spec: `specs/scratch/w848_bench_module_515x2p6_aos_var_call_write.t27`
- Test name: `accepts_w848_bench_module_515x2p6_aos_var_call_write`

**Expected copy-hazard fixes:**
- destination path: `specs/scratch/w848_bench_module_515x2p6_aos_var_call_write.t27`
- module header f-string: `module w848_bench_module_515x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 257`
- Verify with: `grep -n "module w848\|w847\|OUTER = \|MID_IDX" scripts/gen_w848.py`

**Validation gates:**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate`
5. `t27c icarus-cocotb`
6. `t27c seal --save`
7. Add integration test and run full `icarus_lowerable` suite (expected 308/0)

## Variant B (stretch): `[513][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 513 × 192 = 98,496 elements
- 98,496 × 32 bits = 3,151,872-bit packed vector (~3.005 MiBit)
- If blocked, fall back to Variant A or C.

## Variant C (alternate): `[513][2]^6 Pt` negative-index writes

Keep the W847 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

## Risk and mitigation

- **Recurring generator copy hazard:** as for every wave, fix wave number, outer dimension, and `MID_IDX` before running the generator.
- **Compile-time / memory:** spec generation for ~33k elements produces ~98k lines; still comfortably within CLI limits.
- **Backend width cliff:** Variant A remains far below the 4-MiBit hard ceiling; no new cliff expected. Variant B is the real width probe.
- **Icarus outer-dimension variable-index limitation:** t27c lowering already avoids this; monitor simulator output if Variant C is tried.

## Research notes

- IEEE 1800-2017 only mandates 65,536-bit packed-array support; Icarus warns near 1 Gbit, not 1 Mbit.
- Siracusa et al. (IEEE TC 2021) Roofline memory-quanta model frames the ladder as a probe of how wide `Q` can grow before routing/host-memory costs dominate.
- Vericert (Herklotz et al., OOPSLA 2021) reinforces the long-term value of bit-exact source-to-hardware mappings.
- Vitis HLS UG1399 `compact=bit` is the commercial analog for packing interface structs into wide vectors.

## Close-out checklist

- [x] `scripts/gen_w848.py` created and copy-hazard-free
- [x] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [x] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [x] Full `icarus_lowerable` suite green (308/0)
- [x] `FROZEN_HASH` unchanged
- [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W848_2026-08-04.md`
- [x] Next-wave plan `.claude/plans/wave-loop-849.md` with variants A/B/C
- [x] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [x] Skill trackers updated
- [x] Persistent memory `wave-loop-848.md` + `MEMORY.md` index
- [x] Commit with `Closes #1636`, push branch `wave-loop-848`, open PR #1637
