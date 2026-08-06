# Wave Loop 868 Plan

**Issue:** #1684  
**Branch:** `wave-loop-868` (from `wave-loop-867` HEAD)  
**Previous branch PR:** #1683 (`wave-loop-867`)  
**Variant A:** module-scope `[555][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Constants

```text
OUTER    = 555
INNER    = 2
DEPTH    = 6
TOTAL    = 555 * 2^6 = 35,520
LAST_IDX = 554
MID_IDX  = 277
BITS     = 35,520 * 32 = 1,136,640 (~1.084 MiBit)
```

## Decomposed work

1. **Generator**
   - Copy `scripts/gen_w867.py` → `scripts/gen_w868.py`.
   - Fix the three recurring stale-reference locations before running:
     - destination path inside the generator
     - module header f-string (`w868_bench_module_555x2p6_aos_var_call_write`)
     - `MID_IDX` comment to `277`
   - Verify with: `grep -n "w867\|553\|# 276" scripts/gen_w868.py` (expect empty).
2. **Witness**
   - Generate `specs/scratch/w868_bench_module_555x2p6_aos_var_call_write.t27`.
3. **Validation**
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
4. **Test**
   - Add `accepts_w868_bench_module_555x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W868_2026-08-05.md`.
   - Write `.claude/plans/wave-loop-869.md` with variants for W869.
   - Update `.trinity/experience.md`, `docs/NOW.md`,
     `.trinity/current-issue.md`, skill trackers, and persistent memory.
6. **Land**
   - Commit with `Closes #1684`.
   - Push branch `wave-loop-868`.
   - Open PR to `master`.

## Cooperation variants for Wave Loop 869

| Variant | Shape | Outer | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[557][2]^6 Pt` | 557 | 35,648 | 1,140,736 | ~1.088 | Continue mechanical outer += 2 ladder. |
| **B** | `[555][3]^6 Pt` | 555 | 53,760 | 1,720,320 | ~1.640 | Increase second inner dimension, stride pressure. |
| **C** | `[555][2]^6 Pt` (neg-index writes) | 555 | 35,520 | 1,136,640 | ~1.084 | Negative indices / wrap-around writes. |

## Scientific background and weak points to watch

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline; it allocates until memory is
  exhausted. At ~1.084 MiBit we are still far from any practical hard boundary.
  The next meaningful watch-point remains the established 4-MiBit soft cliff
  (~131,072 elements for 32-bit structs).
- **Vitis HLS UG1399 `compact=bit`:** commercial analog for packing structs into
  wide vectors. Internal module variables are not subject to the 8192-bit port
  limit, so the comparison with t27c is about internal representation fidelity,
  not IO pin width.
- **Vericert / CompCert:** verified C-to-Verilog HLS framework. The
  `t27c icarus-cocotb` gate provides a lightweight reference-model equivalence
  check conceptually adjacent to Vericert's translation-validation approach.
- **FPGA Roofline (Siracusa et al., IEEE TC 2021):** each wider vector grows the
  memory quanta `Q` while the compute roof stays flat. We remain on the soft,
  memory-bandwidth-limited side of the wall.
- **Persistent generator copy hazard:** three locations to fix every wave until
  the template is parameterized.
- No compiler, reference-model, or `FROZEN_HASH` changes anticipated.
