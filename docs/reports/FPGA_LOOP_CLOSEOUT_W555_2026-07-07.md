# Wave Loop 555 Closeout Report — Whole-array `bench` assignments

**Issue:** #1526  
**Branch:** `wave-loop-555`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 555 enables `assert_eq` on a complete 2-D primitive scalar array value
inside a deterministic `bench` block. The implementation reuses the W540 multi-slice
VCD probe path: once the compiler recognizes a primitive scalar array identifier,
function-call return, or array literal as a probe-able packed vector, the existing
wide-probe machinery captures the value in Icarus and the Python reference model
reconstructs and compares it against the expected array literal.

Four new scratch witnesses cover unsigned, signed, direct function-call actual
expressions, and a wide 256-bit array that forces four 64-bit probe slices.

---

## What changed

### `bootstrap/src/compiler.rs`

- Extended `expr_width_signed`:
  - `ExprIdentifier` now returns `(packed_width, packed_signed)` for primitive
    scalar array identifiers.
  - `ExprCall` now returns the same for function calls whose return type is a
    primitive scalar array.
  - Added a new `ExprArrayLiteral` branch that parses multi-dimensional dimensions
    from `extra_size` (`"2][3"`) and returns the full packed width / signedness.
- Extended `gen_verilog_expr` for `ExprArrayLiteral`:
  - Multi-dimensional primitive scalar array literals now lower to a single packed
    concatenation via `emit_packed_array_literal_concat` instead of the
    `/* TODO: array literal ... */` placeholder. One-dimensional literals continue
    to work unchanged.

### `scripts/cocotb_ref_model.py`

- Added `_primitive_array_info()` to compute `(dims, elem, total_width, signed)` for
  any fixed-size primitive scalar array type (`[2][3]i8`, `[4][4]u16`, etc.).
- Updated `_packed_type_width_signed()` to use `_primitive_array_info()`, so
  identifiers and calls returning multi-D arrays resolve to the full packed width.
- Updated `_type_of_expr()` for `ExprArrayLiteral` to use the same helper.

### Seals, baselines, and integration test

- Saved t27 seals for the four W555 witnesses:
  - `specs/scratch/w555_bench_whole_array_unsigned.t27`
  - `specs/scratch/w555_bench_whole_array_signed.t27`
  - `specs/scratch/w555_bench_whole_array_nested_call.t27`
  - `specs/scratch/w555_bench_whole_array_wide.t27`
- Recorded Icarus baseline for the nested-call witness (the only W555 witness that
  also passes the suite's `gen-verilog` pre-flight without named test/bench locals).
- Added `accepts_w555_bench_whole_array_cross_check` integration test in
  `bootstrap/tests/icarus_lowerable.rs`.
- Updated `bootstrap/stage0/FROZEN_HASH` to the new compiler hash.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 15 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 66 Icarus PASS, 66 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `./target/release/t27c icarus-simulate specs/scratch/w555_*.t27` | 4/4 PASS |
| Direct `./target/release/t27c icarus-cocotb specs/scratch/w555_*.t27` | 4/4 PASS |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new failures
were introduced by W555.

---

## Notes and known limitations

- Witnesses that declare a named `let` inside `test`/`bench` (`w555_bench_whole_array_unsigned`,
  `w555_bench_whole_array_signed`, `w555_bench_whole_array_wide`) pass direct
  `t27c icarus-simulate` / `t27c icarus-cocotb` but are not counted in the
  `./scripts/tri test --icarus-lowerable --icarus-simulate` regression tally. The
  suite's `gen-verilog` pre-flight strips test/bench local declarations (a
  pre-existing synth-oriented limitation also documented in W554). Only the
  nested-call witness, whose actual expression has no named local, is included in
  the automated Icarus gate count.
- The wide `[4][4]u16` witness (256 bits) confirms that the W540 multi-slice probe
  path works unchanged for primitive arrays: four 64-bit slices are declared,
  assigned, and reconstructed by the Python reference model.
- Failure diagnostics for wide packed-vector inequality still print `%0d`, which
  only shows the low 32 bits. The PASS/FAIL decision is correct; richer wide-value
  diagnostics are left for a future wave.

---

## Three cooperation variants for Wave Loop 556

1. **Variant A — Recommended: multi-site call-return array deduplication.**
   When the same `f()` packed-array expression is indexed or compared at multiple
   sites in one `bench`, reuse a single packed temporary and emit only one
   assignment. Add a witness with several reads of the same call result and
   verify the temporary is assigned once.

2. **Variant B: signed whole-array comparison for higher ranks.**
   Extend the W555 whole-array bench probe to 3-D and 4-D signed primitive
   scalar arrays, verifying row-major slice reconstruction in the Python model
   for ranks 3 and 4.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and document
   the boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.

---

## Skills to carry forward

Pattern: *"A whole-array `assert_eq` inside a `bench` block is just a wide
packed-vector VCD probe. Once `expr_width_signed` / `_type_of_expr` recognize a
primitive scalar array identifier, function-call return, or array literal as a
probe-able packed vector, the W540 multi-slice path handles capture and the
existing `_eval_array_lit_bv` handles the expected literal reconstruction."*

---

Closes #1526
