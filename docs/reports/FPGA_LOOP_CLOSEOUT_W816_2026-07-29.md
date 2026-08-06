# FPGA Loop Closeout — Wave Loop 816

**Date:** 2026-07-29  
**Issue:** #1561  
**Branch:** `wave-loop-816`  
**Base:** `wave-loop-815` @ `e7d0c594d` (because earlier wave PRs remain open)

## Variant executed

**A — `[451][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1561.
2. Created and pushed branch `wave-loop-816` from `wave-loop-815` HEAD `e7d0c594d`.
3. Copied `scripts/gen_w815.py` to `scripts/gen_w816.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w816_bench_module_451x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w816_bench_module_451x2p6_aos_var_call_write`
   - `OUTER = 451`, `MID_IDX = 225`.
4. Generated witness `specs/scratch/w816_bench_module_451x2p6_aos_var_call_write.t27`:
   - 28,864 elements
   - 923,648-bit packed vector (~0.881 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w816_bench_module_451x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w816_bench_module_451x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. Confirmed `bootstrap/stage0/FROZEN_HASH` is unchanged.
8. Performed weak-point audit and refreshed 2025–2026 ternary/MVL literature scan; no new actionable items found.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo clippy -p t27c` | green (780 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed, 0 failed, 2 ignored |
| `cargo test -p tri` | 78/0 |
| `cargo test -p flash-spi` | 2/0 |
| `cargo test -p t27c --test bitnet_pipeline` | 20/0 |
| `cargo test -p t27c --test bitnet_top` | 17/0 |
| `cargo test -p t27c --test icarus_lowerable` | 276/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W816 | PASS |
| `t27c icarus-lowerable` W816 | PASS |
| `t27c icarus-simulate` W816 | PASS (17 cycles) |
| `t27c icarus-cocotb` W816 | PASS |
| `t27c seal --save` W816 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W816 = 44 waves.
- Witness element count: 28,864.
- Packed vector width: 923,648 bits (~0.881 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.
- FROZEN_HASH changes: 0.

## Learnings

- The generator copy hazard still spans two text locations (destination path and module header f-string). Fixing both before the first run remains the only manual failure mode. The stale `MID_IDX` comment was also corrected to `225` for W816.
- For `OUTER = 451`, `MID_IDX = 225`; the frame-condition element is `[225][1][0][0][0][0][0]`, element number `225 * 64 + 32 = 14,432`.
- `make_grid(32768)` period-identity check continues to catch offset wrap-around congruent to 0 modulo 32768.
- `assert_ne` is not emitted by the Icarus simulation path, so bench sites verify changed element values with `assert_eq` instead.
- The packed-vector AoS lowering remains robust without compiler changes up to at least `[451][2]^6 Pt`.

## Cooperation variants for W817

- **A (recommended):** `[453][2]^6 Pt`, outer += 2, MID_IDX = 226.
- **B:** `[451][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[451][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Artifacts

- `scripts/gen_w816.py`
- `specs/scratch/w816_bench_module_451x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` (new test)
- `.trinity/seals/scratch_w816_bench_module_451x2p6_aos_var_call_write.json`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W816_2026-07-29.md`
- `.claude/plans/wave-loop-817.md` (to write)

## Next step

Create issue #1563, branch `wave-loop-817` from `wave-loop-816` HEAD, and execute the recommended variant A.
