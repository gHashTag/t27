# Wave Loop 847 — Decomposed Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1634
- **PR:** #1635
- **Branch:** `wave-loop-847`
- **Parent branch:** `wave-loop-846` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

## Research and weak points

### Weak points audited

1. **The 1-MiBit psychological boundary.** W847 is the first wave whose packed
   vector exceeds 2²⁰ bits (1,050,624 bits vs. 1,048,576). The IEEE 1800-2017
   LRM only requires simulators to support packed arrays of at least 65,536 bits;
   Icarus Verilog warns around 1 Gbit, not 1 Mbit. No hard 1-MiBit limit exists in
   the standard or in Icarus.
2. **Simulator performance scaling.** Cycle count stayed flat at 17 cycles,
   confirming the 1-MiBit class is still comfortably within the event-driven
   simulator's memory model.
3. **Recurring generator copy hazard.** The hazard was prevented by grepping the
   three known stale locations before running `scripts/gen_w847.py`.
4. **Pre-existing regressions (not fixed).** `verilog_array_literal_expr.rs`
   regression and FPGA E2E CI redness remain separate issues.

### Scientific background

- **Icarus Verilog packed-array sizing:** Practical limits are memory dependent;
  no hard 1-MiBit cap.
- **FPGA Roofline model (Siracusa et al., IEEE TC 2021):** memory quanta `Q`
  and bandwidth ceilings frame the ladder as a probe of how wide packed vectors
  can grow before routing/host-memory costs dominate.
- **Vericert verified HLS (Herklotz et al., OOPSLA 2021):** bit-exact
  source-to-hardware mappings provide a correctness anchor analogous to the t27
  packed-vector identity checks.
- **Vitis HLS UG1399:** `compact=bit` for interface structs is the commercial
  analog for packing AoS data into wide vectors.

## Variant A (recommended and executed): `[513][2]^6 Pt`

Same established pattern as W846, outer dimension +2 to 513, `MID_IDX = 513 // 2 = 256`.

- 513 × 64 = 32,832 elements
- 32,832 × 32 bits = 1,050,624-bit packed vector (~1.002 MiBit)
- Generator: copy `scripts/gen_w846.py` → `scripts/gen_w847.py`
- Spec: `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27`
- Test name: `accepts_w847_bench_module_513x2p6_aos_var_call_write`

**Copy-hazard fixes applied:**
- destination path: `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27`
- module header f-string: `module w847_bench_module_513x2p6_aos_var_call_write`
- `MID_IDX` comment: `# 256`

**Validation gates (all PASS):**
1. `cargo build --release -p t27c`
2. `t27c parse`
3. `t27c icarus-lowerable`
4. `t27c icarus-simulate` (17 cycles, PASSED)
5. `t27c icarus-cocotb` (reference-model OK)
6. `t27c seal --save`
7. Integration test added; full `icarus_lowerable` suite 307/0

## Variant B (stretch): `[511][3]^6 Pt`

Grow the second inner dimension instead of the outer. This changes stride scaling and is much larger, so it is intentionally a probe that may convert to a negative-boundary witness if a backend width/stride limit is hit.

- 511 × 192 = 98,112 elements
- 98,112 × 32 bits = 3,139,584-bit packed vector (~2.993 MiBit)
- Deferred to a later wave; W847 focused on the 1-MiBit outer-dimension transition.

## Variant C (alternate): `[511][2]^6 Pt` negative-index writes

Keep the W846 outer dimension but replace signed positive writes with a mix of negative and positive indices to stress wrap-around / signed-index lowering in the packed variable.

- Deferred; can be combined with a future width probe if lowering coverage is needed.

## Risk and mitigation

- **Recurring generator copy hazard:** fixed by pre-run grep of wave number, outer dimension, and `MID_IDX`.
- **Compile-time / memory:** spec generation for ~32.8k elements produced ~97.5k lines; within CLI limits.
- **Backend width cliff:** W847 crosses the 1-MiBit line but stays far below the 4-MiBit hard ceiling; no new cliff observed.
- **Icarus outer-dimension variable-index limitation:** t27c lowering already avoids this.

## Close-out checklist

- [x] `scripts/gen_w847.py` created and copy-hazard-free
- [x] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [x] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [x] Full `icarus_lowerable` suite green (307/0)
- [x] `FROZEN_HASH` unchanged
- [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W847_2026-08-04.md`
- [x] Next-wave plan `.claude/plans/wave-loop-848.md` with variants A/B/C
- [x] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [x] Skill trackers updated
- [x] Persistent memory `wave-loop-847.md` + `MEMORY.md` index
- [x] Commit with `Closes #1634`, push branch `wave-loop-847`, open PR #1635
