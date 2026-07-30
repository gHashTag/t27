# FPGA Loop Closeout — Wave Loop 821

**Date:** 2026-07-30  
**Issue:** #1570  
**PR:** #1571  
**Branch:** `wave-loop-821`  
**Base:** `wave-loop-820` @ `df3d75f91` (because earlier waves' PRs remain open)

## Variant executed

**A — `[461][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1570.
2. Created and pushed branch `wave-loop-821` from `wave-loop-820` HEAD `df3d75f91`.
3. Copied `scripts/gen_w820.py` to `scripts/gen_w821.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w821_bench_module_461x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w821_bench_module_461x2p6_aos_var_call_write`
   - `OUTER = 461`, `MID_IDX = 230`.
4. Generated witness `specs/scratch/w821_bench_module_461x2p6_aos_var_call_write.t27`:
   - 29,504 elements
   - 944,128-bit packed vector (~0.900 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w821_bench_module_461x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w821_bench_module_461x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
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
| `cargo test -p t27c --test icarus_lowerable` | 281/0 |
| `cargo test -p t27c --test verilog_const_array` | 2/0 |
| `t27c parse` W821 | PASS |
| `t27c icarus-lowerable` W821 | PASS |
| `t27c icarus-simulate` W821 | PASS (17 cycles) |
| `t27c icarus-cocotb` W821 | PASS |
| `t27c seal --save` W821 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W821 = 48 waves.
- Witness element count: 29,504.
- Packed vector width: 944,128 bits (~0.900 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.

## Remaining weak points

- Pre-existing `verilog_array_literal_expr` regression remains out of scope for the witness ladder.
- FPGA E2E CI still red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 780 clippy / 780 release warnings still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged.
- 30-day commit traceability remains low; continue putting issue references in commit subjects.
- Generator copy hazard persists across W782–W821. Parameterize `WAVE`/`OUTER` in the generator template to eliminate it.

## Cooperation variants for Wave Loop 822

- **A (recommended):** `[463][2]^6 Pt`, outer += 2, `MID_IDX = 231`.
- **B:** `[461][3]^6 Pt` — grow the second inner dimension to stress stride scaling (intentionally crosses the 4-MiBit cliff; use only if explicitly desired).
- **C:** `[461][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
