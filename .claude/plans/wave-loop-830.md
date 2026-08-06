# Wave Loop 830 — Cooperation Plan

**Date:** 2026-08-01  
**Prev wave:** 829 (`[477][2]^6 Pt`, issue #1599, PR #1600)  
**Current wave:** 830  
**Expected issue:** #1601  
**Expected PR:** #1602  
**Base branch:** `wave-loop-829` (because earlier waves' PRs remain open)

## Goal

Extend the t27 mechanical packed-vector array-of-struct (AoS) ladder by one rung,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals
the wider packed vector without compiler or `FROZEN_HASH` changes.

## Variants

### A — Recommended: continue odd outer-dimension ladder (+2)

- Witness: `[479][2]^6 Pt` module-scope `var` initialized from a function call,
  exercised with indexed signed field writes.
- Outer dimension: 479 (odd, non-power-of-two)
- MID_IDX: 239
- Total elements: 30,656
- Packed vector width: 985,088 bits (~0.939 MiBit)
- Rationale: cheapest mechanical increment; confirms the packed-vector AoS
  lowering remains robust as the vector approaches the 4-MiBit cliff.

### B — Grow the second inner dimension to stress stride scaling

- Witness: `[477][3]^6 Pt` module-scope `var` initialized from a function call,
  exercised with indexed signed field writes.
- Outer dimension: 477 (kept)
- Inner dimensions: `[3]^6`
- Total elements: 45,792
- Packed vector width: 1,465,344 bits (~1.397 MiBit)
- Rationale: keeps the outer dimension constant and exercises multi-dimensional
  stride arithmetic, which is a different scaling axis than the ladder has
  covered so far.
- Risk: larger vector may be slower to simulate or cross the 4-MiBit tuning
  threshold; keep as fallback if Variant A succeeds and maintainers want
  broader coverage.

### C — Negative-index writes to exercise wrap-around addressing

- Witness: `[477][2]^6 Pt` module-scope `var` with additional writes using
  negative signed indices (e.g. `dst[-1]...`) to test the Verilog backend's
  wrap-around semantics in packed-vector indexing.
- Outer dimension: 477
- Total elements: 30,528
- Packed vector width: 980,992 bits (~0.934 MiBit)
- Rationale: adds semantic coverage without growing the vector size; useful if
  the ladder is paused near a width boundary and we want to stress addressing
  correctness.
- Risk: t27's signed-index semantics may not define wrap-around the way Verilog
  does; could require a compiler/behavior change and break the "no compiler
  changes" streak. Treat as exploratory.

## Recommended variant

**A** — `[479][2]^6 Pt`. It preserves the mechanical generator discipline,
requires the smallest diff, and keeps the ladder moving predictably.

## Acceptance criteria

- [ ] Generator `scripts/gen_w830.py` with `OUTER = 479`, `MID_IDX = 239`;
      copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w830_bench_module_479x2p6_aos_var_call_write.t27`
      generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and
      `seal --save` all PASS.
- [ ] Integration test `accepts_w830_bench_module_479x2p6_aos_var_call_write`
      added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and
      persistent memory updated.
- [ ] Commit with `Closes #1601`, push branch, open PR to `master`.

## References

- IEEE 1800-2017 §7.4.1/7.4.3 (packed array width as product of dimensions, no
  power-of-two restriction).
- Previous closeout: `docs/reports/FPGA_LOOP_CLOSEOUT_W829_2026-08-01.md`.

*φ² + φ⁻² = 3 | TRINITY*
