# FPGA Loop Closeout — Wave Loop 826

**Date:** 2026-08-01  
**Issue:** #1593  
**PR:** #1594  
**Branch:** `wave-loop-826`  
**Base:** `wave-loop-825` @ `9eef0ea8a` (because earlier waves' PRs remain open)

## Variant executed

**A — `[471][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1593.
2. Created and pushed branch `wave-loop-826` from `wave-loop-825` HEAD `9eef0ea8a`.
3. Copied `scripts/gen_w825.py` to `scripts/gen_w826.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w826_bench_module_471x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w826_bench_module_471x2p6_aos_var_call_write`
   - `OUTER = 471`, `MID_IDX = 235`.
4. Generated witness `specs/scratch/w826_bench_module_471x2p6_aos_var_call_write.t27`:
   - 30,144 elements
   - 964,608-bit packed vector (~0.920 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w826_bench_module_471x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w826_bench_module_471x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. Confirmed `bootstrap/stage0/FROZEN_HASH` is unchanged.
8. Performed weak-point audit and refreshed 2025–2026 ternary/MVL literature scan; no new actionable items found.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test --release --test icarus_lowerable accepts_w826_bench_module_471x2p6_aos_var_call_write` | 1/0 |
| `cargo test --release --test icarus_lowerable` (full suite) | 286/0 |
| `t27c parse` W826 | PASS |
| `t27c icarus-lowerable` W826 | PASS |
| `t27c icarus-simulate` W826 | PASS (17 cycles) |
| `t27c icarus-cocotb` W826 | PASS |
| `t27c seal --save` W826 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W826 = 53 waves.
- Witness element count: 30,144.
- Packed vector width: 964,608 bits (~0.920 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.

## Remaining weak points

- Pre-existing `verilog_array_literal_expr` regression remains out of scope for the witness ladder.
- FPGA E2E CI still red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged.
- 30-day commit traceability remains low; continue putting issue references in commit subjects.
- Generator copy hazard persists across W782–W826. Parameterize `WAVE`/`OUTER` in the generator template to eliminate it.

## Cooperation variants for Wave Loop 827

- **A (recommended):** `[473][2]^6 Pt`, outer += 2, `MID_IDX = 236`.
- **B:** `[471][3]^6 Pt` — grow the second inner dimension to stress stride scaling (intentionally crosses the 4-MiBit cliff; use only if explicitly desired).
- **C:** `[471][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
