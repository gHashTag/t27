# Wave Loop 564 Closeout Report — Whole-array comparison for 1-D arrays of scalar structs

**Issue:** #1535  
**Branch:** `wave-loop-564`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 564 implements **Variant A** from the W564 cooperation slate: extend
the W555/W562 whole-array `assert_eq` probe path to packed one-dimensional
arrays of lowerable packed scalar structs (`[N]Pt`). After W563 added the
packed 1-D AoS local declaration, call-return CSE, and element/field access
paths, the remaining gap was that the bench/test whole-array assertion path
still treated `[N]Pt` as non-scalar and fell back to the unsupported-literal
placeholder.

This wave closes that gap with four small changes:

1. `expr_width_signed` now returns the total packed width for local variables,
   function calls, and array literals whose type is an array of lowerable scalar
   structs.
2. `gen_verilog_expr` now lowers `ExprArrayLiteral` of `[N]Pt` to a packed
   concatenation via `emit_packed_array_literal_concat`, the same helper used
   for primitive arrays.
3. The Python cocotb reference model now computes the correct packed width for
   `[N]Pt` so the independent cross-check can reconstruct and compare the whole
   vector.
4. A new scratch witness and integration test exercise `assert_eq` on a local
   1-D AoS variable and on a call returning `[2]Pt`, in both a `test` block
   and a deterministic `bench` block.

---

## What changed

### `.claude/plans/wave-loop-564.md`

- Decomposed plan documenting Variant A implementation points, validation
  matrix, and next-wave deliverables.

### `bootstrap/src/compiler.rs`

- `expr_width_signed` `ExprIdentifier` branch: a lowerable scalar struct or
  array of scalar structs now reports the packed-vector width/signedness from
  `packed_width` / `packed_signed` (W564).
- `expr_width_signed` `ExprCall` branch: function returns that are arrays of
  lowerable scalar structs are now treated as packed vectors, enabling wide
  probe declarations for `assert_eq(make_pts(...), ...)`.
- `expr_width_signed` `ExprArrayLiteral` branch: lowerable scalar-struct array
  literals (`[N]Pt{...}`) now resolve to the total packed width.
- `gen_verilog_expr` `ExprArrayLiteral` branch: array literals whose element
  type is a lowerable packed scalar struct are now emitted as a packed
  concatenation instead of the `TODO` placeholder.

### `scripts/cocotb_ref_model.py`

- `_packed_type_width_signed` now recognizes fixed-size arrays of lowerable
  packed scalar structs and folds the base struct width across all dimensions.
- `_type_of_expr` `ExprArrayLiteral` now checks `_packed_type_width_signed`
  first, so array literals of `[N]Pt` carry the correct bit-vector width.

### `bootstrap/stage0/FROZEN_HASH`

- Updated to `b429fb26b535cb51e29a8405d5af494df6a40825e32b871abe4587ca77148b2e`.

### Witnesses, seals, and baselines

- Added `specs/scratch/w564_bench_whole_aos_1d.t27`:
  - `struct Pt { x: i16, y: i16 }`
  - `pub fn make_pts(...) -> [2]Pt`
  - test: local 1-D AoS compared whole-array against an array literal; call
    return compared whole-array against an array literal.
  - bench: same pattern under deterministic bench semantics.
- Saved t27 seal under `.trinity/seals/scratch_w564_bench_whole_aos_1d.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w564_bench_whole_aos_1d.json`.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w564_bench_whole_aos_1d` integration test.

### Reseals

No existing corpus seals changed: the W564 compiler edits only affect the
previously-unsupported `[N]Pt` whole-array literal expression shape, which no
existing corpus spec exercised. `./scripts/tri test` reports zero seal
mismatches.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 24 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W564 witness | `[TEST] ... PASSED`, `[BENCH] ... PASSED` |
| Direct `t27c icarus-cocotb` on W564 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W564.

---

## Notes and known limitations

- The witness exercises a 1-D `i16` scalar-struct array. Multi-dimensional
  arrays of scalar structs are not explicitly covered by whole-array
  assertions in this wave, although `packed_width` and
  `emit_packed_array_literal_concat` already fold dimensions generically.
- Whole-array `assert_eq` for arrays of scalar structs assumes the element
  type is lowerable (all fields primitive scalars or fixed-size arrays of
  primitive scalars). Non-lowerable AoS returns remain rejected by the
  structural classifier, as locked by W561.
- The reference-model fix is required because the previous Python width
  inference treated `[N]Pt` as a single struct, not `N` packed structs.

---

## Three cooperation variants for Wave Loop 565

1. **Variant A — Recommended: multi-site whole-array AoS call deduplication.**  
   Extend the W563 call-CSE machinery so that a whole `[N]Pt` call used at
   multiple whole-array `assert_eq` sites in the same bench block shares one
   packed-vector temporary. Add a bench witness with two `assert_eq` statements
   both consuming the same `make_pts(...)` return.

2. **Variant B: 2-D array-of-struct return call deduplication.**  
   Generalize W563 to function calls returning 2-D arrays of scalar structs
   (`[N][M]Pt`). Verify that the existing multi-D local/field access paths and
   the new CSE descriptor cooperate correctly, and add a bench witness.

3. **Variant C: negative / boundary witnesses for non-lowerable array-of-struct
   returns.**  
   Add witnesses where a function returns `[N]Pt` and `Pt` contains `string`,
   `enum`, `f32`, or an unresolved-import field. Prove the structural classifier
   rejects the whole return type, so the W563 CSE optimization cannot fire.

---

## Skills to carry forward

- When adding a new packed-vector expression shape, update width inference in
  three places: `ExprIdentifier`, `ExprCall`, and `ExprArrayLiteral`. Once the
  width is right, the existing W540 multi-slice probe path and the W551 bench
  cross-check path handle the rest.
- Array literals of lowerable scalar structs can reuse the same packed
  concatenation emitter as primitive arrays; the element renderer already knows
  how to pack struct literals.
- The cocotb reference model must independently compute the same packed-vector
  width. Arrays of structs need to multiply the base struct width by all array
  dimensions, not just return the struct width.
