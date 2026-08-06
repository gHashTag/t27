# Wave Loop 862 Plan

**Issue:** #1668  
**Branch:** `wave-loop-862` (from `wave-loop-861` HEAD)  
**Previous branch PR:** #1667 (`wave-loop-861`)  
**Variant A:** module-scope `[543][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 543
INNER    = 2
DEPTH    = 6
TOTAL    = 543 * 2^6 = 34,752
LAST_IDX = 542
MID_IDX  = 271
BITS     = 34,752 * 32 = 1,112,064 (~1.060 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w861.py` → `scripts/gen_w862.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path
     - module header f-string (`w862_bench_module_543x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `271`
2. **Witness**
   - Generate `specs/scratch/w862_bench_module_543x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w862_bench_module_543x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W862_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-863.md` with variants for W863.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1668`.
   - Push branch `wave-loop-862`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 863

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[545][2]^6 Pt` | 545 | 34,880 | 1,116,160 | ~1.064 | Continue mechanical outer += 2 ladder. |
| **B** | `[543][3]^6 Pt` | 543 | 52,128 | 1,668,096 | ~1.591 | Increase second inner dimension, stride pressure. |
| **C** | `[543][2]^6 Pt` (neg-index writes) | 543 | 34,752 | 1,112,064 | ~1.060 | Negative indices / wrap-around writes. |

## Weak points to watch

- Icarus width soft cliff still expected near 4 MiBit.
- Parse dump size will keep growing with vector width.
- Generator copy hazard: three locations to fix every wave.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
