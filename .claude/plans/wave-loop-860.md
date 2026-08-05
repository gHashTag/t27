# Wave Loop 860 — Cooperation Plan

**Date:** 2026-08-05  
**Issue:** #1664 (expected)  
**Branch:** `wave-loop-860` (from `wave-loop-859` HEAD)  

## Goal

Continue the mechanical packed-vector AoS ladder one step past W859.

## Selected variant

**A (recommended):** `[539][2]^6 Pt` module-scope non-power-of-two outer-dimension
array-of-struct variable from call with indexed signed writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 539 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 34,496 |
| Packed vector width | 1,103,872 bits |
| Approximate size | ~1.052 MiBit |
| `MID_IDX` | 269 |

## Work breakdown

1. **Generator**
   - Copy `scripts/gen_w859.py` → `scripts/gen_w860.py`.
   - Fix recurring copy hazard:
     - destination path (`w859` → `w860`, `537` → `539`)
     - module header f-string
     - `MID_IDX` comment
   - Run `python3 scripts/gen_w860.py`.

2. **Validation gates**
   - `t27c parse`
   - `t27c icarus-lowerable --json`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable accepts_w860_bench_module_539x2p6_aos_var_call_write`

3. **Integration**
   - Add Rust test in `bootstrap/tests/icarus_lowerable.rs`.
   - Confirm `FROZEN_HASH` unchanged.

4. **Close-out**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W860_2026-08-05.md`.
   - Update `.claude/skills/wave-loop-master-plan.md`,
     `.claude/skills/wave-loop-autopilot.md`, `.claude/skills/t27-wave-loop.md`.
   - Update `.trinity/experience.md` and `docs/NOW.md`.
   - Save persistent memory entry.
   - Commit with `Closes #1664`, push, open PR.
   - Create issue #1666 / branch `wave-loop-861`.

## Stop conditions

- Any validation gate fails.
- Backend emits a width or memory error.
- `FROZEN_HASH` would need to change.

## Cooperation variants for W861

- **A (recommended):** `[541][2]^6 Pt`, outer += 2, `MID_IDX = 270`.
- **B:** `[539][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[539][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
