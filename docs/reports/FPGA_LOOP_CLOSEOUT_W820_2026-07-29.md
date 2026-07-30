# FPGA Loop Closeout — Wave Loop 820

**Date:** 2026-07-29  
**Issue:** #1568  
**PR:** #1569  
**Branch:** `wave-loop-820`  
**Base:** `wave-loop-819` @ `811740999` (because earlier wave PRs remain open)

## Variant executed

**A — `[459][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1568.
2. Created and pushed branch `wave-loop-820` from `wave-loop-819` HEAD `811740999`.
3. Copied `scripts/gen_w819.py` to `scripts/gen_w820.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w820_bench_module_459x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w820_bench_module_459x2p6_aos_var_call_write`
   - `OUTER = 459`, `MID_IDX = 229`.
4. Generated witness `specs/scratch/w820_bench_module_459x2p6_aos_var_call_write.t27`:
   - 29,376 elements
   - 940,032-bit packed vector (~0.897 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w820_bench_module_459x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w820_bench_module_459x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. Confirmed `bootstrap/stage0/FROZEN_HASH` is unchanged.
8. Performed weak-point audit and refreshed 2025–2026 ternary/MVL literature scan; no new actionable items found.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo clippy -p t27c` | green (626 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed, 0 failed, 2 ignored |
| `cargo test -p tri` | 78/0 |
| `cargo test -p flash-spi` | 2/0 |
| `cargo test -p t27c --test bitnet_pipeline` | 20/0 |
| `cargo test -p t27c --test bitnet_top` | 17/0 |
| `cargo test -p t27c --test icarus_lowerable` | 280/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W820 | PASS |
| `t27c icarus-lowerable` W820 | PASS |
| `t27c icarus-simulate` W820 | PASS (17 cycles) |
| `t27c icarus-cocotb` W820 | PASS |
| `t27c seal --save` W820 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W820 = 47 waves.
- Witness element count: 29,376.
- Packed vector width: 940,032 bits (~0.897 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.
- FROZEN_HASH changes: 0.

## Learnings

- The generator copy hazard still spans two text locations (destination path and module header f-string). Fixing both before the first run remains the only manual failure mode. The stale `MID_IDX` comment was also corrected to `229` for W820.
- For `OUTER = 459`, `MID_IDX = 229`; the frame-condition element is `[229][1][0][0][0][0][0]`, element number `229 * 64 + 32 = 14,688`.
- `make_grid(32768)` period-identity check continues to catch offset wrap-around congruent to 0 modulo 32768.
- `assert_ne` is not emitted by the Icarus simulation path, so bench sites verify changed element values with `assert_eq` instead.
- The packed-vector AoS lowering remains robust without compiler changes up to at least `[459][2]^6 Pt`.

## Cooperation variants for W821

- **A (recommended):** `[461][2]^6 Pt`, outer += 2, MID_IDX = 230.
- **B:** `[459][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[459][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Artifacts

- `scripts/gen_w820.py`
- `specs/scratch/w820_bench_module_459x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` (new test)
- `.trinity/seals/scratch_w820_bench_module_459x2p6_aos_var_call_write.json`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W820_2026-07-29.md`
- `.claude/plans/wave-loop-821.md`
- Updated `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`, `.claude/skills/wave-loop-autopilot.md`
- Persistent memory `wave-loop-820.md`
