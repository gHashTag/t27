# Wave Loop 858 — Cooperation Plan

**Date:** 2026-08-05  
**Issue:** #1656  
**Branch:** `wave-loop-858` (from `wave-loop-857` HEAD)  

## Goal

Continue the mechanical packed-vector AoS ladder one step past W857.

## Selected variant

**A (recommended):** `[535][2]^6 Pt` module-scope non-power-of-two outer-dimension
array-of-struct variable from call with indexed signed writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 535 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 34,240 |
| Packed vector width | 1,095,680 bits |
| Approximate size | ~1.045 MiBit |
| `MID_IDX` | 267 |

## Work breakdown

1. **Generator**
   - Copy `scripts/gen_w857.py` → `scripts/gen_w858.py`.
   - Fix recurring copy hazard:
     - destination path (`w857` → `w858`, `533` → `535`)
     - module header f-string
     - `MID_IDX` comment
   - Run `python3 scripts/gen_w858.py`.

2. **Validation gates**
   - `t27c parse`
   - `t27c icarus-lowerable --json`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable accepts_w858_bench_module_535x2p6_aos_var_call_write`

3. **Integration**
   - Add Rust test in `bootstrap/tests/icarus_lowerable.rs`.
   - Confirm `FROZEN_HASH` unchanged.

4. **Close-out**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W858_2026-08-05.md`.
   - Update `.claude/skills/wave-loop-master-plan.md`,
     `.claude/skills/wave-loop-autopilot.md`, `.claude/skills/t27-wave-loop.md`.
   - Update `.trinity/experience.md` and `docs/NOW.md`.
   - Save persistent memory entry.
   - Commit with `Closes #1656`, push, open PR.
   - Create issue #1657 / branch `wave-loop-859`.

## Stop conditions

- Any validation gate fails.
- Backend emits a width or memory error.
- `FROZEN_HASH` would need to change.

*φ² + φ⁻² = 3 | TRINITY*
