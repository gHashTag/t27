# Wave Loop 867 Plan

**Issue:** #1682  
**Branch:** `wave-loop-867` (from `wave-loop-866` HEAD)  
**Previous branch PR:** #1681 (`wave-loop-866`)  
**Variant A:** module-scope `[553][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 553
INNER    = 2
DEPTH    = 6
TOTAL    = 553 * 2^6 = 35,392
LAST_IDX = 552
MID_IDX  = 276
BITS     = 35,392 * 32 = 1,132,544 (~1.080 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w866.py` → `scripts/gen_w867.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path inside the generator
     - module header f-string (`w867_bench_module_553x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `276`
2. **Witness**
   - Generate `specs/scratch/w867_bench_module_553x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w867_bench_module_553x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W867_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-868.md` with variants for W868.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1682`.
   - Push branch `wave-loop-867`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 868

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[555][2]^6 Pt` | 555 | 35,520 | 1,136,640 | ~1.084 | Continue mechanical outer += 2 ladder. |
| **B** | `[553][3]^6 Pt` | 553 | 53,184 | 1,701,888 | ~1.622 | Increase second inner dimension, stride pressure. |
| **C** | `[553][2]^6 Pt` (neg-index writes) | 553 | 35,392 | 1,132,544 | ~1.080 | Negative indices / wrap-around writes. |

## Weak points to watch

- Icarus width soft cliff still expected near 4 MiBit.
- Parse dump size will keep growing with vector width.
- Generator copy hazard: three locations to fix every wave.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
