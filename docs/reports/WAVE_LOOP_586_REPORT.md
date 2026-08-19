# Wave Loop 586 — more than half of the "compile failures" are specs nobody has written

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_585_REPORT.md`](WAVE_LOOP_585_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
harness verdicts     BEFORE                    AFTER
  ALL_PASS              28  (683 tests)          28  (683 tests)
  COMPILE_FAIL         216                       98
  UNIMPLEMENTED          -                      118     <- new

parse 341 -> 397 (0 regressions) · lex-conform 29/29 · parse-conform 13/13
truncation 0 · T1/T2/T3 re-proved
```

**118 of the 216 things this project has been calling compile failures are specs
whose functions have no bodies.** They are not broken. They are unwritten, and
for twenty-five waves both facts have been one number.

---

## 1. The falsification check killed Variant A

W585 recommended regenerating the 571 empty function bodies from the `.tri`
sources every one of those specs names in its header comment — *"Implement from
.tri spec"* — with the condition:

> *"If the `.tri` sources exist and already contain the bodies, this is not a
> 571-decision problem at all — it is one regeneration. Check for the sources
> first."*

Checked:

| | |
|---|---:|
| `.tri` files in the repository | 26 |
| Empty-body specs with a same-named `.tri` | **1** |
| …and that one | `specs/tri/graph/graph.t27` ↔ `architecture/graph.tri` — a basename collision with an architecture diagram |
| Function declarations across all 26 `.tri` files | 94 |
| …of those, with a body | **5** |

**The sources do not exist and do not contain implementations.** The header
comment points at something that is not there. Variant A is not a regeneration
task, and each of the 571 empty functions is a spec-authoring decision.

So the wave took Variant B, which is what the falsification discipline is for.

## 2. `t27c impl-status`

A static measure over the AST: a function declaration with no statements is
exactly what the Zig backend turns into `@compileError("not yet implemented")`.

```
  specs fully implemented   232
  specs PARTLY written        6
  specs entirely UNWRITTEN  159
  specs that do not parse   211

  functions declared       2854
  functions with NO BODY    667      (23%)
```

**159 of the 397 specs that parse — 40% — have no implementation at all.** Every
function empty. Six more are partly written, which is the interesting minority:
somebody started those.

## 3. The harness now says which

```
ALL_PASS       28   683 tests passing
UNIMPLEMENTED 118   functions have no bodies
COMPILE_FAIL   98   actually broken
```

`COMPILE_FAIL 216 → 98`. The number this chain has been driving down since W560
was **more than half composed of specs nobody had written**, and no compiler work
could ever have moved that half.

Both `t27c impl-status` and the harness verdict are wired into suite Phase 6.

## 4. What this changes about every earlier number

Nothing measured in earlier waves was wrong, but several were reported against a
denominator that silently included unwritten specs:

- W560's *"45 of 14,996 tests execute"* — the denominator counted tests over
  functions that did not exist.
- Every `COMPILE_FAIL` count from W560 to W585 conflated the two states.
- The C gate's 296 failing headers include the same population; `t27c cc-gate`
  does not yet split them, which is W587 Variant B.

This is the second measurement-hygiene finding in three waves, after W580's 15
Markdown documents named `*.t27`. Both have the same shape: **a denominator
containing things that can never pass.**

---

## 5. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| `t27c impl-status` unit tests | 2 passed |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| T1 / T2 / T3 | re-proved |

---

## 6. Three cooperation variants for W587

### Variant A (recommended) — The 98 that are genuinely broken

For the first time this chain has a failure list with nothing unwritten in it.
98 specs, each with a real defect. Taxonomise them the way W578 taxonomised the
parse failures — by first error, weighted by the assertions each class releases —
and work the top.

**Why now.** Every previous taxonomy of `COMPILE_FAIL` was diluted by 118 specs
that could not be fixed by any compiler change. This is the first ranking that
points only at real work.

### Variant B — Split the C gate the same way

`t27c cc-gate` reports 296 failing headers and does not know which of them are
unwritten. The same `impl_status` predicate applies; the two commands should
agree on the denominator.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, preflight correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** The list is clean for the first time in twenty-six waves.

---

*φ² + φ⁻² = 3 | TRINITY*
