# Wave Loop 562 Report — string literals had no quotes

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_561_REPORT.md`](WAVE_LOOP_561_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W562 worked the compile-failure queue by measured size. Two Zig-emitter defects
fixed, one queue item proved **not** mechanically fixable.

```
                     W560   W561   W562
ALL_PASS                5      7      9
COMPILE_FAIL          194    192    190
tests executing        45     54     64      (+42% since W560)
```

---

## 1. String literals lost their quotes

The lexer strips the surrounding quotes and stores the raw text, tagging the
node `extra_kind == "string"`. The Zig emitter wrote `node.value` back
**unquoted**, so every string literal became a bare identifier:

```
pub const NAME: str = "hello";   ->   pub const NAME = hello;
assert(x == "world")             ->   if (!(x == world))
.port_name = "clk"               ->   .port_name = clk
```

A literal containing spaces became a *run* of identifiers:

```
name == "Digilent Arty A7-35T"   ->   if (!(name == Digilent Arty A7-35T))
```

This is a large share of the `use of undeclared identifier` class W560 measured
at **104 of 194** first errors. The C and Rust paths already handled
`extra_kind == "string"` (`compiler.rs:3701`, `9470`); **the Zig path never
did.**

## 2. Struct fields and const decls bypassed the type mapper

W561 mapped `str`/`&str` → `[]const u8` in `t27_array_type_to_zig`, but struct
field types and const declaration types were emitted raw, so `&str` still
reached Zig in those positions — the residual
`expected type expression, found '&'` class. Both now route through the mapper.

---

## 3. `default_input()` is not mechanically fixable

W561 deferred this; W562 settled it. Of the **169** specs calling
`default_input()`:

| | |
|---|---:|
| All template calls share one first-param type (**one helper works**) | **48** |
| **Mixed** first-param types — one helper cannot satisfy them | **96** |
| Call a function that **does not exist in the spec at all** | **25** |

So it is not one missing helper, and not 169 either — it is a scaffold that
calls differently-typed functions through a single untyped hole, and in 25 cases
calls functions nobody wrote. **Patching it is not possible; the 571 template
tests need rewriting or removal**, which is a maintainer's decision because it
changes test intent.

---

## 4. Where the corpus stands

| | |
|---|---:|
| Substantive assertion clauses written | 11,282 |
| **Executing today** | **64** |
| Specs blocked by `default_input()` | 169 |
| Specs blocked by other causes | ~21 |

---

## 5. Three cooperation variants for W563

### Variant A (recommended) — Finish the tractable queue

Remaining measured classes, all compiler-side and all with a before/after
metric:

1. `expected X, found Y` (8/80) — undiagnosed.
2. `expected ; after statement` (5/80) — undiagnosed.
3. `duplicate test name` (5 specs) — a genuine spec defect; Zig rejects it.
4. `operator <` on enums — emit `@intFromEnum` in comparisons.

Re-run the committed harness after each; report `ALL_PASS` / `tests executing`.

### Variant B — Decide the fate of the 571 template tests

They assert `result != undefined`, call an untyped helper that cannot exist,
and in 25 specs call functions that were never written. Rewrite them with real
assertions, or delete them. **Maintainer's call** — it changes test intent, and
leaving them means `validate-vacuity` keeps understating how much of the corpus
asserts nothing.

### Variant C — Lower keyword-form invariants

Unchanged since W559: **5,163 invariants** still emit
`// invariant: X verified (no statements)`. The W559 lowering pattern and its
fixture-and-census discipline apply directly. This is the last large inert
population.

---

## Recommendation

**Variant A.** Four measured classes remain, each small and each with a
reproduction. Variant B needs a human, and Variant C is the largest single
remaining win but also the riskiest — worth doing after the cheap queue is
drained.

---

*φ² + φ⁻² = 3 | TRINITY*
