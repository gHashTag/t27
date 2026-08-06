# Wave Loop 815 Plan — `[449][2]^6 Pt`

**Date:** 2026-07-29  
**Branch base:** `wave-loop-814` (parent branch because earlier Wave Loop PRs remain open awaiting review)

## Recommended variant A

Extend the mechanical module-scope packed-array-of-struct ladder by one rung:

- Generator `scripts/gen_w815.py` from `scripts/gen_w814.py`.
- `OUTER = 449` (outer dimension +2), `MID_IDX = 449 // 2 = 224`.
- Destination path and module header f-string both fixed from `w814`/`447` to `w815`/`449` before first run.
- Generated witness `specs/scratch/w815_bench_module_449x2p6_aos_var_call_write.t27`.
- 28,736 elements × 32 bits = 919,552 bits (~0.877 MiBit).

## Variants for W815

### A — `[449][2]^6 Pt` (recommended, mechanical ladder)

Smallest reviewable diff; continues validation that t27c can lower ever-wider non-power-of-two packed vectors unchanged.

### B — `[447][3]^6 Pt` stride scaling

Keep outer dimension at the previous rung but grow the second inner dimension from `2` to `3`. This changes the element stride and tests whether the packed-vector layout formula generalizes beyond powers of two in inner dimensions. Larger (447 × 3 × 2^5 = 42,912 elements × 32 bits = 1.373 MiBit) and may need formula adjustments.

### C — `[447][2]^6 Pt` negative-index wrap-around

Add signed negative-index writes to the current witness (e.g. `dst[-1][...].x = 42`) to exercise wrap-around/undefined-behavior classification. Requires confirming the compiler either lowers a defined wrap or emits a clean diagnostic; do not silently pass broken code.

## Procedure

1. Create issue #1559 for Wave Loop 815.
2. Create branch `wave-loop-815` from `wave-loop-814` HEAD.
3. Copy generator, fix copy hazard, generate spec.
4. Run `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save`.
5. Add integration test to `bootstrap/tests/icarus_lowerable.rs`.
6. Run validation matrix:
   - `cargo build --release -p t27c`
   - `cargo clippy -p t27c` (expect 780 warnings, 0 errors)
   - `cargo test -p t27c --bin t27c` (1494/0/2)
   - `cargo test -p tri` (78/0)
   - `cargo test -p flash-spi` (2/0)
   - `cargo test -p t27c --test bitnet_pipeline` (20/0)
   - `cargo test -p t27c --test bitnet_top` (17/0)
   - `cargo test -p t27c --test icarus_lowerable` (275/0)
   - `cargo test -p t27c --test verilog_const_array` (2/0)
7. Refresh weak-point audit and literature scan.
8. Write closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W815_2026-07-29.md`.
9. Update live skill tracker, NOW.md, experience.md, persistent memory.
10. Commit `Closes #1559`, push, open PR.
