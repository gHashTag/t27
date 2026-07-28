# FPGA Loop Closeout — Wave Loop 811

**Date:** 2026-07-24  
**Issue:** #1551  
**Branch:** `wave-loop-811`  
**Base:** `wave-loop-810` @ `cee47fbbe63020ee8150ccbf4e3e0ab25f722e03` (because earlier wave PRs remain open)

## Variant executed

**A — `[441][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1551.
2. Created and pushed branch `wave-loop-811` from `wave-loop-810` HEAD `cee47fbbe63020ee8150ccbf4e3e0ab25f722e03`.
3. Copied `scripts/gen_w810.py` to `scripts/gen_w811.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w811_bench_module_441x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w811_bench_module_441x2p6_aos_var_call_write`
   - `OUTER = 441`, `MID_IDX = 220`.
4. Generated witness `specs/scratch/w811_bench_module_441x2p6_aos_var_call_write.t27`:
   - 28,224 elements
   - 903,168-bit packed vector (~0.861 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w811_bench_module_441x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w811_bench_module_441x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
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
| `cargo test -p t27c --test icarus_lowerable` | 271/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W811 | PASS |
| `t27c icarus-lowerable` W811 | PASS |
| `t27c icarus-simulate` W811 | PASS (17 cycles) |
| `t27c icarus-cocotb` W811 | PASS |
| `t27c seal --save` W811 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W811 = 39 waves.
- Witness element count: 28,224.
- Packed vector width: 903,168 bits (~0.861 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.
- FROZEN_HASH changes: 0.

## Learnings

- The generator copy hazard still spans two text locations (destination path and module header f-string). Fixing both before the first run remains the only manual failure mode. The stale `MID_IDX` comment was also corrected to `220` for W811.
- `make_grid(32768)` period-identity check continues to catch offset wrap-around congruent to 0 modulo 32768.
- `assert_ne` is not emitted by the Icarus simulation path, so bench sites verify changed element values with `assert_eq` instead.
- The packed-vector AoS lowering remains robust without compiler changes up to at least `[441][2]^6 Pt`.

## Cooperation variants for W812

- **A (recommended):** `[443][2]^6 Pt`, outer += 2, MID_IDX = 221.
- **B:** `[441][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[441][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Artifacts

- `scripts/gen_w811.py`
- `specs/scratch/w811_bench_module_441x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` (new test)
- `.trinity/seals/scratch_w811_bench_module_441x2p6_aos_var_call_write.json`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W811_2026-07-24.md`
- `.claude/plans/wave-loop-812.md`

## Next step

Create issue #1553, branch `wave-loop-812` from `wave-loop-811` HEAD, and execute the recommended variant A.
