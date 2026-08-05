# Wave Loop 856 — Cooperation Plan

**Date:** 2026-08-05  
**Issue:** #1652  
**Branch:** `wave-loop-856` (from `wave-loop-855` HEAD)  

## Goal

Continue the mechanical packed-vector AoS ladder one step past W855.

## Selected variant

**A (recommended):** `[531][2]^6 Pt` module-scope non-power-of-two outer-dimension
array-of-struct variable from call with indexed signed writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 531 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 33,984 |
| Packed vector width | 1,087,488 bits |
| Approximate size | ~1.038 MiBit |
| `MID_IDX` | 265 |

## Work breakdown

1. **Generator**
   - Copy `scripts/gen_w855.py` → `scripts/gen_w856.py`.
   - Fix recurring copy hazard:
     - destination path (`w855` → `w856`, `529` → `531`)
     - module header f-string
     - `MID_IDX` comment
   - Run `python3 scripts/gen_w856.py`.

2. **Validation gates**
   - `t27c parse`
   - `t27c icarus-lowerable --json`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable accepts_w856_bench_module_531x2p6_aos_var_call_write`

3. **Integration**
   - Add Rust test in `bootstrap/tests/icarus_lowerable.rs`.
   - Confirm `FROZEN_HASH` unchanged.

4. **Close-out**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W856_2026-08-05.md`.
   - Update `.claude/skills/wave-loop-master-plan.md`,
     `.claude/skills/wave-loop-autopilot.md`, `.claude/skills/t27-wave-loop.md`.
   - Update `.trinity/experience.md` and `docs/NOW.md`.
   - Save persistent memory entry.
   - Commit with `Closes #1652`, push, open PR.
   - Create issue #1653 / branch `wave-loop-857`.

## Stop conditions

- Any validation gate fails.
- Backend emits a width or memory error.
- `FROZEN_HASH` would need to change.

*φ² + φ⁻² = 3 | TRINITY*
