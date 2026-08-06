# FPGA Loop Closeout — Wave Loop 807

**Date:** 2026-07-24  
**Issue:** #1543  
**Branch:** `wave-loop-807`  
**Base:** `wave-loop-806` @ `7729d479f7183ef59a9b74bab56d916ea48ebd93` (because earlier wave PRs remain open)

## Variant executed

**A — `[433][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1543.
2. Created and pushed branch `wave-loop-807` from `wave-loop-806` HEAD `7729d479f7183ef59a9b74bab56d916ea48ebd93`.
3. Copied `scripts/gen_w806.py` to `scripts/gen_w807.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w807_bench_module_433x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w807_bench_module_433x2p6_aos_var_call_write`
   - `OUTER = 433`, `MID_IDX = 216`.
4. Generated witness `specs/scratch/w807_bench_module_433x2p6_aos_var_call_write.t27`:
   - 27,712 elements
   - 886,784-bit packed vector (~0.845 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w807_bench_module_433x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w807_bench_module_433x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
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
| `cargo test -p t27c --test icarus_lowerable` | 267/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W807 | PASS |
| `t27c icarus-lowerable` W807 | PASS |
| `t27c icarus-simulate` W807 | PASS (17 cycles) |
| `t27c icarus-cocotb` W807 | PASS |
| `t27c seal --save` W807 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W807 = 35 waves.
- Witness element count: 27,712.
- Packed vector width: 886,784 bits (~0.845 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.
- FROZEN_HASH changes: 0.

## Learnings

- The generator copy hazard still spans two text locations (destination path and module header f-string). Fixing both before the first run remains the only manual failure mode.
- `make_grid(32768)` period-identity check continues to catch offset wrap-around congruent to 0 modulo 32768.
- `assert_ne` is not emitted by the Icarus simulation path, so bench sites verify changed element values with `assert_eq` instead.
- The packed-vector AoS lowering remains robust without compiler changes up to at least `[433][2]^6 Pt`.

## Cooperation variants for W808

- **A (recommended):** `[435][2]^6 Pt`, outer += 2, MID_IDX = 217.
- **B:** `[433][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[433][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Artifacts

- `scripts/gen_w807.py`
- `specs/scratch/w807_bench_module_433x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` (new test)
- `.trinity/seals/scratch_w807_bench_module_433x2p6_aos_var_call_write.json`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W807_2026-07-24.md`
- `.claude/plans/wave-loop-808.md`

## Next step

Create issue #1545, branch `wave-loop-808` from `wave-loop-807` HEAD, and execute the recommended variant A.
