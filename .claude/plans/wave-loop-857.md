# Wave Loop 857 — Cooperation Plan

**Date:** 2026-08-05  
**Issue:** #1654  
**Branch:** `wave-loop-857` (from `wave-loop-856` HEAD)  

## Goal

Continue the mechanical packed-vector AoS ladder one step past W856.

## Selected variant

**A (recommended):** `[533][2]^6 Pt` module-scope non-power-of-two outer-dimension
array-of-struct variable from call with indexed signed writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 533 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 34,112 |
| Packed vector width | 1,091,584 bits |
| Approximate size | ~1.042 MiBit |
| `MID_IDX` | 266 |

## Work breakdown

1. **Generator**
   - Copy `scripts/gen_w856.py` → `scripts/gen_w857.py`.
   - Fix recurring copy hazard:
     - destination path (`w856` → `w857`, `531` → `533`)
     - module header f-string
     - `MID_IDX` comment
   - Run `python3 scripts/gen_w857.py`.

2. **Validation gates**
   - `t27c parse`
   - `t27c icarus-lowerable --json`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable accepts_w857_bench_module_533x2p6_aos_var_call_write`

3. **Integration**
   - Add Rust test in `bootstrap/tests/icarus_lowerable.rs`.
   - Confirm `FROZEN_HASH` unchanged.

4. **Close-out**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W857_2026-08-05.md`.
   - Update `.claude/skills/wave-loop-master-plan.md`,
     `.claude/skills/wave-loop-autopilot.md`, `.claude/skills/t27-wave-loop.md`.
   - Update `.trinity/experience.md` and `docs/NOW.md`.
   - Save persistent memory entry.
   - Commit with `Closes #1654`, push, open PR.
   - Create issue #1655 / branch `wave-loop-858`.

## Stop conditions

- Any validation gate fails.
- Backend emits a width or memory error.
- `FROZEN_HASH` would need to change.

*φ² + φ⁻² = 3 | TRINITY*
