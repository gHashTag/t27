# Wave Loop 587 — the first clean failure list, and an import that resolved to nothing

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_586_REPORT.md`](WAVE_LOOP_586_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants taken.

```
A  taxonomy of the 98 genuinely broken     8,072 assertions, ranked
B  C gate split                            296 failures -> 159 unwritten + 137 broken
C  the board                               verified, still BLOCKED

parse 341 -> 397 (0 regressions) · ALL_PASS 28 (683 tests)
UNIMPLEMENTED 118 · COMPILE_FAIL 98 · lex-conform 29/29 · parse-conform 13/13
T1/T2/T3 re-proved
```

Science extended: two new measured propositions (P7, P8) in
[`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md), and
the open question about the `.tri` sources is now **settled** rather than open.

---

## 1. Variant A — the first failure list with nothing unwritten in it

W586 separated *unwritten* from *broken*. The 98 that remain are all real, and
they hold **8,072 substantive assertion clauses**:

| Assertions | Specs | Class |
|---:|---:|---|
| **4,142** | **44** | `use of undeclared identifier` |
| 733 | 3 | `expected type X, found Y` |
| 706 | 2 | `duplicate struct member name` |
| 699 | 6 | `expected X after declaration` |
| 311 | 2 | `expected expression, found X` |

Every previous ranking of `COMPILE_FAIL` was diluted by 118 specs no compiler
change could move. This one is not.

The top class is a **long tail** — 44 specs, mostly unique names — except for
two clusters that are module-qualified references (`constants`, `eval`), and one
that turned out to be a bug in my own resolver.

## 2. The bug: an import that silently resolved to nothing

`cordic_fixed.t27` still failed on `cordic_gain` — a function I had connected in
W571 by adding the import that declares it. The import line:

```t27
use igla::race::cordic;   // W571: cordic_gain, tested here and declared there
```

`use_targets` strips a trailing `;` and then splits on `::`. The semicolon is not
at the end of the line — **the comment is** — so the module path became
`igla::race::cordic;   // W571: cordic_gain, tested here…`, resolved to no file,
and the import quietly did nothing.

**The comment that broke the import was the one I wrote to explain the import.**

26 `use` lines in the corpus carry a trailing comment. Comments are now stripped
before the path is parsed, in both `use_resolve` and `check_calls`.

This is the seventh instance of the chain's recurring shape — a component that
accepted input, produced a smaller result, and reported success — and the first
where the input I broke was my own.

## 3. Variant B — the C gate agrees with the harness

`t27c cc-gate` counted 296 failing headers without knowing which were unwritten.
Both commands now share one predicate (`impl_status::spec_is_unwritten`):

```
  headers that COMPILE   101
  headers that FAIL      296
    of those, UNWRITTEN  159   (every function body empty -- not a header defect)
    genuinely broken     137
  no header generated    211
```

Two measurement systems, one definition. Before this the C gate and the harness
would have reported different totals for the same population.

## 4. Variant C — the board

```
board      : QMTech Wukong V1 (XC7A200T-FGG676)   expect id : 0x03636093
bitstream  : ternary_mac_demo_top_v2_200t.bit     9,730,764 bytes  [OK]
cable link : ABSENT
verdict    : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2 and T3 re-proved this wave. Software-side ready since W553. The only
external dependency in the entire project remains the physical board and its
Digilent HS2 cable.

## 5. The science

Two propositions added, each with method, number and falsification condition:

- **P7** — 40% of the specs that parse have no implementation. 667 of 2,854
  declared functions have no body. Consequence: `COMPILE_FAIL 216` was
  `98 + 118`, and the metric this chain drove down from W560 to W585 was more
  than half a population no compiler change could move.
- **P8** — the `.tri` sources named in 169 header comments **do not exist**.
  1 basename match (an architecture diagram), 94 function declarations across all
  26 `.tri` files, 5 bodies.

P8 closes an open question rather than raising one: the 571 empty bodies are not
recoverable by regeneration, so each is a spec-authoring decision.

The recurring-shape table now has seven rows, and W587 adds the sharpest one:
**a component silently discarding input, where the input was a comment I wrote
to document the thing it broke.**

---

## 6. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| `t27c cc-gate` | 101 compile · 159 unwritten · 137 broken |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| `check-calls` | 51 (48 arity, 3 aggregate-vs-scalar) |
| T1 / T2 / T3 | re-proved |

---

## 7. Three cooperation variants for W588

### Variant A (recommended) — `duplicate struct member name` and the two type classes

706 assertions in 2 specs, 733 in 3, 699 in 6 — **2,138 assertions in eleven
specs**, all in classes with a small, definite shape. `duplicate struct member
name` in particular is the same class of spec defect as W568's duplicate test
names, which had a one-line fix per site and a codegen mitigation.

### Variant B — The `constants` / `eval` module-qualified references

`constants.PHI` and `eval::has_substring` are cross-module references that the
resolver does not pull, because it matches on **bare names** and these arrive
qualified. Seven specs; the fix is in `use_resolve`'s needed-name computation and
is the natural completion of W569.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** Eleven specs, three classes with definite shapes, 2,138
assertions, and the list is finally free of anything that cannot be fixed.

---

*φ² + φ⁻² = 3 | TRINITY*
