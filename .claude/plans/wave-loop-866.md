# Wave Loop 866 Plan

**Issue:** #1680  
**Branch:** `wave-loop-866` (from `wave-loop-865` HEAD)  
**Previous branch PR:** #1679 (`wave-loop-865`)  
**Variant A:** module-scope `[551][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 551
INNER    = 2
DEPTH    = 6
TOTAL    = 551 * 2^6 = 35,264
LAST_IDX = 550
MID_IDX  = 275
BITS     = 35,264 * 32 = 1,128,448 (~1.076 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w865.py` → `scripts/gen_w866.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path inside the generator
     - module header f-string (`w866_bench_module_551x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `275`
2. **Witness**
   - Generate `specs/scratch/w866_bench_module_551x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w866_bench_module_551x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W866_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-867.md` with variants for W867.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1680`.
   - Push branch `wave-loop-866`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 867

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[553][2]^6 Pt` | 553 | 35,392 | 1,132,544 | ~1.080 | Continue mechanical outer += 2 ladder. |
| **B** | `[551][3]^6 Pt` | 551 | 52,992 | 1,695,744 | ~1.615 | Increase second inner dimension, stride pressure. |
| **C** | `[551][2]^6 Pt` (neg-index writes) | 551 | 35,264 | 1,128,448 | ~1.076 | Negative indices / wrap-around writes. |

## Weak points to watch

- Icarus width soft cliff still expected near 4 MiBit.
- Parse dump size will keep growing with vector width.
- Generator copy hazard: three locations to fix every wave.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
