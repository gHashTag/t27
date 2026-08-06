# Wave Loop 540 Closeout — Multi-signal VCD probes for wide packed structs and arrays

**Issue:** #1511  
**Branch:** `wave-loop-540`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was delivered

Wave Loop 539 added typed 64-bit VCD probes for every `assert_eq` actual expression and a
full bit-vector expression evaluator in `scripts/cocotb_ref_model.py`.  Wave Loop 540
tackled the obvious next boundary: packed t27 values whose bit width exceeds 64 bits
(wide packed scalar structs, including those with fixed-size scalar array fields).  Such
values are now captured as multiple 64-bit (or final partial) VCD slices and reconstructed
independently by the Python reference model.

### Compiler (`bootstrap/src/compiler.rs`)

- Extended `expr_width_signed` so it can size `assert_eq` actual expressions whose root
  is:
  - a function call returning a lowerable packed scalar struct, or
  - a literal of a lowerable packed scalar struct (`ExprStructLit`).
  The packed width is computed from the struct's fields, including fixed-size scalar
  arrays.
- `gen_verilog_test` now, for each wide assertion, pre-declares:
  - a packed temporary register `_t27_probe_tmp_<block>_<idx>` sized to the full width, and
  - one register per slice `_t27_probe_<block>_<idx>_s<N>` sized to its slice width.
  All declarations precede procedural statements, keeping the generated initial block
  acceptable to Icarus Verilog.
- `gen_verilog_test_stmt` was rewritten to derive the base probe name from the block name
  and probe index, collect every matching slice spec, assign the temporary register from
  the actual expression, then assign each slice by part-select.  Narrow assertions keep
  the original single-probe path.
- Updated `bootstrap/stage0/FROZEN_HASH` after every compiler surface change.

### Reference model (`scripts/cocotb_ref_model.py`)

- Added `u128` / `i128` to the type-width table so wide scalar literals are not mis-sized.
- Added `_eval_struct_lit_bv` and `_eval_array_lit_bv` so whole packed-struct/array
  literals can be evaluated as bit-vectors (field 0 / array element 0 at the LSB, matching
  the Verilog packed layout).
- Added `_VcdParser.probe_slices` to discover the `..._s0`, `..._s1`, ... probe names
  emitted for wide values and return `(value, width, offset)` tuples sorted by offset.
- `_collect_assertions` now infers the actual expression's declared width and signedness
  via `_type_of_expr` and, when necessary, re-wraps a simple literal expected value at the
  actual width so the VCD comparison compares the correct number of bits.
- `_cross_check` reconstructs the full packed value by OR-ing shifted slices, masks to the
  full width, applies the declared signedness, and compares against the expected value.

### Witness and seals

- Added `specs/scratch/w540_wide_packed_struct_array.t27`: a lowerable packed scalar
  struct with a `[5]u16` field (80-bit total width).  The test asserts equality between
  a function return and a matching struct literal, forcing the multi-slice probe path.
- Sealed the witness: `.trinity/seals/scratch_w540_wide_packed_struct_array.json`.
- Recorded the Icarus baseline:
  `.trinity/icarus-baselines/specs/scratch/w540_wide_packed_struct_array.json`.

---

## Validation

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed, 0 failed, 2 ignored |
| `cargo test -p tri` | 78 passed, 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed, 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 36 Icarus PASS, 0 FAIL; 36 cocotb PASS, 0 FAIL; 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` (in `proofs/lean4`) | 8572 jobs, 0 `sorry` |

The 24 pre-existing yosys smoke baseline failures are unchanged and documented.

---

## Known residual boundaries / weak points

- The Python reference model still evaluates only a subset of expressions.  In particular,
  module-level constants and variables are not bound into `EvalContext`, so some wide
  assertions would fall back to log-only verification.  The new witness deliberately uses
  a function-call actual expression that the evaluator already handles.
- Wide probe reconstruction assumes slice offsets are multiples of 64 bits, which matches
  the current compiler emission strategy.  A future change to variable slice sizes would
  need to encode offsets explicitly in the probe names or metadata.
- `gen_verilog_test_stmt` emits one `$display("[PROBE] ...")` per slice; this is verbose
  but correct and matches the existing narrow-probe log format.

---

## Next step

See `docs/reports/FPGA_LOOP_COOPERATION_W541_2026-07-07.md` for three concrete cooperation
variants and a recommended path for Wave Loop 541.

---

*φ² + φ⁻² = 3 | TRINITY*
