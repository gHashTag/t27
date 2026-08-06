# Wave Loop 562 Closeout Report — Whole-struct comparison for structs with array-typed fields

**Issue:** #1533  
**Branch:** `wave-loop-562`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 562 implements **Variant B** from Wave Loop 562: extend the packed
scalar-struct lowering to scalar-array-typed fields, add an end-to-end bench
witness, and fix the malformed Verilog emitted when a scalar-array field is
indexed on a function-call return.

The key compiler fix is in `try_emit_struct_array_field_element_access`: when
the base of a `.data[i]` access is a function call returning a lowerable packed
scalar struct, and a block-scoped call temporary exists, the emitter now emits a
single correct dynamic part-select over the temporary instead of applying a
second index to a part-select. The generated Verilog for
`make_packet(...).data[1]` changed from the invalid
`$signed(_t27_call_tmp_...[0 +: 32])[1]` to the correct
`$signed(_t27_call_tmp_...[((0 + (1 * 8)) +: 8])`.

---

## What changed

### `.claude/plans/wave-loop-562.md`

- Decomposed plan documenting the malformed Verilog weak point, the witness
  design, the compiler fix, reference-model alignment, and three W563
  cooperation variants.

### `bootstrap/src/compiler.rs`

- Extended `try_emit_struct_array_field_element_access` to handle an
  `ExprCall` base for scalar-struct returns with scalar-array fields.
- When `use_call_array_temps` is active and a temporary exists for the call,
  the temporary name replaces the original call expression and the slice is
  emitted without parentheses.
- When the base is a bare call without a temporary, the original call
  expression is used with parentheses around the slice base.

### `bootstrap/stage0/FROZEN_HASH`

- Updated to `fedc9333f22a0590e38200410cffe7969b76f3a9fd7548ab6101b62d15a69d40`
  to match the new SHA-256 of `bootstrap/src/compiler.rs`.

### `scripts/cocotb_ref_model.py`

- Verified that `_eval_struct_lit_bv` already routes array-typed fields through
  `_eval_array_lit_bv`, which masks each element to the declared width before
  packing. W560 fixed scalar fields; the same discipline now covers
  scalar-array-typed fields end-to-end.

### Witnesses and seals

- Added `specs/scratch/w562_bench_struct_array_field.t27`:
  - `struct Packet { data: [4]i8, sum: i16 }`
  - `pub fn make_packet(...)` returning `Packet`
  - test: whole-struct `assert_eq(make_packet(...), Packet{...})`,
    element access `make_packet(...).data[1]`, scalar field access `.sum`.
  - bench: local `var tmp : Packet = make_packet(...)`, same checks on `tmp`.
- Saved t27 seal under `.trinity/seals/scratch_w562_bench_struct_array_field.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w562_bench_struct_array_field.json`.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w562_bench_struct_array_field` integration test that asserts
  the W562 witness is accepted by the structural `icarus-lowerable` classifier.

### Reseals

Ten existing corpus/scratch seals whose generated Verilog changed due to the
compiler edit were resealed:
- `compiler_Lexing.json`
- `compiler_Stdlib.json`
- `scratch_w532_negative_enum_field.json`
- `scratch_w532_negative_string_field.json`
- `scratch_w532_signed_struct_array_field_param.json`
- `scratch_w532_signed_struct_array_field_return.json`
- `scratch_w533_module_scalar_struct_const.json`
- `scratch_w533_module_scalar_struct_var_call.json`
- `scratch_w533_module_scalar_struct_var_copy.json`
- `scratch_w533_module_scalar_struct_var_literal.json`

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 22 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W562 witness | `[TEST] ... PASSED`, `[BENCH] ... PASSED` |
| Direct `t27c icarus-cocotb` on W562 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W562.

---

## Notes and known limitations

- The fix is scoped to scalar structs whose fields are fixed-size scalar
  arrays. Structs with non-scalar-array fields (`string`, `enum`, `f32`,
  nested struct, unresolved import) remain rejected by the structural
  classifier, as locked by W561 negative witnesses.
- The W562 witness exercises a single 1-D `i8` array field. Higher-rank scalar
  array fields and multi-dimensional array fields are not explicitly covered
  by this wave.
- The call-CSE optimization continues to apply only to pure, side-effect-free
  calls inside deterministic `test` / `bench` blocks.

---

## Three cooperation variants for Wave Loop 563

1. **Variant A — Recommended: array-of-struct return call deduplication.**
   Extend the W556–W558 / W560 block-scoped call temporary machinery to
   function calls that return fixed-size arrays of lowerable packed scalar
   structs (`[N]Pt`). Requires prerequisite fixes already identified in W561:
   `ExprArrayLiteral` lowering for `[N]Pt`, bench-local 1-D AoS variables, and
   1-D AoS element field access.

2. **Variant B: whole-struct comparison for structs with multi-dimensional
   array-typed fields.**
   Generalize W562 to scalar struct fields that are 2-D fixed-size scalar
   arrays, e.g. `struct Tile { m: [2][3]i8, tag: u8 }`.

3. **Variant C: negative / boundary witnesses for non-lowerable scalar-array
   fields.**
   Add witnesses where a scalar struct field is an array of `f32`, `string`,
   `enum`, or unresolved-import type, proving the classifier rejects the whole
   struct and the W562/W560 optimization cannot fire.

---

## Skills to carry forward

- When extending a packed-vector slice path, distinguish three base shapes:
  plain identifier, predeclared call temporary, and raw function-call
  expression. Parentheses rules differ for each.
- A single malformed Verilog pattern (`$signed(tmp[...])[i]`) is often the
  symptom of two independent indexing passes fighting. Collapse them into one
  dynamic part-select with the correct offset arithmetic.
- End-to-end bench witnesses with whole-struct assertions are the fastest way
  to lock both compiler packing and reference-model packing at once.
