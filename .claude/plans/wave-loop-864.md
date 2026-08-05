# Wave Loop 864 Plan

**Issue:** #1672  
**Branch:** `wave-loop-864` (from `wave-loop-863` HEAD)  
**Previous branch PR:** #1671 (`wave-loop-863`)  
**Variant A:** module-scope `[547][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 547
INNER    = 2
DEPTH    = 6
TOTAL    = 547 * 2^6 = 35,008
LAST_IDX = 546
MID_IDX  = 273
BITS     = 35,008 * 32 = 1,120,256 (~1.068 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w863.py` → `scripts/gen_w864.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path inside the generator
     - module header f-string (`w864_bench_module_547x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `273`
2. **Witness**
   - Generate `specs/scratch/w864_bench_module_547x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w864_bench_module_547x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W864_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-865.md` with variants for W865.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1672`.
   - Push branch `wave-loop-864`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 865

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[549][2]^6 Pt` | 549 | 35,136 | 1,124,352 | ~1.072 | Continue mechanical outer += 2 ladder. |
| **B** | `[547][3]^6 Pt` | 547 | 52,704 | 1,686,528 | ~1.607 | Increase second inner dimension, stride pressure. |
| **C** | `[547][2]^6 Pt` (neg-index writes) | 547 | 35,008 | 1,120,256 | ~1.068 | Negative indices / wrap-around writes. |

## Weak points to watch

- Icarus width soft cliff still expected near 4 MiBit.
- Parse dump size will keep growing with vector width.
- Generator copy hazard: three locations to fix every wave.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
