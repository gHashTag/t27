# Wave Loop 808 Plan — `[435][2]^6 Pt`

**Date:** 2026-07-24  
**Branch base:** `wave-loop-807` (parent branch because earlier Wave Loop PRs remain open awaiting review)

## Recommended variant A

Extend the mechanical module-scope packed-array-of-struct ladder by one rung:

- Generator `scripts/gen_w808.py` from `scripts/gen_w807.py`.
- `OUTER = 435` (outer dimension +2), `MID_IDX = 435 // 2 = 217`.
- Destination path and module header f-string both fixed from `w807`/`433` to `w808`/`435` before first run.
- Generated witness `specs/scratch/w808_bench_module_435x2p6_aos_var_call_write.t27`.
- 27,840 elements × 32 bits = 890,880 bits (~0.849 MiBit).

## Variants for W808

### A — `[435][2]^6 Pt` (recommended, mechanical ladder)

Smallest reviewable diff; continues validation that t27c can lower ever-wider non-power-of-two packed vectors unchanged.

### B — `[433][3]^6 Pt` stride scaling

Keep outer dimension at the previous rung but grow the second inner dimension from `2` to `3`. This changes the element stride and tests whether the packed-vector layout formula generalizes beyond powers of two in inner dimensions. Larger (433 × 3 × 2^5 = 41,568 elements × 32 bits = 1.330 MiBit) and may need formula adjustments.

### C — `[433][2]^6 Pt` negative-index wrap-around

Add signed negative-index writes to the current witness (e.g. `dst[-1][...].x = 42`) to exercise wrap-around/undefined-behavior classification. Requires confirming the compiler either lowers a defined wrap or emits a clean diagnostic; do not silently pass broken code.

## Procedure

1. Create issue #1545 for Wave Loop 808.
2. Create branch `wave-loop-808` from `wave-loop-807` HEAD.
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
   - `cargo test -p t27c --test icarus_lowerable` (268/0)
   - `cargo test -p t27c --test verilog_const_array` (2/0)
7. Refresh weak-point audit and literature scan.
8. Write closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W808_2026-07-24.md`.
9. Update live skill tracker, NOW.md, experience.md, persistent memory.
10. Commit `Closes #1545`, push, open PR.
