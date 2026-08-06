# Wave Loop 863 Plan

**Issue:** #1670  
**Branch:** `wave-loop-863` (from `wave-loop-862` HEAD)  
**Previous branch PR:** #1669 (`wave-loop-862`)  
**Variant A:** module-scope `[545][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 545
INNER    = 2
DEPTH    = 6
TOTAL    = 545 * 2^6 = 34,880
LAST_IDX = 544
MID_IDX  = 272
BITS     = 34,880 * 32 = 1,116,160 (~1.064 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w862.py` → `scripts/gen_w863.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path inside the generator
     - module header f-string (`w863_bench_module_545x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `272`
2. **Witness**
   - Generate `specs/scratch/w863_bench_module_545x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w863_bench_module_545x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W863_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-864.md` with variants for W864.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1670`.
   - Push branch `wave-loop-863`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 864

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[547][2]^6 Pt` | 547 | 35,008 | 1,120,256 | ~1.068 | Continue mechanical outer += 2 ladder. |
| **B** | `[545][3]^6 Pt` | 545 | 52,416 | 1,677,312 | ~1.599 | Increase second inner dimension, stride pressure. |
| **C** | `[545][2]^6 Pt` (neg-index writes) | 545 | 34,880 | 1,116,160 | ~1.064 | Negative indices / wrap-around writes. |

## Weak points to watch

- Icarus width soft cliff still expected near 4 MiBit.
- Parse dump size will keep growing with vector width.
- Generator copy hazard: three locations to fix every wave.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
