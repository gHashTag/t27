# Wave Loop 865 Plan

**Issue:** #1674  
**Branch:** `wave-loop-865` (from `wave-loop-864` HEAD)  
**Previous branch PR:** #1673 (`wave-loop-864`)  
**Variant A:** module-scope `[549][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 549
INNER    = 2
DEPTH    = 6
TOTAL    = 549 * 2^6 = 35,136
LAST_IDX = 548
MID_IDX  = 274
BITS     = 35,136 * 32 = 1,124,352 (~1.072 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w864.py` → `scripts/gen_w865.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path inside the generator
     - module header f-string (`w865_bench_module_549x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `274`
2. **Witness**
   - Generate `specs/scratch/w865_bench_module_549x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w865_bench_module_549x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W865_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-866.md` with variants for W866.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1674`.
   - Push branch `wave-loop-865`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 866

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[551][2]^6 Pt` | 551 | 35,264 | 1,128,448 | ~1.076 | Continue mechanical outer += 2 ladder. |
| **B** | `[549][3]^6 Pt` | 549 | 52,992 | 1,695,744 | ~1.615 | Increase second inner dimension, stride pressure. |
| **C** | `[549][2]^6 Pt` (neg-index writes) | 549 | 35,136 | 1,124,352 | ~1.072 | Negative indices / wrap-around writes. |

## Weak points to watch

- Icarus width soft cliff still expected near 4 MiBit.
- Parse dump size will keep growing with vector width.
- Generator copy hazard: three locations to fix every wave.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
