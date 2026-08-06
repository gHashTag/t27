# FPGA Loop Closeout — Wave Loop 830 (2026-08-01)

| Field | Value |
|-------|-------|
| Wave | 830 |
| Issue | #1601 |
| Branch | `wave-loop-830` |
| Parent branch | `wave-loop-829` HEAD (`f0ebad1`) — earlier wave PRs remain open |
| PR | #1602 |
| Variant | A — module-scope `[479][2]^6 Pt` packed array-of-struct variable from call with indexed signed writes |
| Elements | 30,656 |
| Packed vector | 30,656 × 32 bits = 980,992 bits (~0.935 MiBit) |
| Status | Closed, branch pushed, PR open |

## Summary

Wave Loop 830 continued the non-power-of-two outer-dimension module-scope packed
array-of-struct ladder with `[479][2]^6 Pt`. The witness required **zero
compiler, reference-model, or `FROZEN_HASH` changes**. The only manual failure
mode was the recurring generator copy hazard, which was fixed before the first
generation.

## What landed

- `scripts/gen_w830.py`
  - Copied from `scripts/gen_w829.py` and fixed for copy hazard.
  - `OUTER = 479`, `MID_IDX = 239`.
  - Destination path and module header f-string updated to `w830_bench_module_479x2p6_aos_var_call_write`.
  - Stale `MID_IDX` comment corrected to `239`.
- `specs/scratch/w830_bench_module_479x2p6_aos_var_call_write.t27`
  - Generated witness: 30,656 elements, 980,992-bit packed vector (~0.935 MiBit).
- `.trinity/seals/scratch_w830_bench_module_479x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w830_bench_module_479x2p6_aos_var_call_write`.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W830_2026-08-01.md`
  - This closeout report.
- `.claude/plans/wave-loop-831.md`
  - Cooperation plan for the next wave with variants A/B/C.

## Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` W830 | PASS |
| `t27c icarus-lowerable` W830 | `lowerable` |
| `t27c icarus-simulate` W830 | `PASSED` (17 cycles) |
| `t27c icarus-cocotb` W830 | `reference-model OK` |
| `t27c seal --save` W830 | PASS |
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test --release --test icarus_lowerable accepts_w830_bench_module_479x2p6_aos_var_call_write` | 1/0 |
| `cargo test --release --test icarus_lowerable` (full suite) | **290/0** |
| `FROZEN_HASH` | unchanged |

## Weak points (unchanged)

- `bootstrap/tests/verilog_array_literal_expr.rs` regression remains pre-existing
  and is tracked for a dedicated ring.
- FPGA E2E CI remains red (`sby` missing + Yosys static-cast error in generated
  `uart.v`).
- ~626 release warnings / ~780 clippy warnings still need a cleanup sprint.
- Vivado-in-Docker CI gap remains (private image not yet published).
- 30-day traceability by commit subject remains below target; continue putting
  `Closes #N` in commit subjects.
- Generator copy hazard persists across three text locations (`DST`, module
  header f-string, `MID_IDX` comment). Parameterizing the wave prefix and outer
  dimension in a single config block remains the recommended tooling fix.

## Next-wave cooperation variants (Wave Loop 831)

See `.claude/plans/wave-loop-831.md` for full details.

- **A (recommended):** `[481][2]^6 Pt`, outer += 2, `MID_IDX = 240`.
- **B:** `[479][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[479][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

*φ² + φ⁻² = 3 | TRINITY*
