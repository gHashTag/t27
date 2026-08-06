# FPGA Loop Closeout — Wave Loop 822

**Date:** 2026-07-30  
**Issue:** #1572  
**PR:** #1573  
**Branch:** `wave-loop-822`  
**Base:** `wave-loop-821` @ `b9ae742e7` (because earlier waves' PRs remain open)

## Variant executed

**A — `[463][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1572.
2. Created and pushed branch `wave-loop-822` from `wave-loop-821` HEAD `b9ae742e7`.
3. Copied `scripts/gen_w821.py` to `scripts/gen_w822.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w822_bench_module_463x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w822_bench_module_463x2p6_aos_var_call_write`
   - `OUTER = 463`, `MID_IDX = 231`.
4. Generated witness `specs/scratch/w822_bench_module_463x2p6_aos_var_call_write.t27`:
   - 29,632 elements
   - 948,224-bit packed vector (~0.904 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w822_bench_module_463x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w822_bench_module_463x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
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
| `cargo test -p t27c --test icarus_lowerable` | 282/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W822 | PASS |
| `t27c icarus-lowerable` W822 | PASS |
| `t27c icarus-simulate` W822 | PASS (17 cycles) |
| `t27c icarus-cocotb` W822 | PASS |
| `t27c seal --save` W822 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W822 = 49 waves.
- Witness element count: 29,632.
- Packed vector width: 948,224 bits (~0.904 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.

## Remaining weak points

- Pre-existing `verilog_array_literal_expr` regression remains out of scope for the witness ladder.
- FPGA E2E CI still red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 780 clippy / 780 release warnings still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged.
- 30-day commit traceability remains low; continue putting issue references in commit subjects.
- Generator copy hazard persists across W782–W822. Parameterize `WAVE`/`OUTER` in the generator template to eliminate it.

## Cooperation variants for Wave Loop 823

- **A (recommended):** `[465][2]^6 Pt`, outer += 2, `MID_IDX = 232`.
- **B:** `[463][3]^6 Pt` — grow the second inner dimension to stress stride scaling (intentionally crosses the 4-MiBit cliff; use only if explicitly desired).
- **C:** `[463][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
