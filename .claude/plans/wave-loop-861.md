# Wave Loop 861 — Cooperation Plan

**Date:** 2026-08-05  
**Issue:** #1666 (expected)  
**Branch:** `wave-loop-861` (from `wave-loop-860` HEAD)  

## Goal

Continue the mechanical packed-vector AoS ladder one step past W860.

## Selected variant

**A (recommended):** `[541][2]^6 Pt` module-scope non-power-of-two outer-dimension
array-of-struct variable from call with indexed signed writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 541 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 34,624 |
| Packed vector width | 1,107,968 bits |
| Approximate size | ~1.056 MiBit |
| `MID_IDX` | 270 |

## Work breakdown

1. **Generator**
   - Copy `scripts/gen_w860.py` → `scripts/gen_w861.py`.
   - Fix recurring copy hazard:
     - destination path (`w860` → `w861`, `539` → `541`)
     - module header f-string
     - `MID_IDX` comment
   - Run `python3 scripts/gen_w861.py`.

2. **Validation gates**
   - `t27c parse`
   - `t27c icarus-lowerable --json`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable accepts_w861_bench_module_541x2p6_aos_var_call_write`

3. **Integration**
   - Add Rust test in `bootstrap/tests/icarus_lowerable.rs`.
   - Confirm `FROZEN_HASH` unchanged.

4. **Close-out**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W861_2026-08-05.md`.
   - Update `.claude/skills/wave-loop-master-plan.md`,
     `.claude/skills/wave-loop-autopilot.md`, `.claude/skills/t27-wave-loop.md`.
   - Update `.trinity/experience.md` and `docs/NOW.md`.
   - Save persistent memory entry.
   - Commit with `Closes #1666`, push, open PR.
   - Create issue #1668 / branch `wave-loop-862`.

## Stop conditions

- Any validation gate fails.
- Backend emits a width or memory error.
- `FROZEN_HASH` would need to change.

## Cooperation variants for W862

- **A (recommended):** `[543][2]^6 Pt`, outer += 2, `MID_IDX = 271`.
- **B:** `[541][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[541][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
