# Wave Loop 570 Report — assertions emitted 1,323 → 4,374

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_569_REPORT.md`](WAVE_LOOP_569_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W570 took its own Variant A — supply the helpers the IGLA RACE kernels are missing —
and found that most of them are not missing spec functions at all. They are **typed
spellings the backend never learned**, each defined nowhere because none was ever
supposed to be written.

```
assertions emitted, 201 BDD specs   1,323  ->  4,374     (+3,051, x3.3)
non-scratch parse OK                  341  ->    351     (0 regressions)
harness                             ALL_PASS 22, TEST_FAIL 0
generated Verilog                   17 identical, 1 strictly larger
T1 / T2 / T3                        re-proved
```

The assertion count is the honest measure of this wave: it counts checks that reach
the generated code, in specs whose test blocks were previously discarded or whose
bodies referenced a name the backend could not spell.

---

## 1. Five lowerings, each measured

| Lowering | Occurrences | Was |
|---|---:|---|
| `x.len()` → `x.len` | **1,356** in 51 specs | `use of undeclared identifier` — Zig exposes slice length as a *field* |
| `len(x)` → `x.len` | 318 | same, free-function spelling |
| `: string` → `[]const u8` | **952** | `str` was mapped; `string` never was |
| `cast_i8(x)` → `@as(i8, @intCast(x))` | **1,140** | defined nowhere in the corpus |
| `abs_f32/abs_f64/abs_i16(x)` → `@abs(x)` | 39 | same |

`cast_*` is deliberately restricted to integer targets: a float cast needs
`@floatCast` or `@intFromFloat` depending on the source, and guessing between them
would be a silent semantic choice.

## 2. The array literal that ate its own test block

```t27
test t
    given a = [1, 2, 3]
    then a.len() == 3
```

generated an **empty test body**. `parse_bare_array_literal` rejects a literal
followed by an identifier, because `[5]Pt` is a type and not a list — but it did not
check whether that identifier was on the same line. The `then` on the next line read
as a type name, the literal was rejected, and the whole clause block fell back to
being discarded.

Requiring the type name to sit on the **closing bracket's own line** fixes it. This is
the single largest contributor to the +3,051: array literals in `given` bindings are
the normal shape in the GEMM and systolic specs.

## 3. A parameter that was used and discarded anyway

```zig
fn model_weight_count(model: TernaryModel) u32 {
    _ = model;            // "pointless discard of function parameter"
    return model.weights.len;
}
```

The unused-parameter scan compared `n.name == "model"`, but a dotted callee keeps its
whole path in `name` — `model.weights.len` never matched. Matching the **root segment**
of a call path fixes it.

## 4. One genuine spec function, written from its own tests

`adder_tree.t27` has tested `adder_tree_2` since it was written and never declared it,
so every test in the file was blocked. Its tests fully determine it:

```t27
test adder_tree_2_basic      given a = 3  given b = 4  when sum = adder_tree_2(a, b)  then sum == 7
invariant adder_tree_2_commutative        adder_tree_2(a, b) == adder_tree_2(b, a)
```

Written as the leaf stage of the tree the file already builds, matching
`adder_tree_4`'s style.

## 5. One that could NOT be written, and why

`systolic_ternary_array` is tested but undeclared, like `adder_tree_2`. Unlike it, its
tests **contradict each other**:

```t27
invariant systolic_ternary_array_len_equals_size
    systolic_ternary_array(activations, weights, size).len() == size

test systolic_ternary_array_empty_weights
    given activations = [1, 2]
    given weights = []TernaryWeight{}
    given size = 2
    then result.len() == 0            // <- size is 2
```

and the element semantics do not follow from the two-element case either
(`[3, 4]` with weights `[-1, +1]` is asserted to give `[1, 8]`, which is neither the
elementwise product nor the running accumulation).

Per the standing rule — *when you defer, name the artefact that would decide it* — the
deciding artefact is the RTL: `fpga/verilog/` contains a systolic implementation, and
whichever behaviour it implements is the one the spec should assert. This is a
**specification decision**, not a guess to be made by the compiler.

---

## 6. Verification

| Gate | Result |
|---|---|
| Assertions emitted, 201 BDD specs | **1,323 → 4,374** |
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Harness | `ALL_PASS 22, COMPILE_FAIL 179, TEST_FAIL 0` |
| Generated Verilog, FPGA + board specs | 17 byte-identical, 1 strictly larger |
| Icarus on the file that changed | fails identically to W568 (pre-existing) |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on every `compiler.rs` edit |

`ALL_PASS` is unchanged at 22, and that is the accurate picture: the RACE kernels each
still fail on **one** undeclared name. What moved is the amount of real checking inside
the files that do build — and the distance the rest have left to travel.

The one Verilog difference is an addition: a test block that was previously discarded
now emits its check.

---

## 7. Three cooperation variants for W571

### Variant A (recommended) — Write the six functions the RACE specs test

Every remaining RACE blocker is now a single name that exists nowhere in the corpus:

| Spec | Missing | Decidable from its tests? |
|---|---|---|
| `adder_tree.t27` | `adder_tree` | likely — an alias or the N-input form |
| `ternary_gemm.t27` | `ternary_gemm` | likely — the general form beside `ternary_gemm_2x2` |
| `cordic.t27` | `cordic_sin` | yes — angle in, sine out, tested against 0.70710678 |
| `cordic_fixed.t27` | `cordic_gain` | yes — the CORDIC gain constant |
| `opcodes.t27` | `OP_ADD` | yes — an opcode constant, from the encoding table |
| `systolic_ternary.t27` | `systolic_ternary_array` | **no** — contradictory tests, see §5 |

Five of six are determined by the tests already in the file. Write them there, one at
a time, with the harness as the gate. **~1,600 substantive assertions.**

### Variant B — `.{ … }` where a slice is expected

`ternary_mac.t27` now fails with `expected type '[]i8', found 'struct { comptime T = 1, … }'`.
An anonymous list coerces to `[N]T` but not to `[]T`; where the target is a slice the
backend must emit `&[_]T{ … }`, which needs the element type at the call site. This is
the last *mechanical* blocker in the family and it recurs wherever an array literal is
passed to a function.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, preflight correctly reporting
`BLOCKED -- no programmer on USB`, all three theorems re-proved this wave. Needs the
QMTech Wukong V1 and a Digilent HS2 cable.

---

## Recommendation

**Variant A.** Six names, five of them decidable from tests already written in the same
file, standing between this project and its own kernels' test suites. The sixth is a
specification question with a named artefact to settle it.

---

*φ² + φ⁻² = 3 | TRINITY*
