# Wave Loop 593 — a cordic spec reached `@panic at comptime`, which means it compiled

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_592_REPORT.md`](WAVE_LOOP_592_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  the cordic family     three codegen gaps closed; cordic_top now reaches a
                         COMPTIME ASSERTION -- it compiles, and an invariant is false
B  float inference       extended to locals; and signed `/` -> @divTrunc, 218 sites
C  the board             verified, still BLOCKED

ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98 · parse 397, 0 regressions
lex-conform 29/29 · parse-conform 13/13 · cc-gate 101/159/137 · T1/T2/T3 re-proved
```

---

## 1. Three codegen gaps, each general

**Array literal in RETURN position.** `return ([s], [c])` from a function
declared `-> ([]f32, []f32)` emitted `.{ .{ s }, .{ c } }`, and Zig answered
*"type '[]f32' does not support array initialization syntax"*. The element type
is in the signature — exactly as it is for a call argument (W571) — so the
return type is now tracked per function and a tuple return distributes over its
element types. `&[_]f32{…}` is `*const [1]f32`, so the same `@constCast` W571
needed applies here.

**Signed integer division.** Zig refuses `/` on signed integers: the rounding
mode must be explicit. **218 division sites in the corpus.** A division with a
known-signed operand now emits `@divTrunc`, which is C's and Rust's semantics and
what the specs assume. Signedness is inferred from declared parameter, field and
local types, and from casts — the same mechanism as W582's `string_names` and
W592's `float_names`.

**Float-typed locals.** W592's cast fix inferred float-ness from parameters and
struct fields only, so `let x: f32` took the `@floatFromInt` branch. Locals are
now collected per function and removed on exit, so one function's locals cannot
leak into the next.

## 2. What the cordic family does now

| Spec | Was (W592) | Now |
|---|---|---|
| `cordic.t27` | `[]f32` array-init | `no field named 'sin' in tuple` — a **named** tuple access |
| `cordic_fixed.t27` | missing `cordic_tan` | a float-literal narrowing |
| `cordic_top.t27` | missing `compute_sine` | **`@panic at comptime: assertion failed`** |

That last line is the point. `cordic_top.t27` **compiles**, its invariants are
being evaluated at comptime, and **one of them is false**. After twenty-five waves
of "does not compile", one of the IGLA RACE kernels has produced a real
mathematical verdict about itself.

It is not yet known which invariant, or whether the invariant or the
implementation is wrong — that is W594.

## 3. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved.

---

## 4. A note on the inference mechanism

Three waves have now added a `*_names` set to the Zig codegen — strings (W582),
floats (W592), signed integers (W593) — each collected from declarations and each
used to pick a Zig spelling that depends on a type the AST does not carry.

**This is a type checker being grown one predicate at a time**, and it should be
said plainly rather than discovered later. Its known weakness is already visible:
parameter names are collected corpus-wide into one set, so a parameter `a: i32`
in one function makes every `a` signed. For `@divTrunc` that is harmless — it is
valid for unsigned operands too, and the generated file compiles — but the next
predicate may not have that property.

**The honest description is that t27 has no type checker, and the backends are
accumulating a partial one.** Recorded here so the next wave inherits the
statement rather than the surprise.

---

## 5. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| Parse, 608 non-scratch specs | 397, **0 regressions** |
| `lex-conform` / `parse-conform` / `cc-gate` | 29/29 · 13/13 · 101/159/137 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 6. Three cooperation variants for W594

### Variant A (recommended) — Find out which invariant is false

`cordic_top.t27` compiles and fails a comptime assertion. **This is the first
mathematical claim this corpus has made and had checked**, and the answer is
worth more than another compile fix: either a CORDIC invariant in the spec is
wrong, or the implementation this chain wrote in W592 is.

**Deliverables.** Identify the failing invariant; determine whether the spec or
the implementation is at fault; state which, with the arithmetic.

**What would falsify the framing.** If the failure is in an invariant this chain
*wrote* rather than one the spec already had, it is my defect and not a finding
about the corpus — check the provenance of the invariant before drawing any
conclusion from it.

### Variant B — Name the partial type checker

The three `*_names` sets are a type checker in all but name, with corpus-wide
scoping as a known flaw. Making it one structure with per-function scope would fix
the leak, make the next predicate cheap, and stop the pattern from being
rediscovered a fourth time.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A**, falsification first. A false invariant in a CORDIC kernel is the
most interesting thing this corpus has produced, provided it is the corpus's and
not mine.

---

*φ² + φ⁻² = 3 | TRINITY*
