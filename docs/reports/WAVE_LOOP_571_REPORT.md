# Wave Loop 571 Report — four functions written from their own tests, and the two that could not be

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_570_REPORT.md`](WAVE_LOOP_570_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
assertions emitted, 201 BDD specs   4,374  ->  4,393
non-scratch parse OK                  341  ->    351     (0 regressions vs W568)
harness                             ALL_PASS 22, TEST_FAIL 0
generated Verilog                   17 identical, 1 strictly larger
T1 / T2 / T3                        re-proved
```

A smaller wave in numbers than W570, and a more honest one in what it establishes:
**four of the six missing RACE names were writable from tests already in their own
files; two are specification questions, and this report names the artefact that
settles each.**

---

## 1. Written, each determined by its own tests

| Spec | Written | What determines it |
|---|---|---|
| `cordic.t27` | `cordic_sin`, `cordic_cos` | `cordic_sin(pi, 12)` must satisfy `abs_f32(s) < 0.01` and `cordic_sin(0.001, 12)` must land in `(0.0, 0.002)` — both properties of `cordic_sin_cos(angle, iters).0[0]`, which the file already defines |
| `adder_tree.t27` | `adder_tree`, `adder_tree_inner` | `adder_tree([0,0,0,0]) == 0` and `adder_tree([1]*8) == 8` — the N-input form the fixed-width trees specialise |
| `ternary_gemm.t27` | `ternary_gemm` | dispatch on flattened length to the 2x2 / 4x4 / 8x8 forms the file already defines and tests; adds no arithmetic, only size selection |
| `cordic_fixed.t27` | `use igla::race::cordic;` | its test asserts `abs_f32(g - 0.6073) < 0.01` — the CORDIC gain, already declared in `cordic.t27`. The dependency was assumed and never declared; W569's resolver does the rest |

Nothing here is invention: each definition is the one its own assertions require, in
the style of the functions beside it.

## 2. Two that could not be written, and the artefact that decides each

### `systolic_ternary_array` — contradictory tests

```t27
invariant systolic_ternary_array_len_equals_size
    systolic_ternary_array(...).len() == size

test systolic_ternary_array_empty_weights
    given size = 2
    then result.len() == 0            // <- size is 2
```

and `[3, 4]` with weights `[-1, +1]` is asserted to give `[1, 8]` — neither the
elementwise product nor a running accumulation.

**Deciding artefact:** the systolic implementation in `fpga/verilog/`. Whichever
behaviour the RTL implements is the one the spec should assert, and the contradicting
tests should be corrected against it.

### `OP_ADD` / `OP_SUB` — outside the declared opcode set

`opcodes.t27` declares eleven opcodes (`OP_LOAD_PHYSICS_CONST` 0xDE …
`OP_CORDIC_SIN_COS` 0xE8) with `OPCODE_COUNT : u8 = 11`, and
`validate_opcode_chain([OP_ADD, OP_SUB])` is asserted **true** — which requires both to
pass `is_sacred_opcode`. Adding them changes what the sacred set is and what
`OPCODE_COUNT` counts. Neither name exists anywhere in the repository.

**Deciding artefact:** the ISA encoding table under `specs/isa/`. If `OP_ADD` belongs
in the sacred set it needs an encoding and `OPCODE_COUNT` must change; if it does not,
the test is asserting the wrong thing.

## 3. Two backend fixes found by following the next error

**Array-literal argument where the callee declares a slice.** `.{ 0, 0, 0, 0 }`
coerces to `[4]i32` but not to `[]i32`, and only the callee's signature says what the
element type is. Codegen now records declared parameter types and emits
`&[_]i32{ … }` at such call sites.

**Single-element list vs. array dimension.** `[cast_i8(42)]` was rejected as a
possible dimension (`[4]`, `[SIZE]`), so its contents stayed raw text and the call
inside was never lowered — `use of undeclared identifier 'cast_i8'` from inside an
already-emitted `.{ cast_i8(42) }`. A sole element that is not a literal or a bare
identifier is now accepted as a list.

---

## 4. What still blocks the family, precisely

Each RACE spec now fails on exactly one thing, and the classes are down to three:

| Shape | Specs | Example |
|---|---|---|
| Array literal bound to a **local**, then passed where a slice is expected | `adder_tree`, `ternary_mac` | `const inputs = .{0,0,0,0}; adder_tree(inputs)` |
| Postfix `.method()` on a **call result** loses its receiver | `ternary_gemm` | `ternary_gemm([…], […]).len() == 4` emits `len()` |
| Undeclared name, pending a specification decision | `systolic_ternary`, `opcodes` | §2 above |

The first two are mechanical and bounded. The receiver-losing bug is the more serious
of the two: it silently drops an expression rather than failing, which is the same
class of defect as W569's truncation.

---

## 5. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Assertions emitted, 201 BDD specs | 4,374 → **4,393** |
| Harness | `ALL_PASS 22, COMPILE_FAIL 179, TEST_FAIL 0` |
| Generated Verilog | 17 byte-identical, 1 strictly larger (a previously-discarded test now emits its check) |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on every `compiler.rs` edit |

---

## 6. Three cooperation variants for W572

### Variant A (recommended) — The receiver-losing postfix call

`ternary_gemm([…], […]).len() == 4` emits `len()`. The parser builds a dotted callee
name from a chain of *identifiers* and drops a receiver that is itself a call. It fails
loudly here only because `len` happens to be undeclared; where the method name matches
something in scope it would silently call the wrong thing on nothing.

**This is a silent-wrong-code bug, not a missing feature**, and it is the same shape as
the truncation W569 found: the compiler discarding input without saying so. Fix it
first, and audit for other places the parser builds a name by concatenation instead of
by structure.

### Variant B — Type a local from the slice it is passed to

`const inputs = .{0,0,0,0}; adder_tree(inputs)` needs `inputs` declared as
`[_]i32{…}` and passed as `&inputs`. The element type is knowable from the callee's
signature, which codegen now records — the missing piece is propagating it back to the
binding. Unblocks `adder_tree` and `ternary_mac` directly.

### Variant C — Settle the two specification questions

`systolic_ternary_array` against the systolic RTL in `fpga/verilog/`, and `OP_ADD` /
`OP_SUB` against the ISA table in `specs/isa/`. Both are small once decided, and
neither is mine to decide. Together they are the last two RACE kernels.

---

## Recommendation

**Variant A.** A parser that drops an expression without reporting it is worth more
attention than two blocked specs, and this chain has now found the same shape twice.

---

*φ² + φ⁻² = 3 | TRINITY*
