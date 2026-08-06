# Wave Loop 560 Closeout Report — Scalar-struct return call deduplication

**Issue:** #1531  
**Branch:** `wave-loop-560`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 560 implements **Variant A** from Wave Loop 559/560: extend the
W556–W558 block-scoped call temporary machinery to **lowerable packed
scalar-struct return calls**. A function such as `make(a: i16, b: i16) -> Pt` is
now lowered so that a single packed-vector temporary (`reg [31:0]
_t27_call_tmp_*`) is declared per `test` / `bench` block and reused for every
whole-struct comparison, field-access comparison, and local initializer that
references the same call site.

The change is intentionally narrow:

- The structural classifier already accepted lowerable packed scalar structs;
  W560 only had to teach the **CSE predeclarer** that a scalar-struct return
  is a single packed value like a scalar or a packed primitive-array return.
- The `ExprFieldAccess` emission path already emitted a part-select over the
  packed call expression; it needed only to recognize that the slice base is a
  predeclared temporary and avoid wrapping it in extra parentheses.
- The Python reference model needed a small alignment fix: struct-literal field
  values must be masked to their **declared** width before packing, and the
  packed scalar-struct vector is unsigned, matching the compiler probe reg.

Three scratch witnesses cover the actual side, the expected side, and multiple
field accesses sharing one temporary.

---

## What changed

### `.claude/plans/wave-loop-560.md`

- Decomposed plan documenting weak points, scientific background (CSE in
  hardware compilers), implementation tasks, acceptance criteria, and three
  W561 cooperation variants.

### `bootstrap/src/compiler.rs`

- Added a scalar-struct branch in `call_returning_cse_value_info` so that a
  lowerable packed scalar-struct return is treated as a single packed-vector
  temporary (`width = packed_width(ret_ty)`, `signed = false`).
- Updated `ExprFieldAccess` emission on a call-return base to use the
  predeclared temporary name without parentheses around the part-select, e.g.
  `$signed(_t27_call_tmp_...[0 +: 16])` instead of the malformed
  `$signed((_t27_call_tmp_...)[0 +: 16])`.

### `scripts/cocotb_ref_model.py`

- Fixed `_eval_struct_lit_bv` to pack each field at its declared width rather
  than the natural evaluated width of the literal (e.g. `i16` not `32`).
- Fixed `_packed_type_width_signed` to return `signed = false` for lowerable
  packed scalar structs and arrays of such structs, mirroring the compiler's
  `packed_signed` behavior.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w560_bench_scalar_struct_call_dedup` integration test
  covering all three W560 witnesses.

### `bootstrap/stage0/FROZEN_HASH`

- Updated to the SHA-256 of the modified `bootstrap/src/compiler.rs`:
  `8ef77f2178287ff3bc2be45cb932788782a7440061f3e303516c71d18f0eb039`.

### Witnesses, seals, and baselines

- Added `specs/scratch/w560_bench_scalar_struct_call_dedup.t27`:
  whole-struct `assert_eq(make(1,2), Pt{x:1,y:2})`, field-access checks, and
  a local initializer, all reusing one call temporary.
- Added `specs/scratch/w560_bench_scalar_struct_call_dedup_both_sides.t27`:
  `assert_eq(make(5,6), make(5,6))` to exercise expected-side reuse.
- Added `specs/scratch/w560_bench_scalar_struct_call_dedup_nested.t27`:
  `make(9,10).x + make(9,10).y` and `make(11,12).y - make(11,12).x` to
  exercise two field accesses sharing one temporary per call.
- Saved t27 seals under `.trinity/seals/` for all three witnesses.
- Recorded Icarus baselines under `.trinity/icarus-baselines/specs/scratch/`
  for all three witnesses.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 20 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate specs/scratch/w560_bench_scalar_struct_call_dedup.t27` | PASS |
| Direct `t27c icarus-simulate specs/scratch/w560_bench_scalar_struct_call_dedup_both_sides.t27` | PASS |
| Direct `t27c icarus-simulate specs/scratch/w560_bench_scalar_struct_call_dedup_nested.t27` | PASS |
| Direct `t27c icarus-cocotb specs/scratch/w560_bench_scalar_struct_call_dedup.t27` | PASS |
| Direct `t27c icarus-cocotb specs/scratch/w560_bench_scalar_struct_call_dedup_both_sides.t27` | PASS |
| Direct `t27c icarus-cocotb specs/scratch/w560_bench_scalar_struct_call_dedup_nested.t27` | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W560.

---

## Generated-Verilog evidence

For `w560_bench_scalar_struct_call_dedup.t27`, the simulation Verilog shows a
single packed-vector temporary per block reused for the whole-struct
assertion, the `.x` field assertion, and the local initializer:

```verilog
// function: make
function [31:0] make; // -> Pt
    input signed [15:0] a;
    input signed [15:0] b;
    begin : make_body
        make = {$signed(b), $signed(a)};
    end
endfunction

// test: scalar_struct_call_dedup_test
initial begin : scalar_struct_call_dedup_test_test
    reg [31:0] _t27_call_tmp_scalar_struct_call_dedup_test_0; // W557 packed/scalar call tmp w=32 signed=false
    reg [31:0] tmp;
    _t27_call_tmp_scalar_struct_call_dedup_test_0 = make(1, 2);
    if (((_t27_call_tmp_scalar_struct_call_dedup_test_0) != ({16'sd2, 16'sd1}))) begin
        ...
    end
    if ((($signed(_t27_call_tmp_scalar_struct_call_dedup_test_0[0 +: 16])) != (1))) begin
        ...
    end
    tmp = _t27_call_tmp_scalar_struct_call_dedup_test_0;
    if ((($signed(tmp[16 +: 16])) != (2))) begin
        ...
    end
    $display("[TEST] scalar_struct_call_dedup_test : PASSED");
end
```

The function returns `{b, a}` as a 32-bit unsigned packed vector, the temporary
reg is 32-bit unsigned, and each field assertion uses a 16-bit signed
part-select. No second call to `make` is emitted in the block.

---

## Notes and known limitations

- The optimization applies only to **lowerable packed scalar structs** (fields
  are primitive scalars or 1-D primitive scalar arrays). Non-lowerable struct
  types continue to be rejected by the structural classifier.
- The temporary is declared at the block level, exactly like W557 scalar and
  W556 array temporaries, so it remains valid only for pure, deterministic
  calls inside `test` / `bench` blocks.
- The Python reference model now masks struct-literal fields to their declared
  width; this also makes whole-struct comparison robust for structs with signed
  fields whose sign bit happens to be set.
- The packed scalar-struct vector is treated as **unsigned** by both the
  compiler and the reference model, matching the emitted Verilog.

---

## Three cooperation variants for Wave Loop 561

1. **Variant A — Recommended: array-of-struct return call deduplication.**
   Extend the W556–W558 / W560 block-scoped call temporary machinery to
   function calls that return fixed-size arrays of lowerable packed scalar
   structs (`[N]Pt`). A single packed-vector temporary whose width is
   `N * sizeof(Pt)` would be shared across multiple sites in a `test` or
   `bench` block.

2. **Variant B: whole-struct comparison for structs with array-typed fields.**
   Extend the W555 whole-array probe path to scalar-struct variables whose
   fields are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` and
   bench assignment cross-checks for `struct { xs: [4]i8, ys: [4]i8 }`.

3. **Variant C: negative / boundary witnesses for non-lowerable struct returns.**
   Add scratch negative witnesses that exercise scalar-struct returns containing
   non-lowerable fields (e.g. `String`, unresolved imports, unbounded loops)
   and update `docs/ICARUS_LOWERABLE_BOUNDARY.md` to document that the W560
   deduplication optimization is gated by the existing lowerability classifier.

---

## Skills to carry forward

Pattern: *"When extending a block-scoped CSE optimization to a new value shape,
  first make the structural classifier accept the shape, then add a branch in
  the CSE predeclarer that returns the packed-vector metadata (width/signed)
  the same way scalars and arrays already do. The emission site that consumes
  the temporary usually needs only one small adjustment — in W560 it was
  recognizing that a predeclared identifier slice base does not need
  parentheses. Finally, verify that the Python reference model packs literal
  values at their declared widths, not at their natural evaluated widths, so
  whole-value cross-checks stay bit-accurate."*

---

Closes #1531
