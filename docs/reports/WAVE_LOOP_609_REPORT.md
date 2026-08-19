# Wave Loop 609 — the backend knew field names and never field types

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_608_REPORT.md`](WAVE_LOOP_608_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
The class was 5 errors in eval.t27.
The corpus has 589, across 20 specs.

eval.t27: 30 -> 27

and the corpus sweep caught a regression I introduced
```

---

## 1. Measure the class before sizing the work

W608 recommended a struct field-type map for 5 errors in `eval.t27`. Measured
first:

| | |
|---|---:|
| struct fields declared | **3 949** |
| of those, slice-typed | **649** |
| array literal assigned to a slice-typed field | **589** in **20 specs** |

`ternary_inference` 134, `rtl` 132, `bram_weights` 124, `formal` 84,
`pipeline` 27, `arch` 22, `backend` 20 …

**A per-file error count is a sample, not a size.**

## 2. Why the map has to be keyed by `(struct, field)`

The Zig backend already collected three sets of field names —
`string_names`, `float_names`, `signed_names` — and never a field's **type**. So

```t27
Struct { data: [1, 2, 3] }   ->   .{ 1, 2, 3 }
```

an anonymous struct Zig will not coerce to `[]T`:
*"type `[]T` does not support array initialization syntax"*.

Those three sets are keyed by **name alone**, and are therefore global — two
structs with a same-named field are indistinguishable. **A per-type question
needs a per-type key**, so the new map is `(struct, field) -> declared type`.

The lowering is the same `@constCast(&[_]T{…})` W607 added for slice *returns*:
`&[_]T{…}` is `*const [N]T`, so the mutable `[]T` most fields declare needs the
cast.

## 3. The regression the corpus sweep caught

`bram_weights.t27` began reporting `expected ',' after initializer`:

```zig
data = @constCast(&[_]i16{ 0;21 })
```

The array-**repeat** form `[v; n]` is stored as element text `v;n`, and
`gen_array_literal_braces` — the helper reused from the return path — **splits
on commas only**. `gen_expr` handles the repeat correctly (`.{v} ** n`); that
helper does not. Zig spells it `[_]T{v} ** n`. Fixed and verified compiling.

> **Not findable by reasoning about the change.** The five `eval.t27` sites that
> motivated the work contain **no repeat forms**; the defect lived in a spec
> reached only by the corpus-wide sweep. **Run the sweep before believing a
> lowering is right, not after shipping it.**
>
> **Reusing a helper inherits its blind spots.** A helper that is correct at one
> call site is not thereby correct at another — check which input shapes each
> site actually sees.

Recorded as **P24**.

## 4. Verification

| Gate | Result |
|---|---|
| `eval.t27` | **30 → 27** errors |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 401 / 608, 0 truncating |
| `cc-gate` | 101 |
| `cordic.t27` / `adder_tree.t27` | 330/336 · 335/335 |
| repeat form | verified compiling as Zig |
| unit tests | 1552 pass, 5 fail — the known `tests_w458` set |
| FROZEN_HASH | resealed |

---

## 5. Three cooperation variants for W610

### Variant A (recommended) — `usize` vs `u32`, the next mechanical class

`ternary_inference.t27` now fails at `expected type 'u32', found 'usize'`, and
the same appears in `eval.t27` twice. `.len()` yields `usize` in Zig while specs
declare `u32`, so every length-derived value needs a cast the backend is not
inserting.

It is the **same shape as W592's cast selection and W593's signed division** —
a type-mapping rule applied at one more site class — and the machinery
(`is_float_expr`, `is_signed_int_expr`, the cast-builtin table) already exists.
Measure the class first; this wave's lesson is that the per-file count
understates it by two orders of magnitude.

### Variant B — String concatenation

Unchanged from W608: 6 errors in `eval.t27`, and `+` on strings is a
language-level question — Zig's `++` is comptime-only, so a runtime concat needs
an allocator the specs never mention. **A decision about what `+` means in
t27**, which wants an owner rather than a patch.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A.** It is mechanical, the machinery exists, and it is the largest
remaining class after this wave's 589.

---

*φ² + φ⁻² = 3 | TRINITY*
