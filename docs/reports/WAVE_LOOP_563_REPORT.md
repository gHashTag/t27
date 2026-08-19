# Wave Loop 563 Report — 45 → 167 executing tests

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_562_REPORT.md`](WAVE_LOOP_562_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W563 drained two more entries from W560's measured queue. The second one was the
unlock.

```
                     W560   W561   W562   W563
ALL_PASS                5      7      9     14
COMPILE_FAIL          194    192    190    184
tests executing        45     54     64    167     (+271% since W560)
```

---

## 1. `&T` in a parameter type produced *two* parameters

`parse_type_annotation` never consumed a leading `&`, so it returned an **empty**
type and the parameter loop then read the type name as the next parameter:

```
fn find_pin_by_port(name: &str)   ->   fn find_pin_by_port(name: , str: )
```

**103 specs** use `str`/`&str`. This was the entire
`expected type expression, found ','` class — 11 of an 80-sample.

t27 has no reference types, so the borrow marker is accepted and dropped; the
Zig and Rust mappers already strip it.

**A parser change, so the full census was mandatory:**

```
PARSE OK=746 FAIL=317   (baseline 317)
REGRESSIONS: 0
```

Taxonomy confirms it landed: `expected type expression, found ','` **11 → 0**.

### The top-line metric did not move for this fix alone

`ALL_PASS` stayed at 9. The fix removed one error class and the next surfaced in
the same specs. **The taxonomy shift is the evidence, not the headline number** —
that is the expected shape when draining a first-error queue: each fix buys the
next diagnosis, not necessarily a passing spec.

## 2. String `==` — a class my own W562 fix created

Zig has no `==` for slices. Once W562 started emitting string literals *with*
their quotes, `name == "clk"` began failing with
`cannot compare strings with ==` — a new class of 7, caused by that fix.

When either operand is a string literal the comparison is a string comparison,
so it now lowers to `std.mem.eql(u8, a, b)`, negated for `!=`:

```
assert(n() == "clk")  ->  if (!(std.mem.eql(u8, n(), "clk"))) @panic(...)
```

**This was the unlock: 64 → 167 executing tests.** String equality is pervasive
in the assertions, so making it compile released a large backlog at once.

---

## 3. Where the corpus stands

| | |
|---|---:|
| Substantive assertion clauses written | 11,282 |
| **Executing today** | **167** |
| Specs fully passing | 14 of 199 |
| Specs blocked by `default_input()` | 169 |

The `default_input()` wall (settled in W562 as *not* mechanically fixable) is now
the overwhelming remainder.

---

## 4. Three cooperation variants for W564

### Variant A (recommended) — Finish the small queue

Remaining measured classes, each with a reproduction:

1. `use of undeclared identifier` (45/80) — overwhelmingly `default_input`;
   the non-`default_input` remainder is worth separating out.
2. `expected X, found Y` (7/80) — undiagnosed.
3. `expected ; after statement` (5/80) — undiagnosed.
4. `duplicate test name` (5 specs) — genuine spec defect.
5. `operator <` on enums (2/80) — emit `@intFromEnum`.
6. One `UNKNOWN` harness outcome — classify it.

### Variant B — Decide the fate of the 571 template tests

Unchanged and still the single biggest lever: 169 specs cannot compile because
of `default_input()`, which W562 proved unfixable mechanically (48 uniform,
96 mixed types, 25 calling functions that do not exist). Rewriting or deleting
them would release the largest remaining block. **Maintainer's call** — it
changes test intent.

### Variant C — Lower keyword-form invariants

Unchanged since W559: **5,163 invariants** still emit
`// invariant: X verified (no statements)`. Largest single remaining inert
population, and the W559 lowering pattern applies directly.

---

## Recommendation

**Variant A**, then **C**. The queue is nearly drained and each item is small.
Variant B is the biggest lever but needs a human decision, and it has needed one
since W562.

---

*φ² + φ⁻² = 3 | TRINITY*
