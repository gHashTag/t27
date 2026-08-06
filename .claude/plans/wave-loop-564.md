# Wave Loop 564 Plan — Variant A

**Issue #1535** — whole-array comparison for 1-D arrays of scalar structs.

## Goal

Close the whole-array `assert_eq` gap for packed 1-D arrays of lowerable scalar
structs. After W555/W562 the probe path already handles primitive scalar arrays
and scalar structs; W563 added the packed 1-D AoS local/CSE path. This wave
extends `expr_width_signed`, `gen_verilog_expr` for `ExprArrayLiteral`, and the
Python cocotb reference model so a bench block can compare a whole 1-D AoS value
(local or call result) against an array literal.

## Steps

1. **OBSERVE** — confirm current baseline and identify exact code points.
   - `bootstrap/src/compiler.rs` `expr_width_signed` branches for
     `ExprIdentifier`, `ExprCall`, `ExprArrayLiteral`.
   - `gen_verilog_expr` `ExprArrayLiteral` packed-concat emission.
   - `scripts/cocotb_ref_model.py` `_packed_type_width_signed` and
     `_type_of_expr`.

2. **IMPLEMENT** — minimal compiler + model changes.
   - Treat lowerable scalar-struct arrays as packed vectors in
     `expr_width_signed` (same helpers as W555 primitive arrays: `packed_width`,
     `packed_signed`).
   - Allow `ExprArrayLiteral` of lowerable scalar structs to lower to a packed
     concatenation via `emit_packed_array_literal_concat`.
   - Fix `_packed_type_width_signed` and `_type_of_expr` in Python so the
     reference model computes the correct total packed width for `[N]Pt`.

3. **WITNESS / TEST**
   - `specs/scratch/w564_bench_whole_aos_1d.t27` with a `bench` block containing
     `assert_eq(make_pts(...), [2]Pt{...})` and `assert_eq(tmp, [2]Pt{...})`.
   - Add `accepts_w564_bench_whole_aos_1d` to
     `bootstrap/tests/icarus_lowerable.rs`.

4. **VERIFY**
   - `cd bootstrap && cargo build --release -p t27c`.
   - `cargo test -p t27c --bin t27c`.
   - `cargo test -p t27c --test icarus_lowerable`.
   - `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`.
   - Reseal any affected corpus specs and update `bootstrap/stage0/FROZEN_HASH`.
   - `lake build Trinity.IcarusLowerable.Soundness`.

5. **SYNTHESIZE / LEARN**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W564_2026-07-07.md`.
   - Update `.trinity/current-issue.md` with three W565 cooperation variants.
   - Update `.trinity/experience.md` and persistent memory.
   - Commit with `Closes #1535`.
