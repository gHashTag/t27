# Wave Loop 563 Closeout Report — Array-of-struct return call deduplication

**Issue:** #1534  
**Branch:** `wave-loop-563`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 563 implements **Variant A** from Wave Loop 563/564: extend the
W556–W560 block-scoped call-CSE machinery to function calls that return
fixed-size arrays of lowerable packed scalar structs (`[N]Pt`). The three
prerequisite gaps identified in W561 are closed:

1. `ExprArrayLiteral` lowering for `[N]Pt` literals already existed in
   `emit_packed_array_literal_concat`; it is now exercised end-to-end.
2. `emit_local` now declares 1-D arrays of scalar structs as a single packed
   vector and assigns them wholesale from a call or a packed array literal.
3. `try_emit_struct_array_access` now handles 1-D arrays and call bases
   returning arrays of scalar structs, emitting a correct packed slice for
   `tmp[i].x` and `make_pts(...)[i].x`.

`call_returning_cse_value_info` now recognizes `[N]Pt` returns, so a single
packed-vector temporary is shared across all use sites of the same call in one
test/bench block.

---

## What changed

### `.claude/plans/wave-loop-563.md`

- Decomposed plan documenting the three prerequisites, the CSE extension,
  witness design, validation matrix, and three W564 cooperation variants.

### `bootstrap/src/compiler.rs`

- Added a 1-D array-of-struct branch in `emit_local`:
  - Declares `reg [N*elem_w-1:0] tmp;`.
  - For an `ExprArrayLiteral` initializer, emits a packed concatenation via
    `emit_packed_array_literal_concat`.
  - For a call or other packed-vector initializer, assigns it directly.
- Generalized `try_emit_struct_array_access`:
  - Removed the `dims.len() < 2` guard so 1-D arrays are handled.
  - Added an `ExprCall` base path: looks up the predeclared call temporary when
    `use_call_array_temps` is active, otherwise falls back to a parenthesized
    raw call expression.
  - Computes the linear element index and optional field offset as before.
- Extended `call_returning_cse_value_info` to return a temporary descriptor for
  arrays whose element type is a lowerable packed scalar struct.
- Updated unit test `test_verilog_struct_field_access_indexed` to assert the
  new packed-slice output instead of the old flattened-name placeholder.

### `bootstrap/stage0/FROZEN_HASH`

- Updated to `92fb8abd6bc5245b5a3f7aa1b9eb54917c5f4e9ec2622f51c2e9a548030f5665`.

### Witnesses and seals

- Added `specs/scratch/w563_bench_array_of_struct_call_dedup.t27`:
  - `struct Pt { x: i16, y: i16 }`
  - `pub fn make_pts(...) -> [2]Pt`
  - test: local 1-D AoS initialized from call, field accesses on local and
    call-return elements.
  - bench: same pattern with different values.
- Saved t27 seal under
  `.trinity/seals/scratch_w563_bench_array_of_struct_call_dedup.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w563_bench_array_of_struct_call_dedup.json`.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w563_bench_array_of_struct_call_dedup` integration test.

### Reseals

Four existing corpus seals whose generated Verilog changed due to the compiler
edit were resealed:
- `boards_BoardMinimalXC7A100T.json`
- `fpga_ApbBridge.json`
- `fpga_TernaryIsa.json`
- `numeric_GoldenFloatFamily.json`

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 23 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W563 witness | `[TEST] ... PASSED`, `[BENCH] ... PASSED` |
| Direct `t27c icarus-cocotb` on W563 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W563.

---

## Notes and known limitations

- The witness exercises a 1-D `i16` scalar-struct array. 2-D array-of-struct
  returns and multi-dimensional field access are not explicitly covered by this
  wave, although the generalized slice path should handle them.
- The call-CSE optimization applies only to pure, side-effect-free calls inside
  deterministic `test` / `bench` blocks.
- Non-lowerable array-of-struct returns (e.g. structs with `string`/`enum`/`f32`
  fields) remain rejected by the structural classifier, as locked by W561.

---

## Three cooperation variants for Wave Loop 564

1. **Variant A — Recommended: whole-array comparison for 1-D arrays of scalar
   structs.** Extend the W555/W562 whole-array `assert_eq` probe path to packed
   1-D AoS values, enabling `assert_eq(make_pts(...), [2]Pt{...})` in bench
   blocks.

2. **Variant B: 2-D array-of-struct return call deduplication.** Generalize
   W563 to function calls returning 2-D arrays of scalar structs (`[N][M]Pt`).
   The existing multi-D local/field access paths should already cover most of
   this; the CSE descriptor and call-temporary slice emission need to be
   verified.

3. **Variant C: negative / boundary witnesses for non-lowerable array-of-struct
   returns.** Add witnesses where a function returns `[N]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the structural
   classifier rejects the whole return type before CSE can apply.

---

## Skills to carry forward

- When extending a packed slice path, handle three base shapes: plain
  identifier, predeclared call temporary, and parenthesized raw call
  expression. The same arithmetic (linear index * elem_width + field_offset)
  applies to all of them.
- A 1-D array of packed scalar structs is just a wider packed vector; declare
  it and assign it wholesale rather than per-element.
- End-to-end bench witnesses that mix local initialization, local indexing, and
  repeated call indexing are the fastest way to lock declaration, access, and
  CSE semantics at once.
