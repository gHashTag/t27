# Wave Loop 820 Plan

**Date:** 2026-07-29  
**Issue:** #1567  
**Branch:** `wave-loop-820`

## Goal
Continue the mechanical module-scope packed array-of-struct ladder from `[457][2]^6 Pt` to the next rung, choosing among three cooperation variants.

## Variants

### A — `[459][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes (recommended)

- Outer dimension increases by 2 to 459; inner dimensions stay `[2]^6`.
- Expected witness: 29,376 elements, 940,032-bit packed vector (~0.897 MiBit).
- Keeps the established mechanical generator pattern unchanged.
- Generator `scripts/gen_w820.py` with `OUTER = 459`, `MID_IDX = 229`; fix copy hazard before first run.
- Witness `specs/scratch/w820_bench_module_459x2p6_aos_var_call_write.t27`.
- Add integration test `accepts_w820_bench_module_459x2p6_aos_var_call_write`.
- Expected zero compiler / reference-model / FROZEN_HASH changes.

### B — `[457][3]^6 Pt` — grow the second inner dimension to stress stride scaling

- Keep outer dimension 457; change inner shape from `[2]^6` to `[3]^6`.
- This exercises a different stride scaling direction: inner dimension 3 produces a 3^6 = 729 inner product, yielding 457 × 729 = 333,153 elements and 10,660,896 bits (~10.17 MiBit), which crosses the 4-MiBit cliff. Use only if the maintainers explicitly want to probe that boundary; otherwise it is intentionally more disruptive.

### C — `[457][2]^6 Pt` with negative-index writes to exercise wrap-around addressing

- Same outer/inner dimensions as variant A (457) but add negative signed index writes inside the bench block (e.g., `dst[-1][...] = ...`).
- Verifies that the Verilog backend handles signed index wrap-around correctly.
- Requires careful choice of negative indices so the wrapped positions are deterministic and the reference model agrees.

## Recommended variant

**A** — the mechanical ladder is still well below the 4-MiBit cliff at `[459][2]^6 Pt` (~0.897 MiBit), and the generator/test/seal pipeline is fully reusable. Variant A minimizes risk and keeps the ladder moving predictably.

## Acceptance criteria

- [ ] Generator `scripts/gen_w820.py` with `OUTER = 459`, `MID_IDX = 229`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w820_bench_module_459x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w820_bench_module_459x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1567`, push branch, open PR to `master`.
