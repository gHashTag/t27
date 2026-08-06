# Wave Loop 827 Cooperation Plan

**Date:** 2026-08-01  
**Current wave:** W826 closed (issue #1593 / PR #1594)  
**Next wave:** W827

## Goal

Continue the mechanical module-scope packed-array-of-struct ladder one rung higher
without compiler or `FROZEN_HASH` changes. Validate that t27c still lowers,
simulates, cocotb-matches, and seals the wider packed vector.

## Variants

### A — Recommended: `[473][2]^6 Pt` module-scope packed AoS variable from call with indexed signed writes

- Increment outer dimension by 2: `OUTER = 473`, `MID_IDX = 236`.
- Expected: 30,272 elements, 968,704-bit packed vector (~0.923 MiBit).
- Mechanical generator copy from `scripts/gen_w826.py` → `scripts/gen_w827.py`.
- Fix generator copy hazard (destination path + module header f-string + `MID_IDX` comment).
- Add integration test `accepts_w827_bench_module_473x2p6_aos_var_call_write`.
- Run all direct gates and full `icarus_lowerable` suite (expect 287/0).

### B — Stride scaling: `[471][3]^6 Pt` module-scope packed AoS variable

- Keep outer at W826 value (`471`) but grow second inner dimension from `2` to `3`.
- Expected: 471 × 3^6 × 32 = 1,095,552 bits (~1.045 MiBit), intentionally crosses the 1-MiBit line.
- Useful only if we want to stress stride scaling; may hit a width/stride cliff. Use only if explicitly requested.

### C — Wrap-around addressing: `[471][2]^6 Pt` with negative-index writes

- Keep W826 dimensions but add negative signed index writes inside a test block.
- Exercises Verilog wrap-around / two's-complement indexing semantics.
- Risk: if the reference model and Verilog disagree on negative-index behavior, this becomes a debugging wave rather than a mechanical increment. Use only if explicitly requested.

## Recommended execution

Proceed with **Variant A**. It preserves the established mechanical discipline,
keeps the diff minimal, and avoids crossing known width cliffs.

## Acceptance criteria

- [ ] Issue opened for W827; branch `wave-loop-827` from `wave-loop-826` HEAD.
- [ ] `scripts/gen_w827.py` with `OUTER = 473`, copy hazard fixed before first run.
- [ ] Witness generated and all direct gates pass.
- [ ] Integration test added; full `icarus_lowerable` suite passes.
- [ ] `FROZEN_HASH` unchanged.
- [ ] Closeout report + W828 plan + tracker/memory updates.
- [ ] Commit with `Closes #<issue>`, push, open PR.

*φ² + φ⁻² = 3 | TRINITY*
