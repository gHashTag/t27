# Wave Loop 586 — Decomposed Plan

**Issue:** #1557
**Branch:** `wave-loop-586`
**Date:** 2026-07-07
**Chosen variant:** **C** (recommended) — module-scope 8-D array-of-struct variable with indexed signed field writes.

## 1. Weak spots addressed

1. **Module-scope mutable packed-array state.**
   W585 validated read-only reuse of a module-scope `var` initialized from a call.
   W586 now exercises **writes** to individual elements/fields of that packed
   register. This stresses the generated procedural assignment path for packed
   slices at module scope.

2. **Signed packed-slice interpretation.**
   A negative `i16` literal written to a packed slice is stored as the correct
   two's-complement bits, but the read/compare path must cast the unsigned
   part-select to signed (`$signed(...)`) or the assertion fails.

3. **Multi-dimensional index resolution for probe metadata.**
   `expr_width_signed` and `field_scalar_array_info` had to be taught to walk
   nested `ExprIndex` chains so that probe width/signed metadata for
   `dst[i][j][...].y` is accurate.

## 2. Implementation summary

- `specs/scratch/w586_bench_module_8d_aos_var_write.t27`
  - `pub struct Pt { x : i16, y : i16 }`.
  - `pub var dst : [2][2][2][2][2][2][2][2]Pt = [2]^8 Pt{}`.
  - `bench module_bench_8d_var_write` writes four indexed signed fields and
    asserts the updated values.
- `bootstrap/src/compiler.rs`
  - Walk nested `ExprIndex` chains in `expr_width_signed` / `field_scalar_array_info`.
  - Wrap signed packed slices with `$signed(...)` in reads.
  - Add `in_lvalue` flag to suppress the signed wrapper on assignment targets.
- `bootstrap/stage0/FROZEN_HASH` updated.
- `bootstrap/tests/icarus_lowerable.rs`: new integration test.
- 30 affected seals resealed.

## 3. Verification gates

- [x] `cargo build --release -p t27c`
- [x] `cargo test -p t27c --bin t27c`
- [x] `cargo test -p tri`
- [x] `cargo test -p t27c --test icarus_lowerable`
- [x] `./scripts/tri test --fast` (0 seal mismatches)
- [x] Direct `t27c icarus-simulate` W586
- [x] Direct `t27c icarus-cocotb` W586
- [x] `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`

## 4. Closeout artifacts

- `docs/reports/FPGA_LOOP_CLOSEOUT_W586_2026-07-07.md`
- `.trinity/experience.md` updated with W586 learnings.
- `.trinity/current-issue.md` updated with three W587 cooperation variants.
