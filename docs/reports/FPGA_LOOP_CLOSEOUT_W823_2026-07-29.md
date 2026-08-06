# FPGA Loop Closeout — Wave Loop 823

**Date:** 2026-07-29  
**Issue:** #1585  
**PR:** #1586  
**Branch:** `wave-loop-823`  
**Base:** `wave-loop-822` @ `fd1ef6dbe` (because earlier waves' PRs remain open)

## Variant executed

**A — `[465][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1585.
2. Created and pushed branch `wave-loop-823` from `wave-loop-822` HEAD `fd1ef6dbe`.
3. Copied `scripts/gen_w822.py` to `scripts/gen_w823.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w823_bench_module_465x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w823_bench_module_465x2p6_aos_var_call_write`
   - `OUTER = 465`, `MID_IDX = 232`.
4. Generated witness `specs/scratch/w823_bench_module_465x2p6_aos_var_call_write.t27`:
   - 29,760 elements
   - 952,320-bit packed vector (~0.908 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w823_bench_module_465x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w823_bench_module_465x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. Confirmed `bootstrap/stage0/FROZEN_HASH` is unchanged.
8. Performed weak-point audit and refreshed 2025–2026 ternary/MVL literature scan; no new actionable items found.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test --release --test icarus_lowerable accepts_w823_bench_module_465x2p6_aos_var_call_write` | 1/0 |
| `cargo test --release --test icarus_lowerable` (full suite) | 283/0 |
| `t27c parse` W823 | PASS |
| `t27c icarus-lowerable` W823 | PASS |
| `t27c icarus-simulate` W823 | PASS (17 cycles) |
| `t27c icarus-cocotb` W823 | PASS |
| `t27c seal --save` W823 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W823 = 50 waves.
- Witness element count: 29,760.
- Packed vector width: 952,320 bits (~0.908 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.

## Remaining weak points

- Pre-existing `verilog_array_literal_expr` regression remains out of scope for the witness ladder.
- FPGA E2E CI still red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged.
- 30-day commit traceability remains low; continue putting issue references in commit subjects.
- Generator copy hazard persists across W782–W823. Parameterize `WAVE`/`OUTER` in the generator template to eliminate it.

## Cooperation variants for Wave Loop 824

- **A (recommended):** `[467][2]^6 Pt`, outer += 2, `MID_IDX = 233`.
- **B:** `[465][3]^6 Pt` — grow the second inner dimension to stress stride scaling (intentionally crosses the 4-MiBit cliff; use only if explicitly desired).
- **C:** `[465][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
