# Wave Loop 824 Plan

**Date:** 2026-07-29  
**Issue:** TBD  
**Branch:** `wave-loop-824`

## Goal

Continue the mechanical module-scope packed array-of-struct ladder from `[465][2]^6 Pt` to the next rung, choosing among three cooperation variants.

## Variants

### A — `[467][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes (recommended)

- Outer dimension increases by 2 to 467; inner dimensions stay `[2]^6`.
- Expected witness: 29,888 elements, 956,416-bit packed vector (~0.912 MiBit).
- Keeps the established mechanical generator pattern unchanged.
- Generator `scripts/gen_w824.py` with `OUTER = 467`, `MID_IDX = 233`; fix copy hazard before first run.
- Witness `specs/scratch/w824_bench_module_467x2p6_aos_var_call_write.t27`.
- Add integration test `accepts_w824_bench_module_467x2p6_aos_var_call_write`.
- Expected zero compiler / reference-model / FROZEN_HASH changes.

### B — `[465][3]^6 Pt` — grow the second inner dimension to stress stride scaling

- Keep outer dimension 465; change inner shape from `[2]^6` to `[3]^6`.
- This exercises a different stride scaling direction: inner dimension 3 produces a 3^6 = 729 inner product, yielding 465 × 729 = 338,985 elements and 10,847,520 bits (~10.35 MiBit), which crosses the 4-MiBit cliff. Use only if the maintainers explicitly want to probe that boundary; otherwise it is intentionally more disruptive.

### C — `[465][2]^6 Pt` with negative-index writes to exercise wrap-around addressing

- Same outer/inner dimensions as variant A (465) but add negative signed index writes inside the bench block (e.g., `dst[-1][...] = ...`).
- Verifies that the Verilog backend handles signed index wrap-around correctly.
- Requires careful choice of negative indices so the wrapped positions are deterministic and the reference model agrees.

## Recommended variant

**A** — the mechanical ladder is still well below the 4-MiBit cliff at `[467][2]^6 Pt` (~0.912 MiBit), and the generator/test/seal pipeline is fully reusable. Variant A minimizes risk and keeps the ladder moving predictably.

## Acceptance criteria

- [ ] Generator `scripts/gen_w824.py` with `OUTER = 467`, `MID_IDX = 233`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w824_bench_module_467x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w824_bench_module_467x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #<issue>`, push branch, open PR to `master`.
