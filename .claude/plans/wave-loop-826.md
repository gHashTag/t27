# Wave Loop 826 Plan

**Date:** 2026-08-01  
**Issue:** TBD  
**Branch:** `wave-loop-826`

## Goal

Continue the mechanical module-scope packed array-of-struct ladder from `[469][2]^6 Pt` to the next rung, choosing among three cooperation variants.

## Variants

### A — `[471][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes (recommended)

- Outer dimension increases by 2 to 471; inner dimensions stay `[2]^6`.
- Expected witness: 30,144 elements, 964,608-bit packed vector (~0.920 MiBit).
- Keeps the established mechanical generator pattern unchanged.
- Generator `scripts/gen_w826.py` with `OUTER = 471`, `MID_IDX = 235`; fix copy hazard before first run.
- Witness `specs/scratch/w826_bench_module_471x2p6_aos_var_call_write.t27`.
- Add integration test `accepts_w826_bench_module_471x2p6_aos_var_call_write`.
- Expected zero compiler / reference-model / FROZEN_HASH changes.

### B — `[469][3]^6 Pt` — grow the second inner dimension to stress stride scaling

- Keep outer dimension 469; change inner shape from `[2]^6` to `[3]^6`.
- This exercises a different stride scaling direction: inner dimension 3 produces a 3^6 = 729 inner product, yielding 469 × 729 = 341,901 elements and 10,940,832 bits (~10.43 MiBit), which crosses the 4-MiBit cliff. Use only if the maintainers explicitly want to probe that boundary; otherwise it is intentionally more disruptive.

### C — `[469][2]^6 Pt` with negative-index writes to exercise wrap-around addressing

- Same outer/inner dimensions as variant A (469) but add negative signed index writes inside the bench block (e.g., `dst[-1][...] = ...`).
- Verifies that the Verilog backend handles signed index wrap-around correctly.
- Requires careful choice of negative indices so the wrapped positions are deterministic and the reference model agrees.

## Recommended variant

**A** — the mechanical ladder is still well below the 4-MiBit cliff at `[471][2]^6 Pt` (~0.920 MiBit), and the generator/test/seal pipeline is fully reusable. Variant A minimizes risk and keeps the ladder moving predictably.

## Acceptance criteria

- [ ] Generator `scripts/gen_w826.py` with `OUTER = 471`, `MID_IDX = 235`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w826_bench_module_471x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w826_bench_module_471x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #<issue>`, push branch, open PR to `master`.
