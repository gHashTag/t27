# FPGA Loop Closeout — Wave Loop 812

**Date:** 2026-07-24  
**Issue:** #1553  
**Branch:** `wave-loop-812`  
**Base:** `wave-loop-811` @ `a14ec54ad5989b1040ab6339ac35713099792edb` (because earlier wave PRs remain open)

## Variant executed

**A — `[443][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1553.
2. Created and pushed branch `wave-loop-812` from `wave-loop-811` HEAD `a14ec54ad5989b1040ab6339ac35713099792edb`.
3. Copied `scripts/gen_w811.py` to `scripts/gen_w812.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w812_bench_module_443x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w812_bench_module_443x2p6_aos_var_call_write`
   - `OUTER = 443`, `MID_IDX = 221`.
4. Generated witness `specs/scratch/w812_bench_module_443x2p6_aos_var_call_write.t27`:
   - 28,352 elements
   - 907,264-bit packed vector (~0.865 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w812_bench_module_443x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w812_bench_module_443x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
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
| `cargo test -p t27c --test icarus_lowerable` | 272/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W812 | PASS |
| `t27c icarus-lowerable` W812 | PASS |
| `t27c icarus-simulate` W812 | PASS (17 cycles) |
| `t27c icarus-cocotb` W812 | PASS |
| `t27c seal --save` W812 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W812 = 40 waves.
- Witness element count: 28,352.
- Packed vector width: 907,264 bits (~0.865 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.
- FROZEN_HASH changes: 0.

## Learnings

- The generator copy hazard still spans two text locations (destination path and module header f-string). Fixing both before the first run remains the only manual failure mode. The stale `MID_IDX` comment was also corrected to `221` for W812.
- `make_grid(32768)` period-identity check continues to catch offset wrap-around congruent to 0 modulo 32768.
- `assert_ne` is not emitted by the Icarus simulation path, so bench sites verify changed element values with `assert_eq` instead.
- The packed-vector AoS lowering remains robust without compiler changes up to at least `[443][2]^6 Pt`.

## Cooperation variants for W813

- **A (recommended):** `[445][2]^6 Pt`, outer += 2, MID_IDX = 222.
- **B:** `[443][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[443][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Artifacts

- `scripts/gen_w812.py`
- `specs/scratch/w812_bench_module_443x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` (new test)
- `.trinity/seals/scratch_w812_bench_module_443x2p6_aos_var_call_write.json`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W812_2026-07-24.md`
- `.claude/plans/wave-loop-813.md`

## Next step

Create issue #1555, branch `wave-loop-813` from `wave-loop-812` HEAD, and execute the recommended variant A.
