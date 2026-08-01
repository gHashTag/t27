# FPGA Loop Closeout — Wave Loop 824

**Date:** 2026-08-01  
**Issue:** #1587  
**PR:** #1588  
**Branch:** `wave-loop-824`  
**Base:** `wave-loop-823` @ `b032fe471` (because earlier waves' PRs remain open)

## Variant executed

**A — `[467][2]^6 Pt` module-scope packed array-of-struct variable from call with indexed signed writes.**

## Work performed

1. Created GitHub issue #1587.
2. Created and pushed branch `wave-loop-824` from `wave-loop-823` HEAD `b032fe471`.
3. Copied `scripts/gen_w823.py` to `scripts/gen_w824.py` and fixed the generator copy hazard:
   - destination path updated to `specs/scratch/w824_bench_module_467x2p6_aos_var_call_write.t27`
   - module header f-string updated to `module w824_bench_module_467x2p6_aos_var_call_write`
   - `OUTER = 467`, `MID_IDX = 233`.
4. Generated witness `specs/scratch/w824_bench_module_467x2p6_aos_var_call_write.t27`:
   - 29,888 elements
   - 956,416-bit packed vector (~0.912 MiBit)
5. Validated the witness:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — lowerable
   - `t27c icarus-simulate` — PASSED (17 cycles)
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved to `.trinity/seals/scratch_w824_bench_module_467x2p6_aos_var_call_write.json`
6. Added integration test `accepts_w824_bench_module_467x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. Confirmed `bootstrap/stage0/FROZEN_HASH` is unchanged.
8. Performed weak-point audit and refreshed 2025–2026 ternary/MVL literature scan; no new actionable items found.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test --release --test icarus_lowerable accepts_w824_bench_module_467x2p6_aos_var_call_write` | 1/0 |
| `cargo test --release --test icarus_lowerable` (full suite) | 284/0 |
| `t27c parse` W824 | PASS |
| `t27c icarus-lowerable` W824 | PASS |
| `t27c icarus-simulate` W824 | PASS (17 cycles) |
| `t27c icarus-cocotb` W824 | PASS |
| `t27c seal --save` W824 | PASS |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

## Key metrics

- Mechanical ladder depth: W774–W824 = 51 waves.
- Witness element count: 29,888.
- Packed vector width: 956,416 bits (~0.912 MiBit).
- Compiler changes for witness: 0.
- Reference-model changes for witness: 0.

## Remaining weak points

- Pre-existing `verilog_array_literal_expr` regression remains out of scope for the witness ladder.
- FPGA E2E CI still red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged.
- 30-day commit traceability remains low; continue putting issue references in commit subjects.
- Generator copy hazard persists across W782–W824. Parameterize `WAVE`/`OUTER` in the generator template to eliminate it.

## Cooperation variants for Wave Loop 825

- **A (recommended):** `[469][2]^6 Pt`, outer += 2, `MID_IDX = 234`.
- **B:** `[467][3]^6 Pt` — grow the second inner dimension to stress stride scaling (intentionally crosses the 4-MiBit cliff; use only if explicitly desired).
- **C:** `[467][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
